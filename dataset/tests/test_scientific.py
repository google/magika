import gzip
import struct
import zlib

from helpers import status

from magika_datasets.validators.data import dicom, fits, grib, mat, netcdf, nifti, nrrd


def card(key, value):
    return f"{key:<8}= {value:>20}".ljust(80).encode()


def fits_file(extension=True, trailing=b""):
    header = (
        card("SIMPLE", "T")
        + card("BITPIX", "16")
        + card("NAXIS", "2")
        + card("NAXIS1", "3")
        + card("NAXIS2", "2")
        + b"END".ljust(80)
    )
    header += b" " * (-len(header) % 2880)
    data = b"\0" * 12
    data += b"\0" * (-len(data) % 2880)
    out = header + data
    if extension:
        ext = (
            card("XTENSION", "'BINTABLE'")
            + card("BITPIX", "8")
            + card("NAXIS", "2")
            + card("NAXIS1", "4")
            + card("NAXIS2", "2")
            + card("PCOUNT", "3")
            + card("GCOUNT", "1")
            + b"END".ljust(80)
        )
        ext += b" " * (-len(ext) % 2880)
        body = b"\1" * 11
        body += b"\0" * (-len(body) % 2880)
        out += ext + body
    return out + trailing


def test_fits_header_units_and_data_sizes():
    observation = fits.validate(fits_file(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "fits")
    assert status(fits, fits_file(extension=False)) == "pass"
    assert status(fits, fits_file()[:-2880]) == "fail"
    assert status(fits, fits_file(trailing=b"\0")) == "fail"
    assert status(fits, b"SIMPLE  = F" + b" " * 2869) == "fail"
    assert status(fits, b"SIMPLE  = T" + b" " * 100) == "fail"
    assert status(fits, b"SIMPLE not fits") == "not_applicable"


def netcdf_file(version=1, bad=False):
    def name(text):
        return struct.pack(">I", len(text)) + text + b"\0" * (-len(text) % 4)

    header = b"CDF" + bytes([version]) + struct.pack(">I", 2)  # numrecs
    header += (
        struct.pack(">II", 0x0A, 2)
        + name(b"time")
        + struct.pack(">I", 0)
        + name(b"x")
        + struct.pack(">I", 3)
    )
    header += struct.pack(">II", 0, 0)  # no global attrs
    header += struct.pack(">II", 0x0B, 2)
    values_size = 3 * 4
    offset_fmt = ">I" if version == 1 else ">Q"
    body_start = None
    var1 = (
        name(b"v")
        + struct.pack(">I", 1)
        + struct.pack(">I", 1)
        + struct.pack(">II", 0, 0)
        + struct.pack(">I", 5)
        + struct.pack(">I", values_size)
    )
    var2 = (
        name(b"r")
        + struct.pack(">I", 2)
        + struct.pack(">II", 0, 1)
        + struct.pack(">II", 0, 0)
        + struct.pack(">I", 5)
        + struct.pack(">I", values_size)
    )
    prefix_len = (
        len(header)
        + len(var1)
        + struct.calcsize(offset_fmt)
        + len(var2)
        + struct.calcsize(offset_fmt)
    )
    body_start = prefix_len
    header += (
        var1
        + struct.pack(offset_fmt, body_start)
        + var2
        + struct.pack(offset_fmt, 0xFFFFFF if bad else body_start + values_size)
    )
    return header + b"\0" * values_size + b"\0" * (values_size * 2)


def test_netcdf_classic_headers():
    observation = netcdf.validate(netcdf_file(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "netcdf")
    assert status(netcdf, netcdf_file(version=2)) == "pass"
    assert status(netcdf, netcdf_file(bad=True)) == "fail"
    assert status(netcdf, netcdf_file()[:-20]) == "fail"
    assert status(netcdf, b"\x89HDF\r\n\x1a\n" + b"\0" * 40) == "not_applicable"
    assert status(netcdf, b"CDF\x07" + b"\0" * 40) == "fail"


def mat_file(compressed=False, trailing=b""):
    header = (
        b"MATLAB 5.0 MAT-file, Platform: test".ljust(116)
        + b"\0" * 8
        + struct.pack("<H", 0x0100)
        + b"IM"
    )
    element = struct.pack("<II", 6, 8) + b"\x01" * 8  # miDOUBLE, 8 bytes
    small = struct.pack("<HH", 6, 4) + b"\x02" * 4  # small data element
    if compressed:
        payload = zlib.compress(element)
        element = struct.pack("<II", 15, len(payload)) + payload  # unpadded, as MATLAB writes it
    return header + element + small + trailing


def test_mat_v5_elements():
    observation = mat.validate(mat_file(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "mat")
    assert status(mat, mat_file(compressed=True)) == "pass"
    assert status(mat, mat_file()[:-3]) == "fail"
    assert status(mat, mat_file(trailing=b"\0" * 8)) == "fail"
    assert status(mat, b"MATLAB 7.3 MAT-file".ljust(128) + b"\0" * 100) == "inconclusive"
    assert status(mat, b"MATLAB 5.0".ljust(124) + struct.pack("<H", 0x0100) + b"XX") == "fail"
    assert status(mat, b"other") == "not_applicable"


def element(group, item, vr, value, explicit=True):
    if not explicit:
        return struct.pack("<HHI", group, item, len(value)) + value
    if vr in (b"OB", b"OW", b"SQ", b"UN", b"UT"):
        return (
            struct.pack("<HH", group, item) + vr + b"\0\0" + struct.pack("<I", len(value)) + value
        )
    return struct.pack("<HH", group, item) + vr + struct.pack("<H", len(value)) + value


def dicom_file(syntax=b"1.2.840.10008.1.2.1\0", trailing=b"", undefined=False):
    meta = element(2, 0x0010, b"UI", syntax)
    meta = element(2, 0x0000, b"UL", struct.pack("<I", len(meta))) + meta
    explicit = syntax.startswith(b"1.2.840.10008.1.2.1")
    body = element(0x0008, 0x0016, b"UI", b"1.2.3\0", explicit) + element(
        0x0010, 0x0010, b"PN", b"Doe^J ", explicit
    )
    if undefined:
        item = element(0x0008, 0x0100, b"SH", b"AB", explicit)
        sequence_items = (
            struct.pack("<HHI", 0xFFFE, 0xE000, len(item))
            + item
            + struct.pack("<HHI", 0xFFFE, 0xE00D, 0)
        )
        if explicit:
            body += (
                struct.pack("<HH", 0x0008, 0x1140)
                + b"SQ"
                + b"\0\0"
                + struct.pack("<I", 0xFFFFFFFF)
                + sequence_items
                + struct.pack("<HHI", 0xFFFE, 0xE0DD, 0)
            )
        else:
            body += (
                struct.pack("<HHI", 0x0008, 0x1140, 0xFFFFFFFF)
                + sequence_items
                + struct.pack("<HHI", 0xFFFE, 0xE0DD, 0)
            )
    return b"\0" * 128 + b"DICM" + meta + body + trailing


def test_dicom_meta_and_dataset():
    observation = dicom.validate(dicom_file(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "dicom")
    assert status(dicom, dicom_file(undefined=True)) == "pass"
    assert status(dicom, dicom_file(syntax=b"1.2.840.10008.1.2\0", undefined=True)) == "pass"
    assert status(dicom, dicom_file()[:-3]) == "fail"
    assert status(dicom, dicom_file(trailing=b"\0")) == "fail"
    assert status(dicom, b"\0" * 128 + b"DICX" + b"\0" * 20) == "not_applicable"


def nifti_file(kind=b"n+1\0", bad=False):
    header = bytearray(348)
    struct.pack_into("<i", header, 0, 348)
    struct.pack_into("<8h", header, 40, 3, 2, 3, 1, 1, 1, 1, 1)
    struct.pack_into("<hh", header, 70, 4, 16)  # datatype int16, bitpix 16
    struct.pack_into("<f", header, 108, 352.0 if not bad else 8000.0)
    header[344:348] = kind
    data = b"\0" * (2 * 3 * 2)
    return bytes(header) + b"\0" * 4 + data


def test_nifti_headers():
    observation = nifti.validate(nifti_file(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "nifti")
    assert status(nifti, nifti_file()[:-2]) == "fail"
    assert status(nifti, nifti_file(bad=True)) == "fail"
    assert status(nifti, nifti_file(kind=b"ni1\0")[:348]) == "pass"
    assert status(nifti, b"\0" * 400) == "not_applicable"


def nrrd_file(encoding=b"raw", detached=False, trailing=b""):
    header = b"NRRD0004\ntype: uchar\ndimension: 2\nsizes: 3 2\nencoding: " + encoding + b"\n"
    if detached:
        header += b"data file: other.raw\n"
    data = b"\x07" * 6
    if encoding in (b"gzip", b"gz"):
        data = gzip.compress(data)
    return header + b"\n" + (b"" if detached else data) + trailing


def test_nrrd_headers_and_data_sizes():
    observation = nrrd.validate(nrrd_file(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "nrrd")
    assert status(nrrd, nrrd_file(encoding=b"gzip")) == "pass"
    assert status(nrrd, nrrd_file(trailing=b"\0")) == "fail"
    assert status(nrrd, nrrd_file()[:-1]) == "fail"
    assert status(nrrd, nrrd_file(detached=True)) == "inconclusive"
    assert status(nrrd, b"NRRD0009\nsizes: 1\n\n\0") == "fail"
    assert status(nrrd, b"NRRX") == "not_applicable"


def grib1(trailing=b""):
    pds = struct.pack(">I", 28)[1:] + b"\0" * 25  # length 28, no GDS/BMS flags
    bds = struct.pack(">I", 11)[1:] + b"\0" * 8
    body = pds + bds + b"7777"
    total = 8 + len(body)
    return b"GRIB" + struct.pack(">I", total)[1:] + b"\x01" + body + trailing


def grib2(trailing=b""):
    sections = b"".join(struct.pack(">IB", 5 + 3, n) + b"\0" * 3 for n in (1, 3, 4, 5, 6, 7))
    total = 16 + len(sections) + 4
    return (
        b"GRIB"
        + b"\0\0"
        + b"\0"
        + b"\x02"
        + struct.pack(">Q", total)
        + sections
        + b"7777"
        + trailing
    )


def test_grib_editions():
    for build in (grib1, grib2):
        observation = grib.validate(build(), frozenset())
        assert (observation.status, observation.format_id) == ("pass", "grib"), build.__name__
        assert status(grib, build() + build()) == "pass", build.__name__
        assert status(grib, build()[:-4]) == "fail", build.__name__
        assert status(grib, build(trailing=b"\0" * 8)) == "pass", build.__name__  # NUL padding
        assert status(grib, build(trailing=b"x")) == "fail", build.__name__
    assert status(grib, b"GRIB\0\0\0\x03" + b"\0" * 20) == "fail"
    assert status(grib, b"GRIC") == "not_applicable"
