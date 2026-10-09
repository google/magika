import pytest

from magika_datasets.validators._shared import der


def test_element_sizes_and_walk():
    data = b"\x30\x06\x02\x01\x05\x04\x01x"
    assert der.total(data, 0) == 8
    der.walk(data, 0, len(data))
    with pytest.raises(der.Malformed):
        der.walk(data[:-1], 0, len(data) - 1)
    with pytest.raises(der.Malformed):
        der.element(b"\x30\x81\x05", 0)  # non-minimal long form
