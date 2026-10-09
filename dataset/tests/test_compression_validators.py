import gzip as gzip_module

from helpers import status

from magika_datasets.validators.archive import gzip


def test_gzip_members_crc_and_trailing():
    one = gzip_module.compress(b"hello" * 100, mtime=0)
    assert status(gzip, one) == "pass"
    assert status(gzip, one + one) == "pass"
    assert status(gzip, one[:-1]) == "fail"
    assert status(gzip, one + b"trailing") == "fail"
    corrupt = one[:-8] + b"\x00\x00\x00\x00" + one[-4:]
    assert status(gzip, corrupt) == "fail"
    assert status(gzip, b"\x1f\x8b\x08") == "fail"
    assert status(gzip, b"not gzip") == "not_applicable"


def test_gzip_bomb_is_inconclusive():
    assert status(gzip, gzip_module.compress(b"\x00" * (65 * 1024 * 1024))) == "inconclusive"


def test_bzip2_streams_and_corruption():
    import bz2

    from magika_datasets.validators.archive import bzip

    one = bz2.compress(b"data" * 200)
    assert status(bzip, one) == "pass"
    assert status(bzip, one + one) == "pass"
    assert status(bzip, one[:-1]) == "fail"
    assert status(bzip, one + b"x") == "fail"
    assert status(bzip, one[:-4] + b"\x00\x00\x00\x00") == "fail"
    assert status(bzip, bz2.compress(b"")) == "pass"
    assert status(bzip, b"BZh9 not really") == "not_applicable"


def test_xz_streams_padding_and_corruption():
    import lzma

    from magika_datasets.validators.archive import xz

    one = lzma.compress(b"data" * 200, format=lzma.FORMAT_XZ)
    assert status(xz, one) == "pass"
    assert status(xz, one + b"\x00" * 8 + one) == "pass"
    assert status(xz, one + b"\x00" * 3) == "fail"
    assert status(xz, one[:-1]) == "fail"
    assert status(xz, one[:-4] + b"\x00\x00" + one[-2:]) == "fail"  # stream flags
    assert status(xz, lzma.compress(b"x", format=lzma.FORMAT_ALONE)) == "not_applicable"


def test_zlib_stream_header_adler_and_single_stream():
    import zlib

    from magika_datasets.validators.archive import zlibstream

    one = zlib.compress(b"payload" * 50)
    assert status(zlibstream, one) == "pass"
    assert status(zlibstream, one + one) == "fail"
    assert status(zlibstream, one[:-1]) == "fail"
    assert status(zlibstream, one[:-4] + b"\x00\x00\x00\x00") == "fail"
    assert status(zlibstream, b"\x78\x9d" + one[2:]) == "not_applicable"
    assert status(zlibstream, b"xZ") == "not_applicable"
    assert status(zlibstream, b"\x78\x20" + b"\x00" * 8) == "inconclusive"  # FDICT set


def test_compressed_streams_are_generic_containers():
    import bz2
    import lzma
    import zlib

    from magika_datasets.validators.archive import bzip, xz, zlibstream

    for module, data in [
        (gzip, gzip_module.compress(b"x")),
        (bzip, bz2.compress(b"x")),
        (xz, lzma.compress(b"x", format=lzma.FORMAT_XZ)),
        (zlibstream, zlib.compress(b"x")),
    ]:
        observation = module.validate(data, frozenset())
        assert observation.status == "pass" and observation.generic, module.__name__
