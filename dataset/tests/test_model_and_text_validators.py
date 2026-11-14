# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

import base64
import json
import struct

from helpers import status

from magika_datasets.validators.data import berkeleydb
from magika_datasets.validators.text import json_family, model_text, plaintext, ssh_key, xml_family


def kind(module, data, hints=frozenset()):
    result = module.validate(data, hints)
    return None if result is None else (result.status, result.format_id)


def test_tensorflowjs_manifest_is_proven_and_malformed_weights_fail():
    model = {
        "format": "layers-model",
        "modelTopology": {},
        "weightsManifest": [
            {
                "paths": ["group1.bin"],
                "weights": [{"name": "w", "shape": [2, 3], "dtype": "float32"}],
            }
        ],
    }
    assert kind(json_family, json.dumps(model).encode()) == ("pass", "tensorflowjs")
    model["weightsManifest"][0]["weights"][0]["shape"] = ["x"]
    assert kind(json_family, json.dumps(model).encode()) == ("fail", "tensorflowjs")
    assert kind(json_family, b'{"name": "app", "version": "1.0"}') == ("pass", "json")


def test_xgboost_json_model():
    model = {
        "version": [2, 0, 3],
        "learner": {
            "gradient_booster": {"name": "gbtree"},
            "objective": {"name": "binary:logistic"},
        },
    }
    assert kind(json_family, json.dumps(model).encode()) == ("pass", "xgboost")
    model["version"] = "2.0"
    assert kind(json_family, json.dumps(model).encode()) == ("fail", "xgboost")


def test_lightgbm_tree_count_must_match_its_sizes():
    body = b"tree\nversion=v3\nnum_class=1\ntree_sizes=10 12\n\nTree=0\nx\n\nTree=1\ny\n\nend of trees\n"
    assert kind(model_text, body) == ("pass", "lightgbm")
    assert kind(model_text, body.replace(b"tree_sizes=10 12", b"tree_sizes=10")) == (
        "fail",
        "lightgbm",
    )
    assert kind(model_text, b"tree of life\n") is None


def test_ncnn_param_counts_are_checked():
    good = b"7767517\n2 2\nInput data 0 1 data\nReLU relu 1 1 data out\n"
    assert kind(model_text, good) == ("pass", "ncnn")
    assert kind(model_text, good.replace(b"2 2\n", b"3 2\n")) == ("fail", "ncnn")


def test_openvino_edges_must_reference_declared_layers():
    good = b'<net name="m" version="11"><layers><layer id="0" type="Parameter"/><layer id="1" type="Result"/></layers><edges><edge from-layer="0" from-port="0" to-layer="1" to-port="0"/></edges></net>'
    assert kind(xml_family, good) == ("pass", "openvino_ir")
    assert kind(xml_family, good.replace(b'to-layer="1"', b'to-layer="9"')) == (
        "fail",
        "openvino_ir",
    )


def test_plaintext_names_the_encoding_and_needs_a_hint():
    assert plaintext.REQUIRES_HINT
    assert kind(plaintext, b"hello\nworld\n") == ("pass", "txtascii")
    assert kind(plaintext, "café\n".encode()) == ("pass", "txtutf8")
    assert kind(plaintext, "﻿hi".encode("utf-16")) == ("pass", "txtutf16")
    assert status(plaintext, b"bin\x00ary") == "fail"
    assert status(plaintext, b"\xff\xfe\x00") == "fail"


def test_ssh_public_key_blob_must_encode_its_type():
    blob = struct.pack(">I", 11) + b"ssh-ed25519" + struct.pack(">I", 32) + b"\x01" * 32
    line = b"ssh-ed25519 " + base64.b64encode(blob) + b" me@host\n"
    assert kind(ssh_key, line) == ("pass", "pub")
    assert kind(ssh_key, line.replace(b"ssh-ed25519 ", b"ssh-rsa ", 1)) == ("fail", "pub")
    assert kind(ssh_key, b"name: my_package\n") is None


def btree(pages=3, pagesize=4096, order="<"):
    page = bytearray(pagesize)
    struct.pack_into(order + "I", page, 12, 0x00053162)
    struct.pack_into(order + "I", page, 20, pagesize)
    page[25] = 9
    struct.pack_into(order + "I", page, 32, pages - 1)
    return bytes(page) + b"\0" * pagesize * (pages - 1)


def test_berkeleydb_is_bounded_by_its_last_page_in_either_byte_order():
    assert kind(berkeleydb, btree()) == ("pass", "berkeleydb")
    assert kind(berkeleydb, btree(order=">")) == ("pass", "berkeleydb")
    assert kind(berkeleydb, btree() + b"\0") == ("fail", "berkeleydb")
    assert kind(berkeleydb, b"SQLite format 3\x00" + b"\0" * 1000) is None


def test_geoparquet_needs_geo_metadata_naming_a_real_column():
    import io

    import pyarrow as pa
    import pyarrow.parquet as pq

    from magika_datasets.validators.data import columnar

    def parquet(geo):
        table = pa.table({"geometry": [b"\x01"], "name": ["a"]})
        if geo is not None:
            table = table.replace_schema_metadata({b"geo": json.dumps(geo).encode()})
        buffer = io.BytesIO()
        pq.write_table(table, buffer)
        return buffer.getvalue()

    good = {"primary_column": "geometry", "columns": {"geometry": {"encoding": "WKB"}}}
    assert kind(columnar, parquet(good)) == ("pass", "geoparquet")
    assert kind(columnar, parquet(None)) == ("pass", "parquet")
    assert kind(columnar, parquet({"primary_column": "nope", "columns": {}})) == ("pass", "parquet")
