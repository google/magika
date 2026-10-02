from magika_datasets.validators.text import hta

PAGE = b"""<html><head><title>Tool</title>
<HTA:APPLICATION ID="tool" APPLICATIONNAME="Tool" BORDER="thin"/>
<script language="VBScript">Sub Window_OnLoad : End Sub</script>
</head><body>Hello</body></html>"""


def test_an_hta_page_passes():
    result = hta.validate(PAGE, frozenset())
    assert result.status == "pass" and "(Tool)" in result.detail


def test_utf16_pages_are_decoded():
    assert (
        hta.validate(b"\xff\xfe" + PAGE.decode().encode("utf-16-le"), frozenset()).status == "pass"
    )


def test_a_tag_only_inside_script_fails():
    page = b'<html><body><script>document.write("<hta:application id=x>")</script></body></html>'
    assert hta.validate(page, frozenset()).status == "fail"


def test_a_script_that_writes_an_hta_is_not_claimed():
    dropper = b'Set f = fso.CreateTextFile("x.hta")\r\nf.WriteLine "<HTA:APPLICATION ID=x>"\r\n'
    assert hta.validate(dropper, frozenset()) is None
