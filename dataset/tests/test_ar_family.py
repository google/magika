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


def rmeta(body: bytes = b"rust\0\0\0\x0a" + b"\0" * 8) -> bytes:
    return b"\x7fELF\x02\x01\x01\0" + b"\0" * 24 + body  # metadata wrapped in an object file


def test_rust_library_is_named_rlib():
    data = archive(
        member(b"/", b"\0\0\0\0"), member(b"foo.o/", b"obj"), member(b"lib.rmeta/", rmeta())
    )
    result = ar.validate(data, frozenset())
    assert result.status == "pass" and result.format_id == "rlib" and not result.generic
    assert "metadata version 10" in result.detail


def test_rust_library_without_metadata_magic_fails():
    data = archive(member(b"lib.rmeta/", rmeta(b"not rust metadata")))
    result = ar.validate(data, frozenset())
    assert result.format_id == "rlib" and result.status == "fail"


DESCRIPTOR = (
    b'<vib version="5.0"><type>bootbank</type><name>ne1000</name><version>0.9.2</version></vib>'
)


def test_vmware_installation_bundle_is_named_vib():
    data = archive(
        member(b"descriptor.xml", DESCRIPTOR), member(b"sig.pkcs7", b"sig"), member(b"ne1000", b"x")
    )
    result = ar.validate(data, frozenset())
    assert (result.status, result.format_id) == ("pass", "vib") and "ne1000" in result.detail


def test_a_vib_descriptor_that_is_not_a_vib_fails():
    data = archive(member(b"descriptor.xml", b"<other/>"), member(b"payload", b"x"))
    result = ar.validate(data, frozenset())
    assert (result.status, result.format_id) == ("fail", "vib")
