from magika_datasets.validators.text import vcard

CARD_3 = b"BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Ada Lovelace\r\nitem1.EMAIL;TYPE=INTERNET:ada@example.org\r\nNOTE:folded\r\n  across lines\r\nEND:VCARD\r\n"
CARD_21 = (
    b"BEGIN:VCARD\nVERSION:2.1\nN;CHARSET=UTF-8;ENCODING=QUOTED-PRINTABLE:=E6=9D=8E;=\n=E5=9B=9B\n"
    b"PHOTO;ENCODING=BASE64;TYPE=JPEG:/9j/4AAQ\nSkZJRgABAQ\n\nEND:VCARD\n"
)


def test_several_cards_pass():
    result = vcard.validate(CARD_3 + CARD_21, frozenset())
    assert result.status == "pass" and result.detail.startswith("2 cards")


def test_crcrlf_line_endings_and_padding_are_tolerated():
    data = CARD_3.replace(b"\r\n", b"\r\r\n") + b"\x1a\0\0"
    result = vcard.validate(data, frozenset())
    assert result.status == "pass" and result.tags == ("trailing_padding",)


def test_a_card_without_a_version_fails():
    data = CARD_3.replace(b"VERSION:3.0\r\n", b"")
    assert "VERSION" in vcard.validate(data, frozenset()).detail


def test_an_unclosed_card_fails():
    assert vcard.validate(CARD_3[:-11], frozenset()).status == "fail"


def test_text_after_the_last_card_fails():
    result = vcard.validate(CARD_3 + b"The remote server returned an error.\r\n", frozenset())
    assert result.status == "fail"


def test_a_non_content_line_fails():
    data = CARD_3.replace(b"FN:Ada Lovelace", b"just some words")
    assert "not a content line" in vcard.validate(data, frozenset()).detail


def test_a_nested_card_is_only_accepted_in_version_2_1():
    nested = CARD_3.replace(b"FN:Ada Lovelace\r\n", b"FN:Ada\r\n" + CARD_3)
    assert vcard.validate(nested, frozenset()).status == "fail"
