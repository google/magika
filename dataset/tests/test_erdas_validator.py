import struct

from magika_datasets.validators.data import erdas

DICTIONARY = b"{1:lversion,1:LrootEntryPtr,}Ehfa_File,{1:lwidth,}Eimg_Layer,."


def entry(following, previous, parent, child, name, kind, block=0, size=0) -> bytes:
    fields = struct.pack("<6I", following, previous, parent, child, block, size)
    return fields + name.ljust(64, b"\0") + kind.ljust(32, b"\0") + b"\0" * 8


def hfa(layer_type: bytes = b"Eimg_Layer", parent: int | None = None) -> bytes:
    file_record = 20
    dictionary = file_record + 18
    root = dictionary + len(DICTIONARY)
    layer = root + 128
    body = erdas.MAGIC + struct.pack("<I", file_record)
    body += struct.pack("<IIIHI", 1, 0, root, 128, dictionary) + DICTIONARY
    body += entry(0, 0, 0, layer, b"root", b"root")
    body += entry(0, 0, root if parent is None else parent, 0, b"Layer_1", layer_type, 0, 16)
    return body


def test_the_node_tree_is_walked_against_the_dictionary():
    result = erdas.validate(hfa(), frozenset())
    assert result.status == "pass" and "2 nodes of 2 dictionary types" in result.detail


def test_a_node_type_missing_from_the_dictionary_fails():
    result = erdas.validate(hfa(b"Eimg_Unknown"), frozenset())
    assert result.status == "fail" and "Eimg_Unknown" in result.detail


def test_an_inconsistent_parent_link_fails():
    assert erdas.validate(hfa(parent=4), frozenset()).status == "fail"


def test_a_truncated_tree_fails():
    assert erdas.validate(hfa()[:-40], frozenset()).status == "fail"


def test_an_unterminated_dictionary_fails():
    data = hfa().replace(b",.", b",,")
    assert erdas.validate(data, frozenset()).status == "fail"
