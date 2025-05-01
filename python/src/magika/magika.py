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
import json
import logging
import os
import time
from pathlib import Path
from typing import BinaryIO, Dict, List, Optional, Sequence, Set, Tuple, Union

import numpy as np
import numpy.typing as npt
import onnxruntime as rt

from magika.logger import get_logger
from magika.types import (
    ContentTypeInfo,
    ContentTypeLabel,
    MagikaError,
    MagikaPrediction,
    MagikaResult,
    ModelConfig,
    ModelFeatures,
    ModelOutput,
    OverwriteReason,
    PredictionMode,
    Seekable,
    Status,
)

_DEFAULT_MODEL_NAME = "standard_v3_3"


class Magika:
    def __init__(
        self,
        model_dir: Optional[Path] = None,
        prediction_mode: PredictionMode = PredictionMode.HIGH_CONFIDENCE,
        no_dereference: bool = False,
        verbose: bool = False,
        debug: bool = False,
        use_colors: bool = False,
    ) -> None:
        self._log = get_logger(use_colors=use_colors)

        if verbose:
            self._log.setLevel(logging.INFO)

        if debug:
            self._log.setLevel(logging.DEBUG)

        if model_dir is not None:
            self._model_dir = model_dir
        else:
            # use default model
            self._model_dir = (
                Path(__file__).parent / "models" / self._get_default_model_name()
            )

        self._model_path = self._model_dir / "model.onnx"
        self._model_config_path = self._model_dir / "config.min.json"

        if not self._model_dir.is_dir():
            raise MagikaError(f"model dir not found at {str(self._model_dir)}")
        if not self._model_path.is_file():
            raise MagikaError(f"model not found at {str(self._model_path)}")
        if not self._model_config_path.is_file():
            raise MagikaError(
                f"model config not found at {str(self._model_config_path)}"
            )

        self._model_config: ModelConfig = Magika._load_model_config(
            self._model_config_path
        )

        self._target_labels_space_np = np.array(
            list(map(str, self._model_config.target_labels_space))
        )

        self._prediction_mode = prediction_mode

        self._no_dereference = no_dereference

        content_types_kb_path = (
            Path(__file__).parent / "config" / "content_types_kb.min.json"
        )
        self._cts_infos = Magika._load_content_types_kb(content_types_kb_path)

        self._onnx_session = self._init_onnx_session()

    def __repr__(self) -> str:
        return str(self)

    def __str__(self) -> str:
        return f'Magika(module_version="{self.get_module_version()}", model_name="{self.get_model_name()}")'

    def get_module_version(self) -> str:
        return str(__import__(self.__module__).__version__)

    def get_model_name(self) -> str:
        return self._model_dir.name

    def identify_path(self, path: Union[str, os.PathLike]) -> MagikaResult:
        """Identify the content type of a file given its path."""

        if isinstance(path, str) or isinstance(path, os.PathLike):
            path = Path(path)
        else:
            raise TypeError(
                f"Path '{path}' is invalid: input path should be of type `Union[str, os.PathLike]`"
            )

        return self._get_result_from_path(path)

    def identify_paths(
        self, paths: Sequence[Union[str, os.PathLike]]
    ) -> List[MagikaResult]:
        """Identify the content type of a list of files given their paths."""

        if not isinstance(paths, Sequence):
            raise TypeError("Input paths should be of type Sequence[Path]")

        paths_ = []
        for path in paths:
            if isinstance(path, str) or isinstance(path, os.PathLike):
                paths_.append(Path(path))
            else:
                raise TypeError(
                    f"Input '{path}' is invalid: input path should be of type `Union[str, os.PathLike]`"
                )

        return self._get_results_from_paths(paths_)

    def identify_bytes(self, content: bytes) -> MagikaResult:
        """Identify the content type of raw bytes."""

        if not isinstance(content, bytes):
            raise TypeError(
                f"Input content should be of type 'bytes', not {type(content)}."
            )

        return self._get_result_from_seekable(Seekable(io.BytesIO(content)))

    def identify_stream(self, stream: BinaryIO) -> MagikaResult:
        """Identify the content type of a BinaryIO stream. Note that this method will
        seek() around the stream."""

        if not isinstance(stream, io.IOBase) or not stream.readable():  # type: ignore[unreachable]
            raise TypeError("Input stream must be a readable BinaryIO object.")

        # Explicitly test for the most common error so that we can return an
        # helpful error message.
        if isinstance(stream, io.TextIOBase):  # type: ignore[unreachable]
            raise TypeError(
                "Input stream must be opened in bytes mode, not in text mode."
            )

        if not isinstance(stream, io.BufferedIOBase):
            raise TypeError("Input stream must be a readable BinaryIO object.")

        if (
            not hasattr(stream, "seek")
            or not hasattr(stream, "read")
            or not hasattr(stream, "tell")
        ):
            raise TypeError("Input stream must have seek, read, and tell methods.")

        try:
            current_position = stream.tell()
            result = self._get_result_from_seekable(Seekable(stream))
        finally:
            # seek to the previous position even in case of exceptions
            stream.seek(current_position)
        return result

    def get_output_content_types(self) -> List[ContentTypeLabel]:
        """This method returns the list of all possible output content types of
        the module. I.e., all possible values for
        `MagikaResult.prediction.output.label`.  This considers the list of
        possible outputs from the model itself, but also keeps into account
        additional configuration such as `override_map` and special content
        types such as `ContentTypeLabel.EMPTY` or `ContentTypeLabel.SYMLINK`.
        """

        target_labels_space = self._model_config.target_labels_space
        overwrite_map = self._model_config.overwrite_map

        output_content_types: Set[ContentTypeLabel] = {
            ContentTypeLabel.DIRECTORY,
            ContentTypeLabel.EMPTY,
            ContentTypeLabel.SYMLINK,
            ContentTypeLabel.TXT,
            ContentTypeLabel.UNKNOWN,
        }
        for ct in target_labels_space:
            # Check if we would overwrite this target label; if not, use the
            # target label itself.
            output_ct = overwrite_map.get(ct, ct)
            output_content_types.add(output_ct)

        return sorted(output_content_types)

    def get_model_content_types(self) -> List[ContentTypeLabel]:
        """This method returns the list of all possible output of the underlying
        model. I.e., all possible values for `MagikaResult.prediction.dl.label`.
        Note that, in general, the list of "model outputs" is different than the
        "tool outputs" as in some cases the model is not even used, or the
        model's output is overwritten due to a low-confidence score, or other
        reasons.  This API is useful mostly for debugging purposes; the vast
        majority of client should use `get_output_content_types()`.
        """

        model_content_types: Set[ContentTypeLabel] = {
            ContentTypeLabel.UNDEFINED,
        }
        model_content_types.update(self._model_config.target_labels_space)
        return sorted(model_content_types)

    @staticmethod
    def _get_default_model_name() -> str:
        """This returns the default model name.

        We make this method static so that it can be used by external
        clients/tests without the need to instantiate a Magika object.
        """

        return _DEFAULT_MODEL_NAME

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

    @staticmethod
    def _load_model_config(model_config_path: Path) -> ModelConfig:
        config = json.loads(model_config_path.read_text())

        return ModelConfig(
            beg_size=config["beg_size"],
            mid_size=config["mid_size"],
            end_size=config["end_size"],
            use_inputs_at_offsets=config["use_inputs_at_offsets"],
            medium_confidence_threshold=config["medium_confidence_threshold"],
            min_file_size_for_dl=config["min_file_size_for_dl"],
            padding_token=config["padding_token"],
            block_size=config["block_size"],
            target_labels_space=[
                ContentTypeLabel(ct_str) for ct_str in config["target_labels_space"]
            ],
            thresholds={
                ContentTypeLabel(k): v for k, v in config["thresholds"].items()
            },
            overwrite_map={
                ContentTypeLabel(k): ContentTypeLabel(v)
                for k, v in config["overwrite_map"].items()
            },
        )

    def _init_onnx_session(self) -> rt.InferenceSession:
        start_time = time.time()
        rt.disable_telemetry_events()

        onnx_session = rt.InferenceSession(
            self._model_path,
            providers=["CPUExecutionProvider"],
        )
        elapsed_time = 1000 * (time.time() - start_time)
        self._log.debug(
            f'ONNX DL model "{self._model_path}" loaded in {elapsed_time:.03f} ms'
        )
        return onnx_session

    def _get_ct_info(self, content_type: ContentTypeLabel) -> ContentTypeInfo:
        return self._cts_infos[content_type]

    def _get_results_from_paths(self, paths: List[Path]) -> List[MagikaResult]:
        """Given a list of paths, returns a list of MagikaResult objects, which
        contain relevant information, such as: file path, the output of the DL
        model, the confidence score, the output of the tool, and associated
        metadata. The order of the predictions matches the order of the input
        paths."""

        # We do a first pass on all files: we collect features for the files
        # that need to be analyzed with the DL model, and we already determine
        # the output for the remaining ones.

        # We use a "str" instead of Path because it makes it easier later on to
        # serialize.
        all_outputs: Dict[str, MagikaResult] = {}  # {path: <output>, ...}

        # We use a list and not the dict because that's what we need later on
        # for inference.
        all_features: List[Tuple[Path, ModelFeatures]] = []

        self._log.debug(
            f"Processing input files and extracting features for {len(paths)} samples"
        )
        start_time = time.time()
        for path in paths:
            output, features = self._get_result_or_features_from_path(path)
            if output is not None:
                all_outputs[str(path)] = output
            else:
                assert features is not None
                all_features.append((path, features))
        elapsed_time = 1000 * (time.time() - start_time)
        self._log.debug(f"First pass and features extracted in {elapsed_time:.03f} ms")

        # Get the outputs via DL for the files that need it.
        for path_str, result in self._get_results_from_features(all_features).items():
            all_outputs[path_str] = result

        # Finally, we collect the predictions in a final list, sorted by the
        # initial paths list (and not by insertion order).
        sorted_outputs = []
        for path in paths:
            sorted_outputs.append(all_outputs[str(path)])
        return sorted_outputs

    def _get_result_from_path(self, path: Path) -> MagikaResult:
        return self._get_results_from_paths([path])[0]

    def _get_result_from_seekable(self, seekable: Seekable) -> MagikaResult:
        result, features = self._get_result_or_features_from_seekable(seekable)
        if result is not None:
            return result
        assert features is not None
        return self._get_results_from_features([(Path("-"), features)])["-"]

    @staticmethod
    def _extract_features_from_seekable(
        seekable: Seekable,
        beg_size: int,
        mid_size: int,
        end_size: int,
        padding_token: int,
        block_size: int,
        use_inputs_at_offsets: bool,
    ) -> ModelFeatures:
        """Extract features from an input seekable.

        This implements features extraction v2 from a seekable, which is an
        abstraction about anything that has a size and that can be "read_at" a
        specific offset, such as a file or a buffer. This is implemented so that
        we do not need to load the entire file in memory or scan the entire
        buffer.

        High-level overview on what we do:
        - We read (at most) `block_size` bytes from the beginning and from the
        end.
        - We normalize these bytes by stripping whitespaces.
        - We consider `beg_size` and `end_size` bytes as `beg` and `end`
        features. If we don't have enough bytes, we use `padding_token` as
        padding.

        See comments below for the specifics and handling of corner cases.

        NOTE: This implementation does not support extraction of `mid` features
        and `use_inputs_at_offsets`.
        """

        assert beg_size < block_size
        assert mid_size == 0
        assert end_size < block_size
        assert not use_inputs_at_offsets

        # we read at most block_size bytes
        bytes_num_to_read = min(block_size, seekable.size)

        if beg_size > 0:
            # Read at most `block_size` bytes from the beginning; `lstrip()``
            # them (or `strip()` them if the file size is less or equal than
            # `block_size`); take at most `beg_size` bytes, and optionally pad
            # them with `padding_token` to get to a list of `beg_size` integers.
            beg_content = seekable.read_at(0, bytes_num_to_read)
            beg_content = beg_content.lstrip()
            beg_ints = Magika._get_beg_ints_with_padding(
                beg_content, beg_size, padding_token
            )
        else:
            beg_ints = []

        if end_size > 0:
            # Read at most `block_size` bytes from the end; `rstrip()`` them (or
            # `strip()` them if the file size is less or equal than
            # `block_size`); take at most `end_size` bytes (from the end), and
            # optionally pad them (at the beginning) with `padding_token` to get
            # to a list of `end_size` integers.
            end_content = seekable.read_at(
                seekable.size - bytes_num_to_read, bytes_num_to_read
            )
            end_content = end_content.rstrip()
            end_ints = Magika._get_end_ints_with_padding(
                end_content, end_size, padding_token
            )
        else:
            end_ints = []

        return ModelFeatures(
            beg=beg_ints,
            mid=[],
            end=end_ints,
            offset_0x8000_0x8007=[],
            offset_0x8800_0x8807=[],
            offset_0x9000_0x9007=[],
            offset_0x9800_0x9807=[],
        )

    @staticmethod
    def _get_beg_ints_with_padding(
        beg_content: bytes, beg_size: int, padding_token: int
    ) -> List[int]:
        """Take an (already-stripped) buffer as input and extract beg ints.
        This returns a list of integers whose length is exactly beg_size. If
        the buffer is bigger than required, take only the initial portion. If
        the buffer is shorter, add padding at the end.
        """

        if beg_size < len(beg_content):
            # we don't need so many bytes
            beg_content = beg_content[0:beg_size]

        beg_ints = list(map(int, beg_content))

        if len(beg_ints) < beg_size:
            # we don't have enough ints, add padding
            beg_ints = beg_ints + ([padding_token] * (beg_size - len(beg_ints)))

        assert len(beg_ints) == beg_size

        return beg_ints

    @staticmethod
    def _get_end_ints_with_padding(
        end_content: bytes, end_size: int, padding_token: int
    ) -> List[int]:
        """Take an (already-stripped) buffer as input and extract end ints. This
        returns a list of integers whose length is exactly end_size.  If the
        buffer is bigger than required, take only the last portion. If the
        buffer is shorter, add padding at the beginning.
        """

        if end_size < len(end_content):
            # we don't need so many bytes
            end_content = end_content[len(end_content) - end_size : len(end_content)]

        end_ints = list(map(int, end_content))

        if len(end_ints) < end_size:
            # we don't have enough ints, add padding
            end_ints = ([padding_token] * (end_size - len(end_ints))) + end_ints

        assert len(end_ints) == end_size

        return end_ints

    def _get_model_outputs_from_features(
        self, all_features: List[Tuple[Path, ModelFeatures]]
    ) -> List[Tuple[Path, ModelOutput]]:
        raw_preds = self._get_raw_predictions(all_features)
        top_preds_idxs = np.argmax(raw_preds, axis=1)
        preds_content_types_labels = self._target_labels_space_np[top_preds_idxs]
        scores = np.max(raw_preds, axis=1)

        return [
            (path, ModelOutput(label=ContentTypeLabel(label), score=float(score)))
            for (path, _), label, score in zip(
                all_features, preds_content_types_labels, scores
            )
        ]

    def _get_results_from_features(
        self, all_features: List[Tuple[Path, ModelFeatures]]
    ) -> Dict[str, MagikaResult]:
        # We now do inference for those files that need it.

        if len(all_features) == 0:
            # nothing to be done
            return {}

        results: Dict[str, MagikaResult] = {}

        for path, model_output in self._get_model_outputs_from_features(all_features):
            # In additional to the content type label from the DL model, we
            # also allow for other logic to overwrite such result. For
            # debugging and information purposes, the JSON output stores
            # both the raw DL model output and the final output we return to
            # the user.

            output_label, overwrite_reason = (
                self._get_output_label_from_dl_label_and_score(
                    model_output.label, model_output.score
                )
            )

            results[str(path)] = self._get_result_from_labels_and_score(
                path=path,
                dl_label=model_output.label,
                output_label=output_label,
                score=model_output.score,
                overwrite_reason=overwrite_reason,
            )

        return results

    def _get_output_label_from_dl_label_and_score(
        self, dl_label: ContentTypeLabel, score: float
    ) -> Tuple[ContentTypeLabel, OverwriteReason]:
        overwrite_reason = OverwriteReason.NONE

        # Overwrite dl_label if specified in the overwrite_map model config.
        output_label = self._model_config.overwrite_map.get(dl_label, dl_label)
        if output_label != dl_label:
            overwrite_reason = OverwriteReason.OVERWRITE_MAP

        # The following code checks whether the score is "high enough", where
        # "high enough" depends on the selected prediction mode. If the score is
        # high enough, we return the (potentially ovewritten) model prediction;
        # if it is not, we return a generic content type, such as TXT or
        # UNKNOWN.
        if self._prediction_mode == PredictionMode.BEST_GUESS:
            # We take the (potentially overwritten) model prediction, no matter
            # what the score is.
            pass
        elif (
            self._prediction_mode == PredictionMode.HIGH_CONFIDENCE
            and score
            >= self._model_config.thresholds.get(
                dl_label, self._model_config.medium_confidence_threshold
            )
        ):
            # The model score is higher than the per-content-type
            # high-confidence threshold, so we keep it (note that the model
            # prediction may have been overwritten).
            pass
        elif (
            self._prediction_mode == PredictionMode.MEDIUM_CONFIDENCE
            and score >= self._model_config.medium_confidence_threshold
        ):
            # The model score is higher than the generic medium-confidence
            # threshold, so we keep it (note that the model prediction may have
            # been overwritten).
            pass
        else:
            # We are not in a condition to trust the model, we opt to return
            # generic labels. Note that here we use an implicit assumption that
            # the model has, at the very least, got the binary vs. text category
            # right. This allows us to pick between unknown and txt without the
            # need to read or scan the file bytes once again.
            overwrite_reason = OverwriteReason.LOW_CONFIDENCE
            if self._get_ct_info(output_label).is_text:
                output_label = ContentTypeLabel.TXT
            else:
                output_label = ContentTypeLabel.UNKNOWN
            if dl_label == output_label:
                # overwrite_reason is useful to convey to clients why the output
                # predicted is different than the model predicted type; if those
                # two are the same, the model predicted type has not actually
                # been overwritten, so we set this to NONE.
                overwrite_reason = OverwriteReason.NONE

        return output_label, overwrite_reason

    def _get_result_from_labels_and_score(
        self,
        path: Path,
        dl_label: ContentTypeLabel,
        output_label: ContentTypeLabel,
        score: float,
        overwrite_reason: OverwriteReason = OverwriteReason.NONE,
    ) -> MagikaResult:
        return MagikaResult(
            path=path,
            prediction=MagikaPrediction(
                dl=self._get_ct_info(dl_label),
                output=self._get_ct_info(output_label),
                score=score,
                overwrite_reason=overwrite_reason,
            ),
        )

    def _get_result_or_features_from_path(
        self, path: Path
    ) -> Tuple[Optional[MagikaResult], Optional[ModelFeatures]]:
        """
        Given a path, we return either a MagikaOutput or a MagikaFeatures.

        There are some files and corner cases for which we do not need to use
        deep learning to get the output; in these cases, we already return a
        MagikaOutput object.

        For some other files, we do need to use deep learning, in which case we
        return a MagikaFeatures object. Note that for now we just collect the
        features instead of already performing inference because we want to use
        batching.
        """

        if self._no_dereference and path.is_symlink():
            result = self._get_result_from_labels_and_score(
                path=path,
                dl_label=ContentTypeLabel.UNDEFINED,
                output_label=ContentTypeLabel.SYMLINK,
                score=1.0,
            )
            return result, None

        if not path.exists():
            return MagikaResult(path=path, status=Status.FILE_NOT_FOUND_ERROR), None

        if path.is_file():
            if not os.access(path, os.R_OK):
                return MagikaResult(path=path, status=Status.PERMISSION_ERROR), None

            else:
                # There are no additional path-specific corner cases, we can
                # treat the input path as a stream.
                with open(path, "rb") as stream:
                    return self._get_result_or_features_from_seekable(
                        Seekable(stream), path
                    )

        elif path.is_dir():
            result = self._get_result_from_labels_and_score(
                path=path,
                dl_label=ContentTypeLabel.UNDEFINED,
                output_label=ContentTypeLabel.DIRECTORY,
                score=1.0,
            )
            return result, None

        else:
            result = self._get_result_from_labels_and_score(
                path=path,
                dl_label=ContentTypeLabel.UNDEFINED,
                output_label=ContentTypeLabel.UNKNOWN,
                score=1.0,
            )
            return result, None

        raise Exception("unreachable")

    def _get_result_or_features_from_seekable(
        self, seekable: Seekable, path: Path = Path("-")
    ) -> Tuple[Optional[MagikaResult], Optional[ModelFeatures]]:
        """
        Given a Seekable object (which is a wrapper of BinaryIO), we return
        either a MagikaOutput or a MagikaFeatures.

        There are some corner cases for which we do not need to use deep
        learning to get the output; in these cases, we return directly a
        MagikaOutput object.

        For all other cases, we do need to use deep learning, in which case we
        return a MagikaFeatures object. Note that for now we just collect the
        features instead of already performing inference because we want to use
        batching.
        """

        if seekable.size == 0:
            result = self._get_result_from_labels_and_score(
                path=path,
                dl_label=ContentTypeLabel.UNDEFINED,
                output_label=ContentTypeLabel.EMPTY,
                score=1.0,
            )
            return result, None

        elif seekable.size < self._model_config.min_file_size_for_dl:
            content = seekable.read_at(0, seekable.size)
            result = self._get_result_from_few_bytes(content, path=path)
            return result, None

        else:
            file_features = Magika._extract_features_from_seekable(
                seekable,
                self._model_config.beg_size,
                self._model_config.mid_size,
                self._model_config.end_size,
                self._model_config.padding_token,
                self._model_config.block_size,
                self._model_config.use_inputs_at_offsets,
            )
            # Check whether we have enough bytes for a meaningful
            # detection, and not just padding.
            if (
                file_features.beg[self._model_config.min_file_size_for_dl - 1]
                == self._model_config.padding_token
            ):
                # If the n-th token is padding, then it means that,
                # post-stripping, we do not have enough meaningful
                # bytes.
                bytes_to_read = min(seekable.size, self._model_config.block_size)
                content = seekable.read_at(0, bytes_to_read)
                result = self._get_result_from_few_bytes(content, path=path)
                return result, None

            else:
                # We have enough bytes, return the features for a model
                # prediction.
                return None, file_features

        raise Exception("unreachable")

    def _get_result_from_few_bytes(
        self, content: bytes, path: Path = Path("-")
    ) -> MagikaResult:
        assert len(content) <= 4 * self._model_config.block_size
        label = self._get_label_from_few_bytes(content)
        return self._get_result_from_labels_and_score(
            path=path,
            dl_label=ContentTypeLabel.UNDEFINED,
            output_label=label,
            score=1.0,
        )

    def _get_label_from_few_bytes(self, content: bytes) -> ContentTypeLabel:
        try:
            label = ContentTypeLabel.TXT
            _ = content.decode("utf-8")
        except UnicodeDecodeError:
            label = ContentTypeLabel.UNKNOWN
        return label

    def _get_raw_predictions(
        self, features: List[Tuple[Path, ModelFeatures]]
    ) -> npt.NDArray:
        """
        Given a list of (path, features), return a (files_num, features_size)
        matrix encoding the predictions.
        """

        start_time = time.time()
        X_bytes = []
        for _, fs in features:
            sample_bytes = []
            if self._model_config.beg_size > 0:
                sample_bytes.extend(fs.beg[: self._model_config.beg_size])
            if self._model_config.mid_size > 0:
                sample_bytes.extend(fs.mid[: self._model_config.mid_size])
            if self._model_config.end_size > 0:
                sample_bytes.extend(fs.end[-self._model_config.end_size :])
            X_bytes.append(sample_bytes)
        X = np.array(X_bytes, dtype=np.int32)
        elapsed_time = 1000 * (time.time() - start_time)
        self._log.debug(f"DL input prepared in {elapsed_time:.03f} ms")

        raw_predictions_list = []
        samples_num = X.shape[0]

        max_internal_batch_size = 1000
        batches_num = samples_num // max_internal_batch_size
        if samples_num % max_internal_batch_size != 0:
            batches_num += 1

        for batch_idx in range(batches_num):
            self._log.debug(
                f"Getting raw predictions for (internal) batch {batch_idx+1}/{batches_num}"
            )
            start_idx = batch_idx * max_internal_batch_size
            end_idx = min((batch_idx + 1) * max_internal_batch_size, samples_num)

            start_time = time.time()
            batch_raw_predictions = self._onnx_session.run(
                ["target_label"], {"bytes": X[start_idx:end_idx, :]}
            )[0]
            elapsed_time = 1000 * (time.time() - start_time)
            self._log.debug(f"DL raw prediction in {elapsed_time:.03f} ms")

            raw_predictions_list.append(batch_raw_predictions)
        return np.concatenate(raw_predictions_list)
