from magika_datasets.validators.text import vtk

HEAD = b"# vtk DataFile Version 5.1\ntitle\nASCII\nDATASET POLYDATA\n"


def test_metadata_blocks_with_component_names_pass():
    body = b"POINTS 1 float\n0 0 0\nMETADATA\nINFORMATION 1\nNAME L2_NORM_RANGE LOCATION vtkDataArray\nDATA 2 0 1\nCOMPONENT_NAMES\n MultipoleID\n Other Name\n\nPOINT_DATA 1\n"
    assert vtk.validate(HEAD + body, frozenset()).status == "pass"
    assert vtk.validate(HEAD + b"POINTS 1 float\nnot numbers here\n", frozenset()).status == "fail"
