import base64

from magika_datasets.validators.application import pgp


def packet(tag: int, body: bytes) -> bytes:
    return bytes([0x80 | tag << 2, len(body)]) + body


def test_binary_packets_tile_the_file():
    data = packet(2, b"\x04\x00\x01") + bytes([0xC0 | 11, 3]) + b"abc"
    assert pgp.validate(data, frozenset()).status == "pass"
    assert pgp.validate(data[:-1], frozenset({"pgp"})).status == "fail"
    assert pgp.validate(data[:-1], frozenset()) is None
    lone = bytes([0x80 | 4 << 2 | 3]) + b"anything to EOF"  # old format, indeterminate length
    assert pgp.validate(lone, frozenset()) is None
    assert pgp.validate(lone, frozenset({"pgp"})).status == "pass"


def test_armor_with_crc24():
    body = packet(2, b"\x04\x00\x01")
    crc = pgp.crc24(body).to_bytes(3, "big")
    text = (
        b"-----BEGIN PGP SIGNATURE-----\nVersion: test\n\n"
        + base64.b64encode(body)
        + b"\n="
        + base64.b64encode(crc)
        + b"\n-----END PGP SIGNATURE-----\n"
    )
    result = pgp.validate(text, frozenset())
    assert result.status == "pass" and result.tags == ("armored",)
    assert pgp.validate(text.replace(b"\n=", b"\n=A"), frozenset()).status == "fail"
    assert (
        pgp.validate(text.replace(b"END PGP SIGNATURE", b"END PGP MESSAGE"), frozenset()).status
        == "fail"
    )


def test_pem_validator_leaves_pgp_armor_alone():
    from magika_datasets.validators.text import pem

    assert (
        pem.validate(
            b"-----BEGIN PGP SIGNATURE-----\n\nAAAA\n=AAAA\n-----END PGP SIGNATURE-----\n",
            frozenset(),
        )
        is None
    )
