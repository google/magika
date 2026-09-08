# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""ZIP containers walked once; the most specific provable package format is named."""

import json
import struct
import zipfile

from .._shared import xml
from .._shared import zip as zipwalk
from ..contract import Observation
from ..data import numpy
from ..data import pickle as pickle_stream
from ..office import opc

FAMILY = "archive"
FORMAT_IDS = (
    "zip",
    "docx",
    "dotx",
    "xlsx",
    "xlsb",
    "pptx",
    "visio",
    "nupkg",
    "msix",
    "3mf",
    "epub",
    "odt",
    "ods",
    "odp",
    "apk",
    "xpi",
    "jar",
    "kmz",
    "crx",
    "qgis",
    "npz",
    "keras",
    "pytorch",
)
SHARED_FORMAT_IDS = ("visio",)  # binary .vsd files are named by office/cfb.py
SCOPE = "Central directory, member bounds, CRCs and safe names of every member; package-specific manifests parsed for identity; member contents not executed"
CONTAINER_NS = "urn:oasis:names:tc:opendocument:xmlns:container"
ODF_NS = "urn:oasis:names:tc:opendocument:xmlns:office:1.0"
MIMETYPES = {
    "application/epub+zip": "epub",
    "application/vnd.oasis.opendocument.text": "odt",
    "application/vnd.oasis.opendocument.spreadsheet": "ods",
    "application/vnd.oasis.opendocument.presentation": "odp",
}


MANIFEST_NS = "urn:oasis:names:tc:opendocument:xmlns:manifest:1.0"


def declared_media(package: zipwalk.Package) -> tuple[str | None, bool]:
    """Package media type from the mimetype member or the ODF manifest, and whether conforming."""
    if "mimetype" in package.contents:
        media = package.contents["mimetype"].strip().decode("ascii", "replace")
        first = package.infos[0]
        conforming = first.filename == "mimetype" and first.compress_type == zipfile.ZIP_STORED
        return media, conforming
    if "META-INF/manifest.xml" in package.contents:
        root = zipwalk.member_xml(package, "META-INF/manifest.xml", "odt")
        if root.tag == "{" + MANIFEST_NS + "}manifest":
            for entry in root.findall("{" + MANIFEST_NS + "}file-entry"):
                if entry.get("{" + MANIFEST_NS + "}full-path") == "/":
                    return entry.get("{" + MANIFEST_NS + "}media-type"), False
    return None, False


def mimetype(package: zipwalk.Package) -> Observation | None:
    media, conforming = declared_media(package)
    kind = MIMETYPES.get(media)
    if kind is None:
        return None
    tags = () if conforming else ("mimetype_nonconforming",)
    if kind == "epub":
        if "META-INF/container.xml" not in package.contents:
            raise zipwalk.Failure(kind, "Missing META-INF/container.xml")
        container = zipwalk.member_xml(package, "META-INF/container.xml", kind)
        rootfiles = container.findall(f".//{{{CONTAINER_NS}}}rootfile")
        if not rootfiles or rootfiles[0].get("full-path", "") not in package.contents:
            raise zipwalk.Failure(kind, "EPUB rootfile missing")
        zipwalk.member_xml(package, rootfiles[0].get("full-path"), kind)
        return Observation(
            "pass",
            f"EPUB checked: ZIP integrity, container and rootfile; mimetype_conforming={conforming}",
            kind,
            tags,
        )
    if "content.xml" not in package.contents:
        raise zipwalk.Failure(kind, "Missing content.xml")
    root = zipwalk.member_xml(package, "content.xml", kind)
    if root.tag != "{" + ODF_NS + "}document-content":
        raise zipwalk.Failure(kind, "content.xml root is not an OpenDocument content element")
    return Observation(
        "pass",
        f"OpenDocument package checked: ZIP integrity and content root; mimetype_conforming={conforming}",
        kind,
        tags,
    )


KML_NS = {
    "http://www.opengis.net/kml/2.2",
    "http://earth.google.com/kml/2.0",
    "http://earth.google.com/kml/2.1",
    "http://earth.google.com/kml/2.2",
}
RDF_NS = "http://www.w3.org/1999/02/22-rdf-syntax-ns#"


def apk(package: zipwalk.Package) -> Observation | None:
    """Android packages, including resource-only and native-only split APKs without classes.dex."""
    if "AndroidManifest.xml" not in package.contents:
        return None
    native = any(name.startswith("lib/") and name.endswith(".so") for name in package.contents)
    if (
        "classes.dex" not in package.contents
        and "resources.arsc" not in package.contents
        and not native
    ):
        return None

    if not package.contents["AndroidManifest.xml"].startswith(b"\x03\x00\x08\x00"):
        raise zipwalk.Failure("apk", "AndroidManifest.xml is not Android binary XML")
    dex = "classes.dex" in package.contents
    if dex and not package.contents["classes.dex"].startswith(b"dex\n"):
        raise zipwalk.Failure("apk", "classes.dex lacks the DEX magic")
    return Observation(
        "pass",
        f"APK checked: ZIP integrity, binary manifest, dex_present={dex}; nothing executed",
        "apk",
        (() if dex else ("no_dex",)) + (("native_split",) if native and not dex else ()),
    )


def xpi(package: zipwalk.Package) -> Observation | None:
    if "manifest.json" in package.contents:
        try:
            manifest = json.loads(package.contents["manifest.json"].decode("utf-8-sig"))
        except (UnicodeError, ValueError):
            manifest = None  # Chrome tolerates comments here; not evidence for or against XPI
        gecko = (
            isinstance(manifest, dict)
            and "manifest_version" in manifest
            and ("browser_specific_settings" in manifest or "applications" in manifest)
        )
        if gecko:
            return Observation(
                "pass", "XPI checked: ZIP integrity and Gecko WebExtension manifest", "xpi"
            )
    if "install.rdf" in package.contents:
        root = zipwalk.member_xml(package, "install.rdf", "xpi")
        if root.tag != "{" + RDF_NS + "}RDF":
            raise zipwalk.Failure("xpi", "install.rdf root is not RDF")
        return Observation("pass", "XPI checked: ZIP integrity and install.rdf root", "xpi")
    return None


def jar(package: zipwalk.Package) -> Observation | None:
    if "META-INF/MANIFEST.MF" not in package.contents:
        return None
    lines = package.contents["META-INF/MANIFEST.MF"].decode("utf-8", "replace").splitlines()
    first = next((line for line in lines if line.strip()), "")
    if not first.startswith("Manifest-Version:"):
        raise zipwalk.Failure("jar", "MANIFEST.MF lacks Manifest-Version")
    return Observation(
        "pass", "JAR checked: ZIP integrity and manifest header; classes not loaded", "jar"
    )


def kmz(package: zipwalk.Package) -> Observation | None:
    documents = [n for n in package.names if "/" not in n and n.lower().endswith(".kml")]
    if not documents:
        return None
    root = zipwalk.member_xml(package, documents[0], "kmz")
    if root.tag not in {"{" + ns + "}kml" for ns in KML_NS}:
        raise zipwalk.Failure("kmz", "Root KML document is not a KML element")
    return Observation("pass", "KMZ checked: ZIP integrity and root KML document", "kmz")


def qgis(package: zipwalk.Package) -> Observation | None:
    """QGIS .qgz: a ZIP whose root .qgs member is a QGIS project XML document."""
    projects = [n for n in package.names if "/" not in n and n.lower().endswith(".qgs")]
    if not projects:
        return None
    root = zipwalk.member_xml(package, projects[0], "qgis")
    if root.tag != "qgis":
        raise zipwalk.Failure("qgis", "Root .qgs member is not a QGIS project element")
    return Observation("pass", "QGZ checked: ZIP integrity and root QGIS project document", "qgis")


def npz(package: zipwalk.Package) -> Observation | None:
    """NumPy archives: every member is a complete .npy array."""
    if not package.names or not all(name.endswith(".npy") for name in package.names):
        return None
    for name in package.names:
        if not numpy.payload_matches(package.contents[name]):
            raise zipwalk.Failure("npz", f"Member {name} is not a complete npy array")
    return Observation("pass", f"npz with {len(package.names)} exactly sized arrays", "npz")


def keras(package: zipwalk.Package) -> Observation | None:
    """Keras 3 model archives: config and metadata JSON plus an HDF5 weights member."""
    if "config.json" not in package.contents or "metadata.json" not in package.contents:
        return None
    if "model.weights.h5" not in package.contents:
        return None
    for name in ("config.json", "metadata.json"):
        try:
            json.loads(package.contents[name].decode("utf-8"))
        except (UnicodeError, ValueError) as error:
            raise zipwalk.Failure("keras", f"{name} is not valid JSON") from error
    if not package.contents["model.weights.h5"].startswith(b"\x89HDF\r\n\x1a\n"):
        raise zipwalk.Failure("keras", "model.weights.h5 is not HDF5")
    return Observation(
        "pass",
        "Keras archive: config, metadata and HDF5 weights present; weights not decoded",
        "keras",
    )


def pytorch(package: zipwalk.Package) -> Observation | None:
    """PyTorch zip checkpoints: <name>/data.pkl walked with pickletools plus data/ and version."""
    pickles = [n for n in package.names if n.endswith("/data.pkl") and n.count("/") == 1]
    if not pickles:
        return None
    root = pickles[0].split("/")[0]
    if f"{root}/version" not in package.contents:
        return None
    ok, detail = pickle_stream.stream_ok(package.contents[pickles[0]])
    if not ok:
        raise zipwalk.Failure("pytorch", f"data.pkl: {detail}")
    storages = sum(1 for n in package.names if n.startswith(f"{root}/data/"))
    return Observation(
        "pass",
        f"PyTorch archive: data.pkl walked, {storages} storages, version present; nothing loaded",
        "pytorch",
    )


# APK before JAR: Android packages may also carry a META-INF/MANIFEST.MF.
PROBES = (opc.inspect, mimetype, apk, xpi, jar, kmz, qgis, npz, keras, pytorch)


def validate(data: bytes, hints: frozenset[str]) -> Observation | None:
    if data.startswith(b"Cr24"):
        return crx(data)
    if not data.startswith(b"PK\x03\x04"):
        return None
    return inspect(data, "zip")


def crx(data: bytes) -> Observation:
    if len(data) < 12:
        return Observation("fail", "Truncated CRX header", "crx")
    version = struct.unpack_from("<I", data, 4)[0]
    if version == 2:
        if len(data) < 16:
            return Observation("fail", "Truncated CRX2 header", "crx")
        key_length, signature_length = struct.unpack_from("<II", data, 8)
        offset = 16 + key_length + signature_length
    elif version == 3:
        offset = 12 + struct.unpack_from("<I", data, 8)[0]
    else:
        return Observation("fail", f"Unsupported CRX version {version}", "crx")
    if offset > len(data) or not data[offset:].startswith(b"PK\x03\x04"):
        return Observation("fail", "CRX header does not lead to a ZIP payload", "crx")
    inner = inspect(data[offset:], "crx")
    if inner.status != "pass":
        return Observation(inner.status, inner.detail, "crx")
    return Observation("pass", f"CRX{version} header bounds and complete inner ZIP checked", "crx")


def inspect(data: bytes, fallback: str) -> Observation:
    try:
        package = zipwalk.walk(data)
    except zipwalk.Budget as error:
        return Observation("inconclusive", str(error), fallback)
    except zipwalk.Unsupported as error:
        return Observation("inconclusive", str(error), fallback)
    except zipwalk.Malformed as error:
        return Observation("fail", str(error), fallback)
    try:
        for probe in PROBES:
            result = probe(package)
            if result is not None:
                return result
    except zipwalk.Failure as failure:
        return Observation("fail", failure.detail, failure.format_id)
    except (xml.Unsafe, xml.Unsupported) as error:
        return Observation("inconclusive", str(error), fallback)
    return Observation(
        "pass",
        f"ZIP central directory, member bounds and CRCs checked; {len(package.names)} members",
        fallback,
        generic=fallback == "zip",
    )
