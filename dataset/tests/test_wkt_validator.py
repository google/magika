from magika_datasets.validators.geometry import wkt


def status(text):
    result = wkt.validate(text.encode(), frozenset())
    return result.status if result else None


def test_simple_features_parse():
    text = """POINT (1 2)
LINESTRING (0 0, 1 1, 2 2)
POLYGON ((0 0, 4 0, 4 4, 0 0), (1 1, 2 1, 2 2, 1 1))
MULTIPOINT ((1 2), (3 4))
MULTIPOINT (1 2, 3 4)
MULTIPOLYGON (((0 0, 1 0, 1 1, 0 0)), EMPTY)
GEOMETRYCOLLECTION (POINT Z (1 2 3), LINESTRING EMPTY)
SRID=4326;POINT M (1 2 3)
"""
    result = wkt.validate(text.encode(), frozenset())
    assert result.status == "pass" and result.detail.startswith("8 geometries")


def test_mixed_ordinate_counts_fail():
    assert status("LINESTRING (0 0, 1 1 1)") == "fail"


def test_a_declared_dimension_is_enforced():
    assert status("POINT Z (1 2)") == "fail"


def test_unbalanced_parentheses_fail():
    assert status("POLYGON ((0 0, 1 1, 0 0)") == "fail"


def test_trailing_prose_fails():
    assert status("POINT (1 2)\nsee the map for details") == "fail"


def test_prose_starting_with_a_type_word_is_not_claimed():
    assert status("Point of order: nothing here") is None
