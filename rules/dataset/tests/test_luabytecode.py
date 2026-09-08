import struct

from helpers import status

from magika_datasets.validators.executable import luabytecode


def lua51(trailing=b"", big=False):
    o = ">" if big else "<"
    header = b"\x1bLua\x51\x00" + (b"\x00" if big else b"\x01") + b"\x04\x08\x04\x08\x00"
    source = struct.pack(o + "Q", 6) + b"=stdin"
    func = source + struct.pack(o + "iiBBBB", 0, 0, 0, 0, 2, 2)
    func += struct.pack(o + "i", 2) + b"\x01\x00\x00\x00" + b"\x1e\x00\x80\x00"  # two instructions
    func += (
        struct.pack(o + "i", 3)
        + b"\x03"
        + struct.pack(o + "d", 1.5)
        + b"\x04"
        + struct.pack(o + "Q", 3)
        + b"hi\0"
        + b"\x01\x01"
    )  # constants
    func += struct.pack(o + "i", 0)  # protos
    func += struct.pack(o + "i", 2) + struct.pack(o + "ii", 1, 1)  # line info
    func += (
        struct.pack(o + "i", 1) + struct.pack(o + "Q", 2) + b"a\0" + struct.pack(o + "ii", 0, 2)
    )  # locvars
    func += struct.pack(o + "i", 0)  # upvalues
    return header + func + trailing


def varint(value):
    out = []
    while True:
        out.append(value & 0x7F)
        value >>= 7
        if not value:
            break
    out.reverse()
    return bytes(out[:-1]) + bytes([out[-1] | 0x80])


def lua54(trailing=b""):
    header = (
        b"\x1bLua\x54\x00\x19\x93\r\n\x1a\n"
        + bytes([4, 8, 8])
        + struct.pack("<q", 0x5678)
        + struct.pack("<d", 370.5)
    )
    header += b"\x01"  # upvalue count of main
    func = varint(7) + b"=stdin"  # source: size+1
    func += (
        varint(0) + varint(0) + bytes([0, 1, 2])
    )  # linedefined, lastlinedefined, numparams, is_vararg, maxstacksize
    func += varint(2) + b"\x51\x00\x00\x00" + b"\x46\x00\x01\x00"  # code
    func += (
        varint(2) + b"\x13" + struct.pack("<q", 7) + b"\x04" + varint(3) + b"hi"
    )  # constants: int, short string
    func += varint(1) + bytes([1, 0, 0])  # upvalues
    func += varint(0)  # protos
    func += varint(2) + b"\x00\x00"  # lineinfo
    func += varint(0)  # abslineinfo
    func += varint(0)  # locvars
    func += varint(1) + varint(5) + b"_ENV"  # upvalue names
    return header + func + trailing


def test_lua_51_and_54_chunks():
    old = luabytecode.validate(lua51(), frozenset())
    assert (old.status, old.format_id, old.tags) == ("pass", "luabytecode", ("lua_5_1",))
    new = luabytecode.validate(lua54(), frozenset())
    assert (new.status, new.tags) == ("pass", ("lua_5_4",))
    assert status(luabytecode, lua51(big=True)) == "pass"
    assert status(luabytecode, lua51()[:-3]) == "fail"
    assert status(luabytecode, lua54()[:-1]) == "fail"
    assert status(luabytecode, lua51(trailing=b"\0")) == "fail"
    assert status(luabytecode, lua54(trailing=b"\0")) == "fail"
    assert status(luabytecode, b"\x1bLua\x50" + b"\0" * 20) == "fail"
    assert status(luabytecode, b"\x1bLux" + b"\0" * 20) == "not_applicable"


def lua52():
    header = b"\x1bLua\x52\x00\x01\x04\x08\x04\x08\x00" + b"\x19\x93\r\n\x1a\n"
    func = struct.pack("<iiBBB", 0, 0, 0, 1, 2)
    func += struct.pack("<i", 1) + b"\x1f\x00\x80\x00"  # code
    func += (
        struct.pack("<i", 2) + b"\x01\x01" + b"\x04" + struct.pack("<Q", 3) + b"hi\0"
    )  # bool + string
    func += struct.pack("<i", 0)  # protos
    func += struct.pack("<i", 1) + b"\x01\x00"  # upvalues
    func += struct.pack("<Q", 6) + b"=stdin"  # source
    func += struct.pack("<i", 1) + struct.pack("<i", 1)  # lineinfo
    func += (
        struct.pack("<i", 0) + struct.pack("<i", 1) + struct.pack("<Q", 5) + b"_ENV\0"
    )  # locvars, upvalue names
    return header + func


def luajit(stripped=False, trailing=b""):
    flags = b"\x02" if stripped else b"\x00"
    name = b"" if stripped else b"\x06" + b"=stdin"
    proto = b"\x10" + b"\x00" * 16
    return b"\x1bLJ\x02" + flags + name + proto + b"\x00" + trailing


def test_lua_52_and_luajit():
    observation = luabytecode.validate(lua52(), frozenset())
    assert (observation.status, observation.tags) == ("pass", ("lua_5_2",))
    assert status(luabytecode, lua52()[:-1]) == "fail"
    jit = luabytecode.validate(luajit(), frozenset())
    assert (jit.status, jit.tags) == ("pass", ("luajit_v2",))
    assert status(luabytecode, luajit(stripped=True)) == "pass"
    assert status(luabytecode, luajit()[:-1]) == "fail"
    assert status(luabytecode, luajit(trailing=b"\0")) == "fail"
