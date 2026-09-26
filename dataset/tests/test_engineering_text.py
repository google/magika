import struct

from helpers import status

from magika_datasets.validators.data import las
from magika_datasets.validators.text import (
    arcgrid,
    diff,
    dxf,
    iges,
    internetshortcut,
    obj,
    pcd,
    step,
    sumfile,
    usda,
    vtk,
)


def test_unified_and_git_diffs():
    unified = b"--- a/x.txt\n+++ b/x.txt\n@@ -1,2 +1,3 @@\n a\n-b\n+c\n+d\n"
    assert diff.validate(unified, frozenset({"diff"})).status == "pass"
    git = b"diff --git a/x b/x\nindex 1..2 100644\n" + unified
    assert diff.validate(git, frozenset({"diff"})).status == "pass"
    assert status(diff, unified.replace(b"+1,3", b"+1,9"), frozenset({"diff"})) == "fail"
    assert status(diff, unified[:-3], frozenset({"diff"})) == "fail"
    assert status(diff, b"prose", frozenset({"diff"})) == "not_applicable"
    assert diff.REQUIRES_HINT


def test_step_sections():
    good = b"ISO-10303-21;\nHEADER;\nFILE_DESCRIPTION(('x'),'2;1');\nENDSEC;\nDATA;\n#1=CARTESIAN_POINT('',(0.,0.,0.));\n#2=DIRECTION('',(0.,0.,1.));\nENDSEC;\nEND-ISO-10303-21;\n"
    assert step.validate(good, frozenset()).format_id == "step"
    assert status(step, good.replace(b"END-ISO-10303-21;", b"")) == "fail"
    assert status(step, good.replace(b"#2=DIRECTION", b"DIRECTION")) == "fail"
    assert status(step, b"ISO-10303-22;") == "not_applicable"


def iges_file(bad=False):
    def record(text, letter, number):
        return text.ljust(72).encode() + f"{letter}{number:7d}".encode() + b"\n"

    lines = (
        record("start", "S", 1)
        + record("1H,,1H;,", "G", 1)
        + record("110 1 0 0 0 0 0 000000000D", "D", 1)
        + record("110,0,0,0,1,1,1;", "P", 1)
    )
    counts = "S      1G      1D      1P      1"
    return lines + record(counts if not bad else "S      9G      1D      1P      1", "T", 1)


def test_iges_records():
    assert iges.validate(iges_file(), frozenset()).format_id == "iges"
    assert status(iges, iges_file(bad=True)) == "fail"
    assert status(iges, iges_file()[:-10]) == "fail"
    assert status(iges, b"start".ljust(72) + b"S      1\n" + b"x" * 30) == "fail"
    assert status(iges, b"nope") == "not_applicable"


def test_dxf_sections():
    good = b"  0\nSECTION\n  2\nHEADER\n  9\n$ACADVER\n  1\nAC1015\n  0\nENDSEC\n  0\nSECTION\n  2\nENTITIES\n  0\nLINE\n  8\n0\n 10\n0.0\n 20\n0.0\n 11\n1.0\n 21\n1.0\n  0\nENDSEC\n  0\nEOF\n"
    assert dxf.validate(good, frozenset()).format_id == "dxf"
    assert status(dxf, good.replace(b"\r", b"").replace(b"\n", b"\r\n")) == "pass"
    assert status(dxf, good.replace(b"  0\nEOF\n", b"")) == "fail"
    assert status(dxf, good.replace(b"  0\nENDSEC\n  0\nSECTION", b"  0\nSECTION")) == "fail"
    assert status(dxf, good + b"junk\n") == "fail"
    assert status(dxf, b"0\nSECTIO") == "not_applicable"
    assert status(dxf, b"hello") == "not_applicable"


def test_obj_and_mtl():
    model = b"# comment\nmtllib m.mtl\nv 0 0 0\nv 1 0 0\nv 0 1 0\nvn 0 0 1\nusemtl a\nf 1//1 2//1 3//1\n"
    assert obj.validate(model, frozenset({"obj"})).format_id == "obj"
    assert status(obj, model.replace(b"f 1//1", b"f x//1"), frozenset({"obj"})) == "fail"
    assert status(obj, b"random words here\n", frozenset({"obj"})) == "fail"
    material = b"newmtl a\nKa 1 1 1\nKd 0.5 0.5 0.5\nmap_Kd t.png\nillum 2\n"
    assert obj.validate(material, frozenset({"mtl"})).format_id == "mtl"
    assert status(obj, material.replace(b"Kd 0.5", b"Kq 0.5"), frozenset({"mtl"})) == "fail"
    assert obj.REQUIRES_HINT


def test_vtk_pcd_usda():
    legacy = b"# vtk DataFile Version 3.0\ntitle\nASCII\nDATASET POLYDATA\nPOINTS 2 float\n0 0 0\n1 1 1\n"
    assert vtk.validate(legacy, frozenset()).format_id == "vtk"
    assert status(vtk, legacy.replace(b"ASCII", b"OTHER")) == "fail"
    fielded = legacy + b"POINT_DATA 2\nFIELD FieldData 1\ntemperature 1 2 float\n1.5 2.5\n"
    assert status(vtk, fielded) == "pass"
    assert status(vtk, legacy + b"garbage words here\n") == "fail"
    binary = (
        b"# vtk DataFile Version 3.0\ntitle\nBINARY\nDATASET POLYDATA\nPOINTS 1 float\n"
        + struct.pack(">fff", 0, 0, 0)
        + b"\n"
    )
    assert status(vtk, binary) == "pass"
    assert status(vtk, binary[:-5]) == "fail"
    cloud = b"# .PCD v0.7\nVERSION 0.7\nFIELDS x y z\nSIZE 4 4 4\nTYPE F F F\nCOUNT 1 1 1\nWIDTH 2\nHEIGHT 1\nVIEWPOINT 0 0 0 1 0 0 0\nPOINTS 2\nDATA ascii\n0 0 0\n1 1 1\n"
    assert pcd.validate(cloud, frozenset()).format_id == "pcd"
    assert status(pcd, cloud + b"2 2 2\n") == "fail"
    packed = cloud.replace(b"DATA ascii\n0 0 0\n1 1 1\n", b"DATA binary\n" + b"\0" * 24)
    assert status(pcd, packed) == "pass"
    assert status(pcd, packed[:-1]) == "fail"
    scene = b'#usda 1.0\n(\n    defaultPrim = "root"\n)\ndef Xform "root" {\n    def Cube "c" { double size = 2 }\n}\n'
    assert usda.validate(scene, frozenset()).format_id == "usd"
    assert status(usda, scene[:-2]) == "fail"
    assert status(usda, b"PXR-USDC" + b"\0" * 40) == "fail"


def test_arcgrid_las_sum_and_shortcut():
    grid = b"ncols 3\nnrows 2\nxllcorner 0\nyllcorner 0\ncellsize 1\nNODATA_value -9999\n1 2 3\n4 5 6\n"
    assert arcgrid.validate(grid, frozenset()).format_id == "arcgrid"
    assert status(arcgrid, grid + b"7\n") == "fail"
    assert status(arcgrid, grid.replace(b"nrows 2", b"nrows 3")) == "fail"
    header = bytearray(227)
    header[:4] = b"LASF"
    struct.pack_into("<BB", header, 24, 1, 2)
    struct.pack_into("<H", header, 94, 227)  # header size
    struct.pack_into("<I", header, 96, 227)  # offset to point data
    struct.pack_into("<I", header, 100, 0)  # number of VLRs
    struct.pack_into("<BHI", header, 104, 0, 20, 3)  # point format 0, 20-byte records, 3 points
    cloud = bytes(header) + b"\0" * 60
    assert las.validate(cloud, frozenset()).format_id == "las"
    assert status(las, cloud[:-1]) == "fail"
    laz = bytearray(header)
    struct.pack_into("<B", laz, 104, 0x83)
    vlr = (
        b"\0\0"
        + b"laszip encoded".ljust(16, b"\0")
        + struct.pack("<HH", 22204, 8)
        + b"\0" * 32
        + b"\0" * 8
    )
    struct.pack_into("<I", laz, 96, 227 + len(vlr))
    struct.pack_into("<I", laz, 100, 1)
    compressed = bytes(laz) + vlr + struct.pack("<q", -1) + b"\x11" * 30
    observation = las.validate(compressed, frozenset())
    assert observation.status == "pass" and "laz" in observation.tags
    assert status(las, b"LASX" + b"\0" * 300) == "not_applicable"
    gosum = b"github.com/x/y v1.0.0 h1:AbCdEfGhIjKlMnOpQrStUvWxYz0123456789abcdefg=\ngithub.com/x/y v1.0.0/go.mod h1:AbCdEfGhIjKlMnOpQrStUvWxYz0123456789abcdefg=\n"
    assert sumfile.validate(gosum, frozenset({"sum"})).format_id == "sum"
    assert status(sumfile, gosum + b"broken line\n", frozenset({"sum"})) == "fail"
    assert sumfile.REQUIRES_HINT
    shortcut = b"[InternetShortcut]\r\nURL=https://example.com/\r\nIconIndex=0\r\n"
    assert internetshortcut.validate(shortcut, frozenset()).format_id == "internetshortcut"
    assert (
        status(
            internetshortcut,
            b"[{000214A0-0000-0000-C000-000000000046}]\r\nProp3=19,2\r\n[InternetShortcut]\r\nURL=http://x/\r\n",
        )
        == "pass"
    )
    assert status(internetshortcut, shortcut.replace(b"URL=", b"URI=")) == "fail"
    assert status(internetshortcut, b"[Desktop Entry]\nType=Link\n") == "not_applicable"


def test_usd_crate_table_of_contents():
    import struct

    from magika_datasets.validators.text import usda

    body = bytes(40)
    sections = [(b"TOKENS", 24, 10), (b"PATHS", 34, 10), (b"SPECS", 44, 20)]
    head = b"PXR-USDC" + bytes([0, 8, 0]) + bytes(5) + struct.pack("<Q", 24 + len(body))
    toc = struct.pack("<Q", len(sections)) + b"".join(
        name.ljust(16, b"\0") + struct.pack("<qq", start, size) for name, start, size in sections
    )
    data = head + body + toc
    result = usda.validate(data, frozenset())
    assert result.status == "pass" and result.tags == ("crate",)
    assert usda.validate(data + b"\0", frozenset()).status == "fail"
