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

import abc
import io
from pathlib import Path
from typing import BinaryIO


class Seekable(abc.ABC):
    def __init__(self) -> None:
        self._size = -1

    @property
    def size(self) -> int:
        return self._size

    @abc.abstractmethod
    def read_at(self, offset: int, size: int) -> bytes:
        pass

    def close(self) -> None:
        pass


class File(Seekable):
    def __init__(self, path: Path) -> None:
        super().__init__()
        self._f = open(path, "rb")
        self._size = path.stat().st_size

    def read_at(self, offset: int, size: int) -> bytes:
        if size == 0:
            return b""

        assert offset + size <= self.size
        self._f.seek(offset, 0)  # whence = 0: start of the file
        return self._f.read(size)

    def close(self) -> None:
        self._f.close()


class Buffer(Seekable):
    def __init__(self, buffer: bytes) -> None:
        self._buffer = buffer
        self._size = len(buffer)

    def read_at(self, offset: int, size: int) -> bytes:
        if size == 0:
            return b""

        assert offset + size <= self.size
        return self._buffer[offset : offset + size]


class Stream(Seekable):
    def __init__(self, stream: BinaryIO) -> None:
        self._stream = stream
        stream.seek(0, io.SEEK_END)
        self._size = stream.tell()

    def read_at(self, offset: int, size: int) -> bytes:
        if size == 0:
            return b""

        assert offset + size <= self.size
        self._stream.seek(offset)
        return self._stream.read(size)
