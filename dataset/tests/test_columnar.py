import io

import pyarrow as pa
import pyarrow.orc
import pyarrow.parquet as pq
from helpers import status

from magika_datasets.validators.data import columnar


def table():
    return pa.table({"a": pa.array([1, 2, 3]), "b": pa.array(["x", "y", "z"])})


def parquet():
    out = io.BytesIO()
    pq.write_table(table(), out, compression="snappy")
    return out.getvalue()


def orc():
    out = io.BytesIO()
    pa.orc.write_table(table(), out)
    return out.getvalue()


def arrow():
    out = io.BytesIO()
    with pa.ipc.new_file(out, table().schema) as writer:
        writer.write_table(table())
    return out.getvalue()


def test_parquet_orc_and_arrow_files():
    for build, kind in ((parquet, "parquet"), (orc, "orc"), (arrow, "arrow")):
        data = build()
        observation = columnar.validate(data, frozenset())
        assert (observation.status, observation.format_id) == ("pass", kind), kind
        assert status(columnar, data[: len(data) - 12], frozenset({kind})) == "fail", kind
        assert status(columnar, data + b"\0", frozenset({kind})) == "fail", kind
    assert "snappy" in columnar.validate(parquet(), frozenset()).tags
    assert status(columnar, b"PAR1" + b"\0" * 20 + b"PAR1") == "fail"
    assert status(columnar, b"other") == "not_applicable"
    assert status(columnar, b"ORCHESTRATION notes\n") == "not_applicable"
    assert status(columnar, b"ORCHESTRATION notes\n", frozenset({"orc"})) == "fail"
