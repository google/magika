from helpers import status

from magika_datasets.validators.system import winregistry


def test_registry_text_exports():
    export = b'Windows Registry Editor Version 5.00\r\n\r\n[HKEY_CURRENT_USER\\Software\\Test]\r\n"Name"="value"\r\n@=dword:00000001\r\n"Bin"=hex:00,01,\\\r\n  02,03\r\n; comment\r\n[-HKEY_CURRENT_USER\\Software\\Old]\r\n'
    observation = winregistry.validate(export, frozenset())
    assert (observation.status, observation.tags) == ("pass", ("export",))
    utf16 = b"\xff\xfe" + export.decode().encode("utf-16-le")
    assert winregistry.validate(utf16, frozenset()).tags == ("utf16", "export")
    assert status(winregistry, b"\xef\xbb\xbfREGEDIT4\r\n[HKEY_LOCAL_MACHINE\\X]\r\n") == "pass"
    lenient = b'Windows Registry Editor Version 5.00\r\n[HKLM\\A] ; trailing comment\r\n"Count"  = dword:9\r\n"Key With = 2"=dword:123\r\n'
    assert status(winregistry, lenient) == "pass"
    assert (
        status(winregistry, b"Windows Registry Editor Version 5.00\r\n# not a comment\r\n")
        == "fail"
    )
    assert status(winregistry, export + b"garbage line\r\n") == "fail"
    assert status(winregistry, export + b'"More"=hex:00,\\\r\n') == "fail"  # ends mid-continuation
    assert status(winregistry, b"Windows Registry Editor Version 6.00\r\n") == "not_applicable"
    assert status(winregistry, b"\xff\xfe\x00\xd8junk") == "not_applicable"
