import json
import struct

from helpers import status

from magika_datasets.validators.geometry import blend, fbx, gltf, ply, stl, threeds


def blend_file(trailing=b"", bad=False):
    header = b"BLENDER-v300"
    blocks = b""
    for code, payload in ((b"GLOB", b"\0" * 8), (b"DNA1", b"\0" * 16)):
        blocks += (
            code
            + struct.pack("<I", len(payload) if not bad else 9999)
            + struct.pack("<QII", 0, 0, 1)
            + payload
        )
    blocks += b"ENDB" + struct.pack("<I", 0) + struct.pack("<QII", 0, 0, 0)
    return header + blocks + trailing


def test_blend_blocks():
    observation = blend.validate(blend_file(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "blend")
    assert status(blend, blend_file(bad=True)) == "fail"
    assert status(blend, blend_file()[:-4]) == "fail"
    assert status(blend, blend_file(trailing=b"\0")) == "fail"
    assert status(blend, b"BLENDER-x300" + b"\0" * 40) == "fail"
    assert status(blend, b"BLENDEX") == "not_applicable"


def chunk(identifier, payload=b"", children=b""):
    return struct.pack("<HI", identifier, 6 + len(payload) + len(children)) + payload + children


def threeds_file(bad=False):
    mesh = chunk(
        0x4100,
        children=chunk(0x4110, struct.pack("<H", 1) + struct.pack("<fff", 0, 0, 0))
        + chunk(0x4140, struct.pack("<H", 1) + struct.pack("<ff", 0, 0)),
    )
    obj = chunk(0x4000, b"Box\0", children=mesh)
    editor = chunk(0x3D3D, children=chunk(0x3D3E, struct.pack("<I", 3)) + obj)
    data = chunk(0x4D4D, struct.pack("<HI", 0x0002, 10) + struct.pack("<I", 3), children=editor)
    if bad:
        marker = struct.pack("<HI", 0x4110, 6 + 2 + 12)
        data = data.replace(
            marker, struct.pack("<HI", 0x4110, 999), 1
        )  # child claims more than its mesh
    return data


def test_3ds_chunk_tree():
    observation = threeds.validate(threeds_file(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "3dsm")
    assert status(threeds, threeds_file(bad=True)) == "fail"
    assert status(threeds, threeds_file()[:-3], frozenset({"3dsm"})) == "fail"
    assert status(threeds, threeds_file()[:-3]) == "not_applicable"
    assert (
        status(threeds, threeds_file() + b"\0") == "not_applicable"
    )  # length no longer matches: not evidence
    assert status(threeds, b"MM\x00*" + b"\0" * 20) == "not_applicable"  # big-endian TIFF prefix
    assert status(threeds, b"MM" + b"\0" * 10) == "not_applicable"


def ply_binary(vertices=2, faces=1, trailing=b""):
    header = b"ply\nformat binary_little_endian 1.0\ncomment test\n"
    header += f"element vertex {vertices}\nproperty float x\nproperty float y\nproperty float z\n".encode()
    header += f"element face {faces}\nproperty list uchar int vertex_indices\nend_header\n".encode()
    body = struct.pack("<fff", 0, 0, 0) * vertices
    body += b"".join(bytes([3]) + struct.pack("<iii", 0, 1, 0) for _ in range(faces))
    return header + body + trailing


def ply_ascii(vertices=2):
    header = b"ply\nformat ascii 1.0\nelement vertex %d\nproperty float x\nend_header\n" % vertices
    return header + b"".join(b"%d.0\n" % i for i in range(vertices))


def test_ply_binary_and_ascii():
    observation = ply.validate(ply_binary(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "ply")
    assert status(ply, ply_binary()[:-2]) == "fail"
    assert status(ply, ply_binary(trailing=b"\0")) == "fail"
    assert status(ply, ply_ascii()) == "pass"
    assert status(ply, ply_ascii() + b"9.0\n") == "fail"
    assert status(ply, b"ply\nformat weird 1.0\nend_header\n") == "fail"
    assert status(ply, b"plz") == "not_applicable"


def stl_binary(facets=2, trailing=b""):
    return (
        b"binary header".ljust(80, b"\0")
        + struct.pack("<I", facets)
        + (struct.pack("<12fH", *([0.0] * 12), 0) * facets)
        + trailing
    )


def stl_ascii(facets=2, broken=False):
    facet = b"  facet normal -4.6e-001 0 1 \r\n    outer loop \r\n      vertex 0 0 0\r\n      vertex 1 0 0\r\n      vertex 0 1 0\r\n    endloop\r\n  endfacet\r\n"
    body = b"solid cube\n" + facet * facets + (b"endsolid cube\n" if not broken else b"")
    return body


def test_stl_binary_and_ascii():
    observation = stl.validate(stl_binary(), frozenset({"stl"}))
    assert (observation.status, observation.format_id, observation.tags) == (
        "pass",
        "stl",
        ("binary",),
    )
    assert status(stl, stl_binary()[:-2], frozenset({"stl"})) == "fail"
    assert status(stl, stl_binary(trailing=b"\0"), frozenset({"stl"})) == "fail"
    assert stl.validate(stl_ascii(), frozenset()).tags == ("ascii",)
    assert status(stl, stl_ascii(broken=True)) == "fail"
    assert status(stl, b"solid but then prose that is not a facet list\n") == "fail"
    assert status(stl, b"other bytes" * 10, frozenset({"stl"})) == "fail"  # hinted, so reported
    assert status(stl, b"other bytes" * 10) == "not_applicable"


def fbx_node(name, props=b"", children=b"", start=0, version=7400):
    wide = version >= 7500
    header_fmt = "<QQQ" if wide else "<III"
    null_record = b"\0" * (25 if wide else 13)
    header_size = struct.calcsize(header_fmt) + 1 + len(name)
    body = props + children + (null_record if children else b"")
    end = start + header_size + len(body)
    return struct.pack(header_fmt, end, 0, len(props)) + bytes([len(name)]) + name + body


def fbx_file(version=7400, bad=False):
    magic = b"Kaydara FBX Binary  \0\x1a\0" + struct.pack("<I", version)
    offset = len(magic)
    child = fbx_node(
        b"Version",
        props=b"I" + struct.pack("<i", 1000),
        start=offset
        + (struct.calcsize("<QQQ") if version >= 7500 else 12)
        + 1
        + len(b"FBXHeaderExtension"),
        version=version,
    )
    top = fbx_node(b"FBXHeaderExtension", children=child, start=offset, version=version)
    if bad:
        top = struct.pack("<I", 999999) + top[4:]
    null_record = b"\0" * (25 if version >= 7500 else 13)
    footer = (
        b"\xfa\xbc\xab\x09\xd0\xc8\xd4\x66\xb1\x76\xfb\x83\x1c\xf7\x26\x7e"
        + b"\0" * 4
        + struct.pack("<I", version)
        + b"\0" * 120
        + b"\xf8\x5a\x8c\x6a\xde\xf5\xd9\x7e\xec\xe9\x0c\xe3\x75\x8f\x29\x0b"
    )
    return magic + top + null_record + footer


def test_fbx_binary_nodes():
    observation = fbx.validate(fbx_file(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "fbx")
    assert fbx.validate(fbx_file(version=7500), frozenset()).status == "pass"
    assert status(fbx, fbx_file(bad=True)) == "fail"
    assert status(fbx, fbx_file()[:-200]) == "fail"  # into the node records
    assert status(fbx, fbx_file()[:-1]) == "fail"  # footer magic clipped
    old = fbx_file()
    assert status(fbx, old[:-16] + b"\0" * 800 + old[-16:]) == "pass"  # FBX 6.x style long footer
    ascii = b"; FBX 7.4.0 project file\nFBXHeaderExtension:  {\n\tFBXHeaderVersion: 1003\n}\n"
    assert status(fbx, ascii, frozenset({"fbx"})) == "pass"
    assert status(fbx, ascii[:-2], frozenset({"fbx"})) == "fail"
    assert status(fbx, b"Kaydara FBX Binary  \0\x1a\0") == "fail"
    assert status(fbx, b"other") == "not_applicable"


def glb(bad=False, trailing=b""):
    payload = json.dumps({"asset": {"version": "2.0"}}).encode()
    payload += b" " * (-len(payload) % 4)
    binary = b"\0" * 8
    chunks = (
        struct.pack("<II", len(payload), 0x4E4F534A)
        + payload
        + struct.pack("<II", len(binary), 0x004E4942)
        + binary
    )
    total = 12 + len(chunks) + len(trailing)
    if bad:
        chunks = struct.pack("<II", 9999, 0x4E4F534A) + chunks[8:]
    return b"glTF" + struct.pack("<II", 2, total - len(trailing)) + chunks + trailing


def test_glb_chunks():
    observation = gltf.validate(glb(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "gltf")
    assert status(gltf, glb(bad=True)) == "fail"
    assert status(gltf, glb()[:-3]) == "fail"
    assert status(gltf, glb(trailing=b"\0\0\0\0")) == "fail"
    assert status(gltf, b"glTF" + struct.pack("<II", 1, 20) + b"\0" * 8) == "fail"
    assert status(gltf, b"glTX") == "not_applicable"
