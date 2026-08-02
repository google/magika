# Copyright 2024 Google LLC
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

import io
from pathlib import Path

import pytest

from magika import Magika


def test_identify_stream_with_various_binary_streams(tmp_path: Path) -> None:
    test_file = tmp_path / "test.txt"
    test_file.write_bytes(b"Hello world\n")

    m = Magika()

    # 1. RawIOBase stream (e.g. io.FileIO) - issue #1403
    with io.FileIO(str(test_file), "rb") as stream:
        res = m.identify_stream(stream)
        assert res.ok
        assert res.path == Path("-")

    # 2. BufferedIOBase stream (open with 'rb')
    with open(test_file, "rb") as stream:
        res = m.identify_stream(stream)
        assert res.ok
        assert res.path == Path("-")

    # 3. BytesIO stream
    bytes_stream = io.BytesIO(b"Hello world\n")
    res = m.identify_stream(bytes_stream)
    assert res.ok
    assert res.path == Path("-")

    # 4. TextIOBase stream (should raise TypeError)
    with open(test_file, "r") as text_stream:
        with pytest.raises(TypeError, match="bytes mode"):
            m.identify_stream(text_stream)  # type: ignore[arg-type]
