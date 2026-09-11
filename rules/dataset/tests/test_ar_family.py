from magika_datasets.validators.archive import ar


def member(name: bytes, body: bytes) -> bytes:
    header = name.ljust(16) + b"0".ljust(12) + b"0".ljust(6) + b"0".ljust(6) + b"644".ljust(8)
    header += str(len(body)).encode().ljust(10) + b"`\n"
    return header + body + (b"\n" if len(body) & 1 else b"")


def archive(*members: bytes) -> bytes:
    return b"!<arch>\n" + b"".join(members)


def test_plain_archive_passes_as_generic_ar():
    result = ar.validate(archive(member(b"a.o", b"abc"), member(b"b.o", b"defg")), frozenset())
    assert result.status == "pass" and result.format_id == "ar" and result.generic


def test_debian_package_is_named_deb():
    data = archive(
        member(b"debian-binary", b"2.0\n"),
        member(b"control.tar.xz", b"x" * 10),
        member(b"data.tar.gz", b"y" * 11),
    )
    result = ar.validate(data, frozenset())
    assert result.status == "pass" and result.format_id == "deb" and not result.generic


def test_debian_package_without_data_member_fails():
    data = archive(member(b"debian-binary", b"2.0\n"), member(b"control.tar.gz", b"x"))
    assert ar.validate(data, frozenset()).format_id == "deb"
    assert ar.validate(data, frozenset()).status == "fail"


def test_bsd_long_name_is_read_from_the_data():
    data = archive(member(b"#1/8", b"long.o\0\0" + b"body"))
    assert ar.validate(data, frozenset()).status == "pass"


def test_truncated_member_fails():
    data = archive(member(b"a.o", b"abcdef"))[:-3]
    assert ar.validate(data, frozenset()).status == "fail"


def test_bad_terminator_fails():
    data = bytearray(archive(member(b"a.o", b"ab")))
    data[8 + 58] = ord("x")
    assert ar.validate(bytes(data), frozenset()).status == "fail"
