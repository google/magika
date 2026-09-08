import pytest
from helpers import status

from magika_datasets.validators.image import svg


def test_svg_identity_not_html_or_wrong_namespace():
    assert status(svg, b'<svg xmlns="http://www.w3.org/2000/svg"/>') == "pass"
    assert status(svg, b'<svg xmlns="http://fake.example"/>') == "not_applicable"
    assert status(svg, b"<html><svg/></html>") == "not_applicable"
    assert status(svg, b'<svg xmlns="http://www.w3.org/2000/svg">') == "fail"


@pytest.mark.parametrize("viewbox", ["0 0 -1 2", "0 0 NaN 2", "0 0 inf 2", "0 0 1", "x y 2 2"])
def test_bad_viewbox(viewbox):
    assert (
        status(svg, f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="{viewbox}"/>'.encode())
        == "fail"
    )


def test_scripts_are_recorded_without_execution_and_entities_refused():
    result = svg.validate(
        b'<svg xmlns="http://www.w3.org/2000/svg"><script>throw "not executed";</script></svg>',
        frozenset(),
    )
    assert result.status == "pass" and "contains_scripts" in result.tags
    assert (
        status(
            svg,
            b'<!DOCTYPE svg [<!ENTITY secret SYSTEM "file:///etc/passwd">]><svg xmlns="http://www.w3.org/2000/svg">&secret;</svg>',
        )
        == "inconclusive"
    )
