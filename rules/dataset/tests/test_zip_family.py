import hashlib
import io
import zipfile

import pytest
from helpers import status

from magika_datasets.validators.archive import zip as zipfamily
from magika_datasets.validators.office.opc import CT, MAIN, OFFICE, REL

NS = "http://schemas.openxmlformats.org/{family}/2006/main"


def build(members, compression=zipfile.ZIP_DEFLATED, first_stored=None):
    stream = io.BytesIO()
    with zipfile.ZipFile(stream, "w", compression) as archive:
        if first_stored:
            archive.writestr(first_stored[0], first_stored[1], zipfile.ZIP_STORED)
        for name, payload in members.items():
            archive.writestr(name, payload)
    return stream.getvalue()


def office(kind="docx", *, missing=False, macro=False, vba=False, bad_xml=False):
    root, family, types = MAIN[kind]
    content_type = next((t for t in types if ("macro" in t) == macro), next(iter(types)))
    main = f"parts/{root}.xml" if family else "parts/workbook.bin"
    parts = {
        "[Content_Types].xml": f'<Types xmlns="{CT}"><Override PartName="/{main}" ContentType="{content_type}"/></Types>',
        "_rels/.rels": f'<Relationships xmlns="{REL}"><Relationship Id="rId1" Type="{sorted(OFFICE)[0]}" Target="{main}"/></Relationships>',
        main: f'<{root} xmlns="{NS.format(family=family)}"><body/></{root}>'
        if family
        else b"\x83\x01",
    }
    if missing:
        del parts[main]
    if bad_xml and family:
        parts[main] = "<broken"
    if vba:
        parts["parts/vbaProject.bin"] = "opaque VBA"
    return build(parts)


def result(data):
    return zipfamily.validate(data, frozenset())


@pytest.mark.parametrize("kind", ["docx", "xlsx", "pptx", "dotx", "xlsb"])
def test_office_identity_and_integrity(kind):
    good = result(office(kind))
    assert (good.status, good.format_id) == ("pass", kind)
    assert result(office(kind, missing=True)).status == "fail"
    if kind != "xlsb":
        assert result(office(kind, bad_xml=True)).status == "fail"
    assert result(office(kind)[:-12]).status == "fail"


def test_macro_enabled_content_type_is_retained_and_vba_not_executed():
    observation = result(office(macro=True, vba=True))
    assert observation.status == "pass"
    assert observation.tags == ("macro_enabled", "vba_present")
    orphan = result(office(vba=True))
    assert (orphan.status, orphan.format_id) == ("pass", "docx")
    assert orphan.tags == ("vba_present", "vba_unreferenced")


def test_plain_zip_and_non_office_opc():
    plain = result(build({"word/document.xml": "<document/>"}))
    assert (plain.status, plain.format_id) == ("pass", "zip")
    opc = result(build({"[Content_Types].xml": f'<Types xmlns="{CT}"/>'}))
    assert (opc.status, opc.format_id) == ("pass", "zip")
    assert status(zipfamily, b"not zip") == "not_applicable"


def test_expansion_budget_is_not_a_validity_failure():
    data = build({"[Content_Types].xml": b" " * (16 * 1024**2 + 1)})
    assert result(data).status == "inconclusive"


def test_macro_properties_survive_label_assignment(tmp_path):
    from magika_datasets.validation import apply_label
    from magika_datasets.validators import observe

    data = office(macro=True, vba=True)
    path = tmp_path / "sample"
    path.write_bytes(data)
    observation = observe(path, hashlib.sha256(data).hexdigest())
    decision = apply_label({"format_ids": ["zip"]}, observation, {"docx": {}, "zip": {}})
    assert decision["format_ids"] == ["docx"]
    assert decision["tags"] == ["macro_enabled", "vba_present"]


def test_visio_package():
    parts = {
        "[Content_Types].xml": f'<Types xmlns="{CT}"><Override PartName="/visio/document.xml" ContentType="application/vnd.ms-visio.drawing.main+xml"/></Types>',
        "_rels/.rels": f'<Relationships xmlns="{REL}"><Relationship Id="rId1" Type="http://schemas.microsoft.com/visio/2010/relationships/document" Target="visio/document.xml"/></Relationships>',
        "visio/document.xml": '<VisioDocument xmlns="http://schemas.microsoft.com/office/visio/2012/main"/>',
    }
    observation = result(build(parts))
    assert (observation.status, observation.format_id) == ("pass", "visio")


CONTAINER_NS = "urn:oasis:names:tc:opendocument:xmlns:container"
ODF_NS = "urn:oasis:names:tc:opendocument:xmlns:office:1.0"


def epub(rootfile="OEBPS/content.opf", stored=True):
    members = {
        "META-INF/container.xml": f'<container xmlns="{CONTAINER_NS}"><rootfiles><rootfile full-path="{rootfile}"/></rootfiles></container>',
        "OEBPS/content.opf": '<package xmlns="http://www.idpf.org/2007/opf"/>',
    }
    if stored:
        return build(members, first_stored=("mimetype", b"application/epub+zip"))
    return build({"mimetype": b"application/epub+zip", **members})


@pytest.mark.parametrize(
    "media,kind",
    [
        ("application/vnd.oasis.opendocument.text", "odt"),
        ("application/vnd.oasis.opendocument.spreadsheet", "ods"),
        ("application/vnd.oasis.opendocument.presentation", "odp"),
    ],
)
def test_opendocument_packages(media, kind):
    members = {"content.xml": f'<office:document-content xmlns:office="{ODF_NS}"/>'}
    observation = result(build(members, first_stored=("mimetype", media.encode())))
    assert (observation.status, observation.format_id) == ("pass", kind)
    broken = result(build({"content.xml": "<broken"}, first_stored=("mimetype", media.encode())))
    assert (broken.status, broken.format_id) == ("fail", kind)


def test_opendocument_identity_survives_nonconforming_or_missing_mimetype():
    media = "application/vnd.oasis.opendocument.text"
    manifest_ns = "urn:oasis:names:tc:opendocument:xmlns:manifest:1.0"
    content = {"content.xml": f'<office:document-content xmlns:office="{ODF_NS}"/>'}
    late = result(build({**content, "mimetype": media.encode()}))  # deflated, not first
    assert (late.status, late.format_id) == ("pass", "odt")
    assert "mimetype_nonconforming" in late.tags
    manifest = f'<manifest:manifest xmlns:manifest="{manifest_ns}"><manifest:file-entry manifest:full-path="/" manifest:media-type="{media}"/></manifest:manifest>'
    absent = result(build({**content, "META-INF/manifest.xml": manifest}))
    assert (absent.status, absent.format_id) == ("pass", "odt")
    assert "mimetype_nonconforming" in absent.tags
    unrelated = result(build({**content, "META-INF/manifest.xml": "<x/>"}))
    assert (unrelated.status, unrelated.format_id) == ("pass", "zip")


def test_epub_packages():
    observation = result(epub())
    assert (observation.status, observation.format_id) == ("pass", "epub")
    late = result(epub(stored=False))
    assert (late.status, late.format_id) == ("pass", "epub")
    assert "mimetype_nonconforming" in late.tags
    assert result(epub(rootfile="missing.opf")).status == "fail"


KML_NS = "http://www.opengis.net/kml/2.2"


def test_apk_requires_binary_manifest_and_dex():
    good = build({"AndroidManifest.xml": b"\x03\x00\x08\x00rest", "classes.dex": b"dex\n035\x00"})
    observation = result(good)
    assert (observation.status, observation.format_id) == ("pass", "apk")
    bad = result(build({"AndroidManifest.xml": b"<manifest/>", "classes.dex": b"dex\n035\x00"}))
    assert (bad.status, bad.format_id) == ("fail", "apk")
    split = result(
        build({"AndroidManifest.xml": b"\x03\x00\x08\x00rest", "resources.arsc": b"\x02\x00"})
    )
    assert (split.status, split.format_id) == ("pass", "apk")
    assert "no_dex" in split.tags
    signed = build(
        {
            "AndroidManifest.xml": b"\x03\x00\x08\x00rest",
            "resources.arsc": b"\x02\x00",
            "META-INF/MANIFEST.MF": "Manifest-Version: 1.0\r\n",
        }
    )
    assert result(signed).format_id == "apk"
    bad_dex = result(build({"AndroidManifest.xml": b"\x03\x00\x08\x00", "classes.dex": b"nope"}))
    assert (bad_dex.status, bad_dex.format_id) == ("fail", "apk")


def test_jar_requires_manifest_version():
    manifest = "Manifest-Version: 1.0\r\n\r\n"
    good = build({"META-INF/MANIFEST.MF": manifest, "a/B.class": b"\xca\xfe\xba\xbe"})
    observation = result(good)
    assert (observation.status, observation.format_id) == ("pass", "jar")
    bad = result(build({"META-INF/MANIFEST.MF": "Nothing: here\r\n"}))
    assert (bad.status, bad.format_id) == ("fail", "jar")


def test_xpi_requires_gecko_manifest_or_install_rdf():
    import json

    manifest = {"manifest_version": 2, "browser_specific_settings": {"gecko": {"id": "x@y"}}}
    observation = result(build({"manifest.json": json.dumps(manifest)}))
    assert (observation.status, observation.format_id) == ("pass", "xpi")
    chrome = result(build({"manifest.json": json.dumps({"manifest_version": 3})}))
    assert (chrome.status, chrome.format_id) == ("pass", "zip")
    rdf_ns = "http://www.w3.org/1999/02/22-rdf-syntax-ns#"
    rdf = result(build({"install.rdf": f'<RDF xmlns="{rdf_ns}"/>'}))
    assert (rdf.status, rdf.format_id) == ("pass", "xpi")
    broken = result(build({"manifest.json": "{not json", "install.rdf": "<RDF/>"}))
    assert (broken.status, broken.format_id) == ("fail", "xpi")


def test_kmz_requires_root_kml_document():
    observation = result(build({"doc.kml": f'<kml xmlns="{KML_NS}"/>', "img/x.png": b"\x89PNG"}))
    assert (observation.status, observation.format_id) == ("pass", "kmz")
    bad = result(build({"doc.kml": "<other/>"}))
    assert (bad.status, bad.format_id) == ("fail", "kmz")


def crx(version, payload):
    import struct

    if version == 2:
        key, signature = b"k" * 10, b"s" * 8
        header = struct.pack("<III", 2, len(key), len(signature)) + key + signature
        return b"Cr24" + header + payload
    header = b"\x0a\x03abc"
    return b"Cr24" + struct.pack("<II", 3, len(header)) + header + payload


def test_crx_wraps_a_complete_zip():
    import struct

    payload = build({"manifest.json": "{}"})
    for version in (2, 3):
        observation = result(crx(version, payload))
        assert (observation.status, observation.format_id) == ("pass", "crx"), version
    assert result(crx(3, payload)[:-3]).status == "fail"
    assert result(b"Cr24" + struct.pack("<II", 3, 999)).status == "fail"
    assert result(b"Cr24" + struct.pack("<II", 4, 0)).status == "fail"
    commented = build({"manifest.json": b'\xef\xbb\xbf{\n  // comment\n  "manifest_version": 3\n}'})
    assert (result(crx(3, commented)).status, result(commented).format_id) == ("pass", "zip")


def test_default_extension_content_types_identify_xlsb_and_3mf():
    ct = "application/vnd.ms-excel.sheet.binary.macroEnabled.main"
    parts = {
        "[Content_Types].xml": f'<Types xmlns="{CT}"><Default Extension="bin" ContentType="{ct}"/></Types>',
        "_rels/.rels": f'<Relationships xmlns="{REL}"><Relationship Id="rId1" Type="{sorted(OFFICE)[0]}" Target="xl/workbook.bin"/></Relationships>',
        "xl/workbook.bin": b"\x83\x01",
    }
    observation = result(build(parts))
    assert (observation.status, observation.format_id, observation.tags) == ("pass", "xlsb", ())
    model_ct = "application/vnd.ms-package.3dmanufacturing-3dmodel+xml"
    model_rel = "http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"
    parts = {
        "[Content_Types].xml": f'<Types xmlns="{CT}"><Default Extension="model" ContentType="{model_ct}"/></Types>',
        "_rels/.rels": f'<Relationships xmlns="{REL}"><Relationship Id="rel0" Type="{model_rel}" Target="/3D/3dmodel.model"/></Relationships>',
        "3D/3dmodel.model": '<model xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"/>',
    }
    observation = result(build(parts))
    assert (observation.status, observation.format_id) == ("pass", "3mf")


@pytest.mark.parametrize(
    "kind,content_type,tag",
    [
        ("pptx", "application/vnd.ms-powerpoint.slideshow.macroEnabled.main+xml", "slideshow"),
        (
            "pptx",
            "application/vnd.openxmlformats-officedocument.presentationml.template.main+xml",
            "template",
        ),
        ("pptx", "application/vnd.ms-powerpoint.addin.macroEnabled.main+xml", "addin"),
        ("xlsx", "application/vnd.ms-excel.addin.macroEnabled.main+xml", "addin"),
        (
            "xlsx",
            "application/vnd.openxmlformats-officedocument.spreadsheetml.template.main+xml",
            "template",
        ),
    ],
)
def test_office_variants_keep_their_family_and_record_the_variant(kind, content_type, tag):
    root, family, _ = MAIN[kind]
    main = f"parts/{root}.xml"
    parts = {
        "[Content_Types].xml": f'<Types xmlns="{CT}"><Override PartName="/{main}" ContentType="{content_type}"/></Types>',
        "_rels/.rels": f'<Relationships xmlns="{REL}"><Relationship Id="rId1" Type="{sorted(OFFICE)[0]}" Target="{main}"/></Relationships>',
        main: f'<{root} xmlns="{NS.format(family=family)}"><body/></{root}>',
    }
    observation = result(build(parts))
    assert (observation.status, observation.format_id) == ("pass", kind)
    assert tag in observation.tags


def test_generic_zip_pass_does_not_relabel_hinted_zip_based_formats(tmp_path):
    from magika_datasets.validators import observe

    data = build({"data.pkl": b"\x80\x02", "version": b"3"})
    path = tmp_path / "sample"
    path.write_bytes(data)
    sha = hashlib.sha256(data).hexdigest()
    hinted = observe(path, sha, hints={"pytorch"})["observations"][0]
    assert (hinted["format_id"], hinted["status"], hinted["auto_eligible"]) == (
        "zip",
        "pass",
        False,
    )
    assert observe(path, sha, hints={"zip", "unknown"})["observations"][0]["auto_eligible"]
    assert observe(path, sha)["observations"][0]["auto_eligible"]
    assert result(data).generic is True
    assert result(office()).generic is False


def test_qgz_project_archives():
    observation = result(build({"project.qgs": '<qgis version="3.28"/>', "project.qgd": b"SQLite"}))
    assert (observation.status, observation.format_id) == ("pass", "qgis")
    bad = result(build({"project.qgs": "<other/>"}))
    assert (bad.status, bad.format_id) == ("fail", "qgis")


def test_unsupported_xml_encoding_inside_a_package_is_inconclusive():
    parts = {"[Content_Types].xml": '<?xml version="1.0" encoding="GB2312"?><Types/>'}
    observation = result(build(parts))
    assert (observation.status, observation.format_id) == ("inconclusive", "zip")


def test_native_only_split_apk_is_named_apk_not_jar():
    import io
    import zipfile

    buffer = io.BytesIO()
    with zipfile.ZipFile(buffer, "w") as archive:
        archive.writestr("AndroidManifest.xml", b"\x03\x00\x08\x00" + bytes(12))
        archive.writestr("META-INF/MANIFEST.MF", b"Manifest-Version: 1.0\n")
        archive.writestr("lib/x86_64/libcode.so", b"\x7fELF" + bytes(60))
    from magika_datasets.validators.archive import zip as zip_family

    result = zip_family.validate(buffer.getvalue(), frozenset())
    assert result.format_id == "apk" and "native_split" in result.tags
