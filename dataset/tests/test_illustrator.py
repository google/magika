from magika_datasets.validators.text import pdf, postscript


def minimal_pdf(extra: bytes = b"") -> bytes:
    body = b"%PDF-1.4\n1 0 obj\n<< /Type /Catalog " + extra + b" >>\nendobj\n"
    xref = len(body)
    body += (
        b"xref\n0 2\n0000000000 65535 f \n0000000009 00000 n \ntrailer\n<< /Size 2 /Root 1 0 R >>\n"
    )
    return body + b"startxref\n" + str(xref).encode() + b"\n%%EOF\n"


def test_pdf_with_private_data_is_named_ai():
    assert pdf.validate(minimal_pdf(), frozenset()).format_id == "pdf"
    result = pdf.validate(minimal_pdf(b"/AIPrivateData 2 0 R"), frozenset())
    assert result.status == "pass" and result.format_id == "ai"


def test_eps_with_illustrator_comments_is_named_ai():
    plain = b"%!PS-Adobe-3.0 EPSF-3.0\n%%BoundingBox: 0 0 10 10\n{ 1 2 add } def\n"
    assert postscript.validate(plain, frozenset()).format_id == "postscript"
    ai = b"%!PS-Adobe-3.0 EPSF-3.0\n%%Creator: Adobe Illustrator(R) 8.0\n%AI8_CreatorVersion: 8.0\n%%BoundingBox: 0 0 10 10\n{ 1 2 add } def\n"
    result = postscript.validate(ai, frozenset())
    assert result.status == "pass" and result.format_id == "ai" and result.tags == ("eps",)
