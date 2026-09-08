import pytest
from helpers import status

from magika_datasets.validators.text import xml_family


def result(data, hints=frozenset()):
    return xml_family.validate(data, hints)


def test_svg_identity_not_html_or_wrong_namespace():
    assert result(b'<svg xmlns="http://www.w3.org/2000/svg"/>').format_id == "svg"
    generic = result(b'<svg xmlns="http://fake.example"/>')
    assert (generic.status, generic.format_id, generic.generic) == ("pass", "xml", True)
    assert result(b"<html><svg/></html>").format_id == "xml"
    assert status(xml_family, b'<svg xmlns="http://www.w3.org/2000/svg">') == "fail"
    assert result(b"<svg", frozenset({"svg"})).format_id == "svg"  # failures attributed to the hint


@pytest.mark.parametrize("viewbox", ["0 0 -1 2", "0 0 NaN 2", "0 0 inf 2", "0 0 1", "x y 2 2"])
def test_bad_viewbox(viewbox):
    assert (
        status(
            xml_family, f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="{viewbox}"/>'.encode()
        )
        == "fail"
    )


def test_scripts_are_recorded_without_execution_and_entities_refused():
    scripted = result(
        b'<svg xmlns="http://www.w3.org/2000/svg"><script>throw "not executed";</script></svg>'
    )
    assert scripted.status == "pass" and "contains_scripts" in scripted.tags
    entity = b'<!DOCTYPE svg [<!ENTITY secret SYSTEM "file:///etc/passwd">]><svg xmlns="http://www.w3.org/2000/svg">&secret;</svg>'
    assert status(xml_family, entity) == "inconclusive"


@pytest.mark.parametrize(
    "document,kind",
    [
        (
            b'<gpx xmlns="http://www.topografix.com/GPX/1/1" version="1.1"><wpt lat="1" lon="2"/></gpx>',
            "gpx",
        ),
        (b'<kml xmlns="http://www.opengis.net/kml/2.2"><Document/></kml>', "kml"),
        (
            b'<COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1"><asset/></COLLADA>',
            "collada",
        ),
        (b'<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"/>', "rdf"),
        (b'<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"/>', "xsd"),
        (
            b'<?xml version="1.0"?><!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd"><plist version="1.0"><dict><key>a</key><integer>1</integer></dict></plist>',
            "appleplist",
        ),
        (
            b'<assembly xmlns="urn:schemas-microsoft-com:asm.v3" manifestVersion="1.0"><assemblyIdentity name="x"/></assembly>',
            "mum",
        ),
        (b'<osm version="0.6"><node id="1" lat="0" lon="0"/></osm>', "osm"),
        (
            b'<gml:FeatureCollection xmlns:gml="http://www.opengis.net/gml"><gml:featureMember/></gml:FeatureCollection>',
            "gml",
        ),
        (
            b'<wfs:FeatureCollection xmlns:wfs="http://www.opengis.net/wfs" xmlns:gml="http://www.opengis.net/gml/3.2"><gml:boundedBy/></wfs:FeatureCollection>',
            "gml",
        ),
        (b'<?xml version="1.0"?><root><child/></root>', "xml"),
        (
            b'<FeatureCollection xmlns="http://www.opengis.net/wfs"><member><Feature><geom xmlns="http://www.opengis.net/gml/3.2"/></Feature></member></FeatureCollection>',
            "gml",
        ),
    ],
)
def test_namespace_dispatch(document, kind):
    observation = result(document)
    assert (observation.status, observation.format_id) == ("pass", kind), kind
    assert observation.generic == (kind == "xml")


def test_plist_values_are_checked_and_encoding_declarations_survive():
    assert (
        status(xml_family, b'<plist version="1.0"><dict><key>a</key><bogus/></dict></plist>')
        == "fail"
    )
    assert status(xml_family, b'<?xml version="1.0" encoding="GB2312"?><a/>') == "inconclusive"
    assert status(xml_family, b'\xef\xbb\xbf<?xml version="1.0"?><a/>') == "pass"
    assert status(xml_family, b"plain text") == "not_applicable"
    assert xml_family.PREFIX_ONLY
