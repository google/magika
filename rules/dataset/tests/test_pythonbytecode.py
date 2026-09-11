import importlib.util
import marshal
import struct

from helpers import status

from magika_datasets.validators.executable import pythonbytecode


def pyc(
    source="x = [1, 2.5, 'a', b'b', None, (1, 2), {3: 4}, frozenset({5})]\ndef f(a, *, b=1):\n    return a + b\n",
    payload=None,
):
    code = compile(source, "<test>", "exec")
    body = marshal.dumps(code) if payload is None else payload
    return importlib.util.MAGIC_NUMBER + struct.pack("<III", 0, 0, len(source)) + body


def test_current_interpreter_pyc_is_walked_exactly():
    observation = pythonbytecode.validate(pyc(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "pythonbytecode")
    assert observation.tags == ("python_3_12",)
    assert status(pythonbytecode, pyc()[:-5]) == "fail"
    assert status(pythonbytecode, pyc() + b"\0") == "fail"
    assert status(pythonbytecode, pyc(payload=marshal.dumps((1, 2)))) == "fail"
    assert status(pythonbytecode, b"\x00\x00\r\n" + b"\0" * 20) == "not_applicable"
    assert (
        status(pythonbytecode, b"\x00\x10\r\n" + b"\0" * 20) == "not_applicable"
    )  # magic 4096 is nobody's
    assert status(pythonbytecode, b"\xcb\x0d\x0a\x0a" + b"\0" * 20) == "not_applicable"


def test_older_layouts_are_walked_by_version():
    # Python 2.7 style: magic 62211, 4-byte mtime, code with 4 leading ints and 'S' strings.
    ints = struct.pack("<iiii", 0, 0, 1, 64)
    empty_tuple = b"(" + struct.pack("<i", 0)
    code = (
        b"c"
        + ints
        + b"s"
        + struct.pack("<i", 4)
        + b"d\0\0S"
        + empty_tuple * 5
        + b"s"
        + struct.pack("<i", 1)
        + b"m"
        + b"s"
        + struct.pack("<i", 1)
        + b"f"
        + struct.pack("<i", 1)
        + b"s"
        + struct.pack("<i", 0)
    )
    data = struct.pack("<H", 62211) + b"\r\n" + b"\0\0\0\0" + code
    observation = pythonbytecode.validate(data, frozenset())
    assert (observation.status, observation.tags) == ("pass", ("python_2_7",))
    assert status(pythonbytecode, data[:-2]) == "fail"


def test_slice_constants_from_python_313():
    magic = struct.pack("<H", 3571) + b"\r\n"
    consts = b"(" + struct.pack("<i", 1) + b":" + b"i" + struct.pack("<i", 1) + b"N" + b"N"
    empty = b")" + b"\x00"
    code = (
        b"c"
        + struct.pack("<iiiii", 0, 0, 0, 1, 64)
        + b"s"
        + struct.pack("<i", 2)
        + b"\x00\x00"
        + consts
        + empty * 3
        + b"s"
        + struct.pack("<i", 0)
        + b"z\x01m"
        + b"z\x01f"
        + struct.pack("<i", 1)
        + b"s"
        + struct.pack("<i", 0)
        + b"s"
        + struct.pack("<i", 0)
    )
    data = magic + b"\0" * 12 + code
    observation = pythonbytecode.validate(data, frozenset())
    assert (observation.status, observation.tags) == ("pass", ("python_3_13",))
