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

"""Magika (the Python library).

This module provides the `Magika` class, the main entry point for using Magika
to identify file content types, backed by the PyO3 native Rust extension.
"""

from __future__ import annotations

import io
import json
import logging
import os
from pathlib import Path
from typing import Any, BinaryIO, Dict, List, Optional, Sequence, Set, Tuple, Union

from magika.logger import get_logger
from magika.types import (
    ContentTypeInfo,
    ContentTypeLabel,
    MagikaError,
    MagikaPrediction,
    MagikaResult,
    OverwriteReason,
    PredictionMode,
    Status,
)

from . import _magika


class Magika:
    """Main Magika class for content type identification.

    This class provides methods to identify the content type of files, bytes,
    and streams by delegating to the native Rust core via PyO3.
    """

    def __init__(
        self,
        model_dir: Optional[Path] = None,
        prediction_mode: PredictionMode = PredictionMode.HIGH_CONFIDENCE,
        no_dereference: bool = False,
        verbose: bool = False,
        debug: bool = False,
        use_colors: bool = False,
    ) -> None:
        """Initializes the Magika instance.

        Args:
            model_dir: Path to the directory containing the model and its
                configuration. If None, the default model is used.
            prediction_mode: The prediction mode to use. Defaults to
                PredictionMode.HIGH_CONFIDENCE.
            no_dereference: If True, do not follow symlinks. Defaults to False.
            verbose: If True, enable verbose logging. Defaults to False.
            debug: If True, enable debug logging. Defaults to False.
            use_colors: If True, use colors in the logger. Defaults to False.
        """
        self._log = get_logger(use_colors=use_colors)

        if verbose:
            self._log.setLevel(logging.INFO)

        if debug:
            self._log.setLevel(logging.DEBUG)

        if model_dir is not None:
            raise NotImplementedError(
                f"Custom model_dir '{model_dir}' is not supported by the Rust backend."
            )

        self._model_config_path = (
            Path(__file__).parent / "config" / "model_config.min.json"
        )
        if not self._model_config_path.is_file():
            raise MagikaError(
                f"model config not found at {str(self._model_config_path)}"
            )

        model_config = json.loads(self._model_config_path.read_text())
        self._target_labels_space = [
            ContentTypeLabel(ct) for ct in model_config["target_labels_space"]
        ]
        self._overwrite_map = {
            ContentTypeLabel(k): ContentTypeLabel(v)
            for k, v in model_config.get("overwrite_map", {}).items()
        }

        self._prediction_mode = prediction_mode
        self._no_dereference = no_dereference

        content_types_kb_path = (
            Path(__file__).parent / "config" / "content_types_kb.min.json"
        )
        self._cts_infos = Magika._load_content_types_kb(content_types_kb_path)

        self._pyo3_session = _magika.Magika()

    def __repr__(self) -> str:
        return str(self)

    def __str__(self) -> str:
        return f'Magika(module_version="{self.get_module_version()}", model_name="{self.get_model_name()}")'

    def get_module_version(self) -> str:
        """Gets the version of the Magika Python module."""
        return str(__import__(self.__module__).__version__)

    def get_model_name(self) -> str:
        """Gets the name of the loaded model."""
        return str(self._pyo3_session.get_model_name())

    def _convert_pyo3_result(self, pyo3_res: Any, path: Path) -> MagikaResult:
        status = Status(pyo3_res.status)
        if not pyo3_res.ok:
            return MagikaResult(path=path, status=status, prediction=None)

        output_info = self._cts_infos[ContentTypeLabel(pyo3_res.label)]
        dl_label = ContentTypeLabel(pyo3_res.dl_label)
        dl_info = self._cts_infos[dl_label]
        overwrite_reason = OverwriteReason(pyo3_res.overwrite_reason)

        prediction = MagikaPrediction(
            dl=dl_info,
            output=output_info,
            score=pyo3_res.score,
            overwrite_reason=overwrite_reason,
        )
        return MagikaResult(path=path, status=status, prediction=prediction)

    def identify_path(self, path: Union[str, os.PathLike]) -> MagikaResult:
        """Identify the content type of a file given its path."""
        if not isinstance(path, (str, os.PathLike)):
            raise TypeError(
                f"Path '{path}' is invalid: input path should be of type `Union[str, os.PathLike]`"
            )
        path = Path(path)
        pyo3_res = self._pyo3_session.identify_path(str(path))
        return self._convert_pyo3_result(pyo3_res, path)

    def identify_paths(
        self, paths: Sequence[Union[str, os.PathLike]]
    ) -> List[MagikaResult]:
        """Identify the content type of a list of files given their paths."""
        if not isinstance(paths, Sequence):
            raise TypeError("Input paths should be of type Sequence[Path]")

        paths_ = []
        for path in paths:
            if not isinstance(path, (str, os.PathLike)):
                raise TypeError(
                    f"Input '{path}' is invalid: input path should be of type `Union[str, os.PathLike]`"
                )
            paths_.append(Path(path))

        pyo3_results = self._pyo3_session.identify_paths([str(p) for p in paths_])
        return [
            self._convert_pyo3_result(res, p) for res, p in zip(pyo3_results, paths_)
        ]

    def identify_bytes(self, content: bytes) -> MagikaResult:
        """Identify the content type of raw bytes."""
        if not isinstance(content, bytes):
            raise TypeError(
                f"Input content should be of type 'bytes', not {type(content)}."
            )

        pyo3_res = self._pyo3_session.identify_bytes(content)
        return self._convert_pyo3_result(pyo3_res, Path("-"))

    def identify_stream(self, stream: BinaryIO) -> MagikaResult:
        """Identify the content type of a BinaryIO stream.

        Identifies the content type from an already-open binary file-like object
        (e.g., the output of `open(file_path, 'rb')`). Note: 1) Magika will
        `seek()` around the stream; 2) the stream _is not closed_ (closing it is
        the responsibility of the caller).
        """
        # Explicitly test for the most common error so that we can return an
        # helpful error message.
        stream_obj: object = stream
        if isinstance(stream_obj, io.TextIOBase):
            raise TypeError(
                "Input stream must be opened in bytes mode, not in text mode."
            )

        if not isinstance(stream_obj, io.BufferedIOBase) or not stream.readable():
            raise TypeError("Input stream must be a readable BinaryIO object.")

        if (
            not hasattr(stream, "seek")
            or not hasattr(stream, "read")
            or not hasattr(stream, "tell")
        ):
            raise TypeError("Input stream must have seek, read, and tell methods.")

        try:
            current_position = stream.tell()
            stream.seek(0)
            content = stream.read()
            return self.identify_bytes(content)
        finally:
            stream.seek(current_position)

    def get_output_content_types(self) -> List[ContentTypeLabel]:
        """This method returns the list of all possible output content types."""
        output_content_types: Set[ContentTypeLabel] = {
            ContentTypeLabel.DIRECTORY,
            ContentTypeLabel.EMPTY,
            ContentTypeLabel.SYMLINK,
            ContentTypeLabel.TXT,
            ContentTypeLabel.UNKNOWN,
        }
        for ct in self._target_labels_space:
            output_ct = self._overwrite_map.get(ct, ct)
            output_content_types.add(output_ct)

        return sorted(output_content_types)

    def get_model_content_types(self) -> List[ContentTypeLabel]:
        """This method returns the list of all possible output of the model."""
        model_content_types: Set[ContentTypeLabel] = {
            ContentTypeLabel.UNDEFINED,
        }
        model_content_types.update(self._target_labels_space)
        return sorted(model_content_types)

    def _get_output_label_from_dl_label_and_score(
        self, dl_label: ContentTypeLabel, score: float
    ) -> Tuple[ContentTypeLabel, OverwriteReason]:
        """Internal Python thresholding helper (deprecated/removed in favor of Rust core)."""
        raise NotImplementedError(
            "Output label thresholding is handled internally by the Rust backend."
        )

    @staticmethod
    def _get_default_model_name() -> str:
        """Returns the default model name."""
        return str(_magika.get_default_model_name())

    @staticmethod
    def _load_content_types_kb(
        content_types_kb_json_path: Path,
    ) -> Dict[ContentTypeLabel, ContentTypeInfo]:
        TXT_MIME_TYPE = "text/plain"
        UNKNOWN_MIME_TYPE = "application/octet-stream"
        UNKNOWN_GROUP = "unknown"

        out = {}
        for ct_name, ct_info in json.loads(
            content_types_kb_json_path.read_text()
        ).items():
            is_text = ct_info["is_text"]
            if is_text:
                default_mime_type = TXT_MIME_TYPE
            else:
                default_mime_type = UNKNOWN_MIME_TYPE
            mime_type = (
                default_mime_type
                if ct_info["mime_type"] is None
                else ct_info["mime_type"]
            )
            group = UNKNOWN_GROUP if ct_info["group"] is None else ct_info["group"]
            description = (
                ct_name if ct_info["description"] is None else ct_info["description"]
            )
            extensions = ct_info["extensions"]
            out[ContentTypeLabel(ct_name)] = ContentTypeInfo(
                label=ContentTypeLabel(ct_name),
                mime_type=mime_type,
                group=group,
                description=description,
                extensions=extensions,
                is_text=is_text,
            )
        return out
