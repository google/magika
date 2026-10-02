import pytest

from magika_datasets.validators._shared import xml


def test_parse_returns_root_and_classifies_errors():
    assert xml.parse(b'<a xmlns="urn:x"><b/></a>').tag == "{urn:x}a"
    with pytest.raises(xml.Malformed):
        xml.parse(b"<a>")
    with pytest.raises(xml.Unsafe):
        xml.parse(b'<!DOCTYPE a [<!ENTITY e SYSTEM "file:///etc/passwd">]><a>&e;</a>')
    with pytest.raises(xml.Unsafe):
        xml.parse(b'<!DOCTYPE a [<!ENTITY lol "lol">]><a>&lol;</a>')
    doctype = b"<!DOCTYPE qgis PUBLIC 'http://mrcc.com/qgis.dtd' 'SYSTEM'><qgis version=\"3.28\"/>"
    assert xml.parse(doctype).tag == "qgis"


def test_unsupported_declared_encoding_is_reported_not_raised():
    with pytest.raises(xml.Unsupported):
        xml.parse(b'<?xml version="1.0" encoding="GB2312"?><a/>')
