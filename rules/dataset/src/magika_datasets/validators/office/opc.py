# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Open Packaging Conventions dispatch: Office, Visio, NuGet, MSIX and 3MF packages."""

import posixpath
from urllib.parse import unquote, urlsplit

from .._shared.zip import Failure, Package, member_xml
from ..contract import Observation

CT = "http://schemas.openxmlformats.org/package/2006/content-types"
REL = "http://schemas.openxmlformats.org/package/2006/relationships"
OFFICE = {
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument",
    "http://purl.oclc.org/ooxml/officeDocument/relationships/officeDocument",
    "http://schemas.microsoft.com/visio/2010/relationships/document",
}
# kind: (main root local name, schema family or None for binary/foreign roots, main content types)
MAIN = {
    "docx": (
        "document",
        "wordprocessingml",
        {
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml",
            "application/vnd.ms-word.document.macroEnabled.main+xml",
        },
    ),
    "dotx": (
        "document",
        "wordprocessingml",
        {
            "application/vnd.openxmlformats-officedocument.wordprocessingml.template.main+xml",
            "application/vnd.ms-word.template.macroEnabledTemplate.main+xml",
        },
    ),
    "xlsx": (
        "workbook",
        "spreadsheetml",
        {
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml",
            "application/vnd.ms-excel.sheet.macroEnabled.main+xml",
            "application/vnd.openxmlformats-officedocument.spreadsheetml.template.main+xml",
            "application/vnd.ms-excel.template.macroEnabled.main+xml",
            "application/vnd.ms-excel.addin.macroEnabled.main+xml",
        },
    ),
    "xlsb": ("workbook", None, {"application/vnd.ms-excel.sheet.binary.macroEnabled.main"}),
    "pptx": (
        "presentation",
        "presentationml",
        {
            "application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml",
            "application/vnd.ms-powerpoint.presentation.macroEnabled.main+xml",
            "application/vnd.openxmlformats-officedocument.presentationml.slideshow.main+xml",
            "application/vnd.ms-powerpoint.slideshow.macroEnabled.main+xml",
            "application/vnd.openxmlformats-officedocument.presentationml.template.main+xml",
            "application/vnd.ms-powerpoint.template.macroEnabled.main+xml",
            "application/vnd.ms-powerpoint.addin.macroEnabled.main+xml",
        },
    ),
    "visio": (
        "VisioDocument",
        None,
        {
            "application/vnd.ms-visio.drawing.main+xml",
            "application/vnd.ms-visio.drawing.macroEnabled.main+xml",
            "application/vnd.ms-visio.template.main+xml",
            "application/vnd.ms-visio.template.macroEnabled.main+xml",
            "application/vnd.ms-visio.stencil.main+xml",
            "application/vnd.ms-visio.stencil.macroEnabled.main+xml",
        },
    ),
}
VISIO_NS = "http://schemas.microsoft.com/office/visio/2012/main"
APPX_NS = {
    "http://schemas.microsoft.com/appx/manifest/foundation/windows10",
    "http://schemas.microsoft.com/appx/2010/manifest",
}
MODEL_CT = "application/vnd.ms-package.3dmanufacturing-3dmodel+xml"
MODEL_REL = "http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"
MODEL_NS = "http://schemas.microsoft.com/3dmanufacturing/core/2015/02"
SCOPE = "OPC content types, package relationships, main part identity and XML well-formedness; no XSD, rendering, formula or VBA validation"


def target_name(relationship) -> str | None:
    """Internal relationship target as a normalized member name, else None."""
    if relationship.get("TargetMode", "Internal") != "Internal":
        return None
    target = urlsplit(relationship.get("Target", ""))
    if target.scheme or target.netloc or target.query or target.fragment:
        return None
    return posixpath.normpath(unquote(target.path).lstrip("/"))


def content_types(package: Package) -> dict[str, str]:
    """Effective content type of every member: Override first, then Default by extension."""
    root = member_xml(package, "[Content_Types].xml", "zip")
    if root.tag != "{" + CT + "}Types":
        raise Failure("zip", "Invalid content types root")
    defaults, overrides = {}, {}
    for entry in root.findall("{" + CT + "}Default"):
        defaults[entry.get("Extension", "").lower()] = entry.get("ContentType", "")
    for entry in root.findall("{" + CT + "}Override"):
        name = entry.get("PartName", "")
        if name in overrides:
            raise Failure("zip", "Duplicate content type override")
        overrides[name] = entry.get("ContentType", "")
    types = dict(overrides)  # an override naming a missing part stays visible as a mismatch
    for name in package.names:
        extension = name.rsplit("/", 1)[-1].rpartition(".")[2].lower()
        if "/" + name not in types and extension in defaults:
            types["/" + name] = defaults[extension]
    return types


def inspect(package: Package) -> Observation | None:
    """Most specific OPC-based format proven by the package, or None for plain OPC."""
    if "[Content_Types].xml" not in package.contents:
        return None
    types = content_types(package)
    if "AppxManifest.xml" in package.contents:
        return appx(package)
    nuspec = [name for name in package.names if "/" not in name and name.endswith(".nuspec")]
    if nuspec:
        return nupkg(package, nuspec[0])
    if MODEL_CT in types.values():
        return model(package)
    return office(package, types)


def appx(package: Package) -> Observation:
    root = member_xml(package, "AppxManifest.xml", "msix")
    if root.tag not in {"{" + ns + "}Package" for ns in APPX_NS}:
        raise Failure("msix", "AppxManifest root is not an appx Package")
    return Observation(
        "pass", "MSIX package checked: ZIP integrity, content types and appx manifest root", "msix"
    )


def nupkg(package: Package, nuspec: str) -> Observation:
    root = member_xml(package, nuspec, "nupkg")
    if root.tag.rsplit("}", 1)[-1] != "package":
        raise Failure("nupkg", "nuspec root is not a package element")
    return Observation(
        "pass", "NuGet package checked: ZIP integrity, content types and nuspec root", "nupkg"
    )


def relationships(package: Package, kind: str):
    if "_rels/.rels" not in package.contents:
        raise Failure(kind, "Missing package relationships")
    root = member_xml(package, "_rels/.rels", kind)
    if root.tag != "{" + REL + "}Relationships":
        raise Failure(kind, "Invalid relationships root")
    return root


def model(package: Package) -> Observation:
    links = [r for r in relationships(package, "3mf") if r.get("Type") == MODEL_REL]
    if len(links) != 1:
        raise Failure("3mf", "Missing or ambiguous 3D model relationship")
    name = target_name(links[0])
    if name is None or name not in package.contents:
        raise Failure("3mf", "3D model relationship target missing")
    root = member_xml(package, name, "3mf")
    if root.tag != "{" + MODEL_NS + "}model":
        raise Failure("3mf", "3D model root differs from 3MF core namespace")
    return Observation(
        "pass",
        "3MF package checked: ZIP integrity, content types, model relationship and root",
        "3mf",
    )


def office(package: Package, types: dict[str, str]) -> Observation | None:
    matching = [
        (kind, part)
        for part, content_type in types.items()
        for kind, (_, _, types) in MAIN.items()
        if content_type in types
    ]
    if not matching:
        return None
    kind, part = matching[0]
    if len(matching) != 1:
        raise Failure(kind, "Multiple Office main parts")
    root_name, family, _ = MAIN[kind]
    mains = [r for r in relationships(package, kind) if r.get("Type") in OFFICE]
    if len(mains) != 1:
        raise Failure(kind, "Missing or ambiguous Office main relationship")
    name = target_name(mains[0])
    if name is None or "/" + name != part or name not in package.contents:
        raise Failure(kind, "Main relationship/content-type mismatch")
    roots = {}
    for member in package.names:
        if member.endswith((".xml", ".rels")):
            roots[member] = member_xml(package, member, kind)
    if family:
        expected = {
            f"{{http://schemas.openxmlformats.org/{family}/2006/main}}{root_name}",
            f"{{http://purl.oclc.org/ooxml/{family}/main}}{root_name}",
        }
    elif kind == "visio":
        expected = {"{" + VISIO_NS + "}" + root_name}
    else:
        expected = None
    if expected is not None:
        root = roots.get(name)
        if root is None or root.tag not in expected:
            raise Failure(kind, "Main XML root differs from Office content type")
        if family == "wordprocessingml" and root.find(root.tag.rsplit("}", 1)[0] + "}body") is None:
            raise Failure(kind, "Word document body missing")
    macro = "macroEnabled" in types[part] and kind != "xlsb"
    vba = any(member.lower().endswith("/vbaproject.bin") for member in package.names)
    # A VBA part under a non-macro content type is ignored by Office, not a broken package.
    variant = next(
        (v for v in ("slideshow", "template", "addin") if v in types[part].lower()), None
    )
    facts = [
        ("macro_enabled", macro),
        ("vba_present", vba),
        ("vba_unreferenced", vba and not macro),
        (variant, variant and kind != "dotx"),
    ]
    tags = tuple(name for name, value in facts if value)
    return Observation(
        "pass",
        f"Office package checked; macro_enabled={macro}; vba_present={vba}; VBA not executed",
        kind,
        tags,
    )
