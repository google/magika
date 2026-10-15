import io
import pickle
import zipfile

from helpers import status

from magika_datasets.validators.archive import zip as zipfamily
from magika_datasets.validators.data import pickle as pickle_validator


class Thing:
    def __init__(self):
        self.value = [1, 2.5, "s", b"b", None, (1,), {2: 3}, {4}]


def test_pickle_streams_are_walked_not_loaded():
    for protocol in (0, 2, 4, 5):
        data = pickle.dumps(Thing(), protocol=protocol)
        observation = pickle_validator.validate(data, frozenset({"pickle"}))
        assert (observation.status, observation.format_id) == ("pass", "pickle"), protocol
        assert f"protocol_{protocol}" in observation.tags and "has_reduce" in observation.tags
    plain = pickle_validator.validate(pickle.dumps([1, 2, 3], protocol=4), frozenset({"pickle"}))
    assert plain.status == "pass" and "has_reduce" not in plain.tags
    data = pickle.dumps(Thing(), protocol=4)
    assert status(pickle_validator, data[:-1], frozenset({"pickle"})) == "fail"
    assert status(pickle_validator, data + b"\0", frozenset({"pickle"})) == "fail"
    assert status(pickle_validator, b"\x80\x09" + b"\0" * 10, frozenset({"pickle"})) == "fail"
    assert (
        status(pickle_validator, b"plain text", frozenset({"pickle"})) == "fail"
    )  # 'p' is PUT; hinted, so reported
    assert status(pickle_validator, b"Zebra", frozenset({"pickle"})) == "not_applicable"
    assert pickle_validator.PREFIX_ONLY and pickle_validator.CONTEXT_REQUIRED


def torch_archive(broken=False):
    stream = io.BytesIO()
    with zipfile.ZipFile(stream, "w") as archive:
        archive.writestr(
            "model/data.pkl",
            pickle.dumps({"weights": "persistent"}, protocol=2)[: -1 if broken else None],
        )
        archive.writestr("model/data/0", b"\0" * 16)
        archive.writestr("model/version", b"3\n")
    return stream.getvalue()


def test_pytorch_archives_are_dispatched_by_the_zip_family():
    observation = zipfamily.validate(torch_archive(), frozenset())
    assert (observation.status, observation.format_id) == ("pass", "pytorch")
    assert zipfamily.validate(torch_archive(broken=True), frozenset()).status == "fail"
