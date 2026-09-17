from helpers import status

from magika_datasets.validators.text import eml, pdf, postscript, rtf


def test_rtf_groups():
    good = b"{\\rtf1\\ansi{\\fonttbl{\\f0 Arial;}}\\'e9 escaped \\{brace\\} text\\par}"
    observation = rtf.validate(good, frozenset())
    assert (observation.status, observation.format_id) == ("pass", "rtf")
    assert status(rtf, good[:-1]) == "fail"
    assert status(rtf, good + b"}") == "fail"
    assert status(rtf, good + b"after") == "fail"
    assert status(rtf, good + b"\r\n\0") == "pass"  # Word trailer
    assert status(rtf, b"{\\rtf2 x}") == "fail"
    assert status(rtf, b"{\\pict}") == "not_applicable"


def test_postscript_structure():
    good = b"%!PS-Adobe-3.0 EPSF-3.0\n%%BoundingBox: 0 0 10 10\n/box { newpath 0 0 moveto (paren (nested) \\) text) show } def\n<< /Key [1 2] >> pop\nbox\n%%EOF\n"
    observation = postscript.validate(good, frozenset())
    assert (observation.status, observation.format_id) == ("pass", "postscript")
    assert "eps" in observation.tags
    assert status(postscript, good.replace(b"} def", b" def")) == "fail"
    assert status(postscript, good.replace(b">> pop", b"> pop")) == "fail"
    assert status(postscript, b"%!PS\n(unterminated string\n") == "fail"
    assert status(postscript, b"%PDF-1.4") == "not_applicable"


def pdf_file(objects=2, xref=True, trailing=b"", encrypted=False, javascript=False):
    body = b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n"
    offsets = []
    for number in range(1, objects + 1):
        offsets.append(len(body))
        content = (
            b"<< /Type /Catalog /Pages 2 0 R >>"
            if number == 1
            else b"<< /Type /Pages /Count 0 /Kids [] >>"
        )
        if javascript and number == 2:
            content = b"<< /Type /Pages /Count 0 /Kids [] /JavaScript << /JS (app.alert(1)) >> >>"
        body += b"%d 0 obj\n" % number + content + b"\nendobj\n"
    start = len(body)
    if xref:
        body += b"xref\n0 %d\n0000000000 65535 f \n" % (objects + 1)
        for offset in offsets:
            body += b"%010d 00000 n \n" % offset
        trailer = b"<< /Size %d /Root 1 0 R" % (objects + 1)
        if encrypted:
            trailer += b" /Encrypt 3 0 R"
        body += b"trailer\n" + trailer + b" >>\n"
    body += b"startxref\n%d\n%%%%EOF\n" % start
    return body + trailing


def test_pdf_objects_xref_and_trailer():
    observation = pdf.validate(pdf_file(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "pdf")
    assert "encrypted" in pdf.validate(pdf_file(encrypted=True), frozenset()).tags
    assert "has_javascript" in pdf.validate(pdf_file(javascript=True), frozenset()).tags
    assert status(pdf, pdf_file()[:-6]) == "fail"
    assert status(pdf, pdf_file(xref=False)) == "fail"
    assert status(pdf, pdf_file().replace(b"endobj", b"endobx", 1)) == "fail"
    assert status(pdf, pdf_file(trailing=b"garbage after eof" * 100)) == "fail"
    assert status(pdf, pdf_file(trailing=b"\n")) == "pass"
    assert status(pdf, b"%PDF-1.4\n" + b"\0" * 10) == "fail"
    assert status(pdf, b"%PDX-1.4") == "not_applicable"


def message(boundary_ok=True, related=False):
    head = b"From: a@example.com\r\nTo: b@example.com\r\nSubject: Hi\r\nDate: Mon, 1 Jan 2024 00:00:00 +0000\r\nMessage-ID: <1@example.com>\r\nMIME-Version: 1.0\r\n"
    kind = b"multipart/related" if related else b"multipart/mixed"
    head += b"Content-Type: " + kind + b'; boundary="B"\r\n\r\n'
    parts = b"--B\r\nContent-Type: text/plain\r\n\r\nhello\r\n--B\r\nContent-Type: text/html\r\n\r\n<p>hi</p>\r\n"
    return head + parts + (b"--B--\r\n" if boundary_ok else b"")


def test_eml_and_mht():
    observation = eml.validate(message(), frozenset({"eml"}))
    assert (observation.status, observation.format_id) == ("pass", "eml")
    assert status(eml, message(boundary_ok=False), frozenset({"eml"})) == "fail"
    assert eml.validate(message(related=True), frozenset({"mht"})).format_id == "mht"
    assert (
        status(eml, message(), frozenset({"mht"})) == "pass"
    )  # multipart/mixed MHTML exists in the wild
    assert (
        status(eml, b"From: a@b\r\nContent-Type: text/plain\r\n\r\nx", frozenset({"mht"})) == "fail"
    )
    assert status(eml, b"just text\n", frozenset({"eml"})) == "not_applicable"
    assert eml.REQUIRES_HINT and eml.CONTEXT_REQUIRED  # never run unhinted by observe()
