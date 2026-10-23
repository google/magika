from magika_datasets.validators.text import proteindb

PDB = (
    b"HEADER    HYDROLASE                               01-JAN-00   1ABC\n"
    b"TITLE     TEST STRUCTURE\n"
    b"ATOM      1  N   MET A   1      10.000  20.000  30.000  1.00 20.00           N\n"
    b"HETATM    2  O   HOH A 101       1.500  -2.250   3.000  1.00 30.00           O\n"
    b"TER       3      MET A   1\n"
    b"END\n"
)


def test_records_pass():
    result = proteindb.validate(PDB, frozenset())
    assert result.status == "pass" and "2 atom records" in result.detail


def test_unknown_record_and_bad_coordinates_fail():
    assert proteindb.validate(PDB + b"BOGUS record\n", frozenset()).status == "fail"
    broken = PDB.replace(b"10.000  20.000", b"10.000  2x.000")
    assert proteindb.validate(broken, frozenset()).status == "fail"
