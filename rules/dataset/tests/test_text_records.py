import base64

from helpers import status

from magika_datasets.validators.text import ics, m3u, pem, po, srt, vtt


def test_icalendar_components():
    good = b"BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:1\r\nSUMMARY:Long\r\n  folded\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n"
    assert (ics.validate(good, frozenset()).status, ics.validate(good, frozenset()).format_id) == (
        "pass",
        "ics",
    )
    assert status(ics, good.replace(b"END:VEVENT\r\n", b"")) == "fail"
    assert status(ics, good + b"junk line\r\n") == "fail"
    assert status(ics, b"BEGIN:VCARD\r\nEND:VCARD\r\n") == "not_applicable"
    assert status(ics, b"BEGIN:VCALENDAR\r\nnoline\r\nEND:VCALENDAR\r\n") == "fail"


def pem_block(label=b"CERTIFICATE", body=b"\x30\x03\x02\x01\x01" * 10, end=None):
    encoded = base64.encodebytes(body).replace(b"\n", b"\r\n")
    return (
        b"-----BEGIN "
        + label
        + b"-----\r\n"
        + encoded
        + b"-----END "
        + (end or label)
        + b"-----\r\n"
    )


def test_pem_blocks():
    observation = pem.validate(pem_block(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "pem")
    two = pem_block() + b"# comment\r\n\r\n" + pem_block(b"PRIVATE KEY")
    assert status(pem, two) == "pass"
    assert status(pem, pem_block(end=b"KEY")) == "fail"
    assert status(pem, pem_block().replace(b"MAMC", b"M*MC")) == "fail"
    outside = pem.validate(b"Certificate:\n    Data:\n" + pem_block(), frozenset())
    assert outside.status == "pass" and "text_outside_blocks" in outside.tags
    encrypted = (
        b"-----BEGIN RSA PRIVATE KEY-----\nProc-Type: 4,ENCRYPTED\nDEK-Info: DES-EDE3-CBC,9CF7E869357B3BD6\n\n"
        + base64.encodebytes(b"x" * 48)
        + b"-----END RSA PRIVATE KEY-----\n"
    )
    assert status(pem, encrypted) == "pass"
    assert status(pem, b"-----BEGIN X-----\r\n") == "fail"
    assert status(pem, b"no pem here") == "not_applicable"
    certificate = b"\x30\x82\x00\x0a\x30\x03\x02\x01\x01\x30\x00\x03\x01\x00"
    der = b"\x30" + bytes([len(certificate)]) + certificate
    assert pem.validate(der, frozenset({"crt"})).format_id == "crt"
    assert status(pem, der[:-1], frozenset({"crt"})) == "fail"
    assert pem.validate(pem_block(), frozenset({"crt"})).format_id == "crt"


def test_subtitles_and_playlists():
    cues = b"1\n00:00:01,000 --> 00:00:02,500\nHello\n\n2\n00:00:03,000 --> 00:00:04,000\nWorld\nagain\n"
    assert srt.validate(cues, frozenset()).format_id == "srt"
    assert status(srt, b"\xef\xbb\xbf" + cues) == "pass"
    assert status(srt, cues.replace(b"-->", b"->")) == "not_applicable"  # no arrow, no evidence
    assert status(srt, cues.replace(b"00:00:03,000 --> 00:00:04,000", b"bad timing")) == "fail"
    assert (
        status(srt, b"2\n00:00:01,000 --> 00:00:02,500\nx\n") == "pass"
    )  # fragments may start anywhere
    assert status(srt, cues.replace(b"\n2\n", b"\n1\n")) == "fail"  # indices must ascend
    assert status(srt, b"prose") == "not_applicable"
    web = b"WEBVTT\n\n00:01.000 --> 00:02.000\nHi\n\nNOTE comment\n\n1\n00:03.000 --> 00:04.000 line:0\nBye\n"
    assert vtt.validate(web, frozenset()).format_id == "vtt"
    assert status(vtt, web.replace(b"00:03.000 --> 00:04.000", b"garbage")) == "fail"
    assert status(vtt, b"WEBVTX\n") == "not_applicable"
    playlist = (
        b"#EXTM3U\r\n#EXTINF:123,Artist - Title\r\nsong.mp3\r\n#EXTINF:-1,Stream\r\nhttp://x/y\r\n"
    )
    assert m3u.validate(playlist, frozenset()).format_id == "m3u"
    assert status(m3u, playlist + b"#EXTINF:1,dangling\r\n") == "fail"
    assert (
        status(
            m3u,
            b"\xef\xbb\xbf#EXTM3U\n#EXT-X-VERSION:3\n#EXT-X-TARGETDURATION:10\n#EXTINF:9.0,\nseg.ts\n#EXT-X-ENDLIST\n",
        )
        == "pass"
    )
    assert status(m3u, b"#EXTM4U") == "not_applicable"


def test_gettext_catalogs():
    catalog = b'# Translations\nmsgid ""\nmsgstr ""\n"Content-Type: text/plain\\n"\n\n#: file.c:1\nmsgctxt "ctx"\nmsgid "Hello"\nmsgstr "Bonjour"\n\nmsgid "One"\nmsgid_plural "Many"\nmsgstr[0] "Un"\nmsgstr[1] "Des"\n'
    assert po.REQUIRES_HINT
    observation = po.validate(catalog, frozenset({"po"}))
    assert (observation.status, observation.format_id) == ("pass", "po")
    assert status(po, catalog + b'msgid "x"\n', frozenset({"po"})) == "fail"
    assert (
        status(po, catalog.replace(b'msgstr "Bonjour"', b"msgstr Bonjour"), frozenset({"po"}))
        == "fail"
    )
    assert status(po, b"# only comments\nmsgid\n", frozenset({"po"})) == "fail"
    assert status(po, b"not a catalog", frozenset({"po"})) == "not_applicable"
