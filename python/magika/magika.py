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

import json
import logging
import os
import time
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

import numpy as np
import numpy.typing as npt
import onnxruntime as rt
from tqdm.auto import tqdm

from magika.content_types import ContentType, ContentTypesManager
from magika.logger import get_logger
from magika.prediction_mode import PredictionMode
from magika.seekable import Buffer, File, Seekable
from magika.types import (
    MagikaOutputFields,
    MagikaResult,
    ModelFeatures,
    ModelFeaturesV2,
    ModelOutput,
    ModelOutputFields,
)


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

        self._disable_progress_bar = True

        self._magika_config = Magika._get_magika_config()

        # Default model, used in case not specified via the Magika constructor
        self._default_model_name = self._magika_config["default_model_name"]
        # Minimum threshold for "default" prediction mode
        self._medium_confidence_threshold = self._magika_config[
            "medium_confidence_threshold"
        ]
        # Minimum file size for using the DL model
        self._min_file_size_for_dl = self._magika_config["min_file_size_for_dl"]
        # Which integer we use to indicate padding
        self._padding_token = self._magika_config["padding_token"]
        self._block_size = self._magika_config["block_size"]

        if verbose:
            self._log.setLevel(logging.INFO)
            self._disable_progress_bar = False

        if debug:
            self._log.setLevel(logging.DEBUG)
            self._disable_progress_bar = False

        if model_dir is not None:
            self._model_dir = model_dir
        else:
            # use default model
            self._model_dir = (
                Path(__file__).parent / "models" / self._default_model_name
            )

        self._model_path = self._model_dir / "model.onnx"
        self._model_config_path = self._model_dir / "model_config.json"
        self._thresholds_path = self._model_dir / "thresholds.json"
        self._model_output_overwrite_map_path = (
            self._model_dir / "model_output_overwrite_map.json"
        )

        if not self._model_dir.is_dir():
            raise MagikaError(f"model dir not found at {str(self._model_dir)}")
        if not self._model_path.is_file():
            raise MagikaError(f"model not found at {str(self._model_path)}")
        if not self._model_config_path.is_file():
            raise MagikaError(
                f"model config not found at {str(self._model_config_path)}"
            )
        if not self._thresholds_path.is_file():
            raise MagikaError(f"thresholds not found at {str(self._thresholds_path)}")
        if not self._model_output_overwrite_map_path.is_file():
            raise MagikaError(
                f"thresholds not found at {str(self._model_output_overwrite_map_path)}"
            )

        self._model_config = json.loads(self._model_config_path.read_text())

        self._thresholds = json.loads(self._thresholds_path.read_text())["thresholds"]

        self._model_output_overwrite_map: Dict[str, str] = json.loads(
            self._model_output_overwrite_map_path.read_text()
        )

        self._input_sizes: Dict[str, int] = {
            "beg": self._model_config["cfg"]["input_sizes"]["beg"],
            "mid": self._model_config["cfg"]["input_sizes"]["mid"],
            "end": self._model_config["cfg"]["input_sizes"]["end"],
        }
        self._target_labels_space_np = np.array(
            self._model_config["train_dataset_info"]["target_labels_info"][
                "target_labels_space"
            ]
        )

        self._prediction_mode = prediction_mode

        self._no_dereference = no_dereference

        self._ctm = ContentTypesManager()
        self._onnx_session = self._init_onnx_session()

    def identify_path(self, path: Path) -> MagikaResult:
        return self._get_result_from_path(path)

    def identify_paths(self, paths: List[Path]) -> List[MagikaResult]:
        return self._get_results_from_paths(paths)

    def identify_bytes(self, content: bytes) -> MagikaResult:
        return self._get_result_from_bytes(content)

    @staticmethod
    def get_default_model_name() -> str:
        """This returns the default model name.

        We make this method static so that it can be used by the client (to
        print help, etc.) without the need to instantiate a Magika object.
        """

        return str(Magika._get_magika_config()["default_model_name"])

    def get_model_name(self) -> str:
        return self._model_dir.name

    def _init_onnx_session(self) -> rt.InferenceSession:
        start_time = time.time()
        onnx_session = rt.InferenceSession(
            self._model_path, providers=["CPUExecutionProvider"]
        )
        elapsed_time = time.time() - start_time
        self._log.debug(
            f'ONNX DL model "{self._model_path}" loaded in {elapsed_time:.03f} seconds'
        )
        return onnx_session

    @staticmethod
    def _get_magika_config() -> Dict[str, Any]:
        config_path = Path(__file__).parent / "config" / "magika_config.json"
        return json.loads(config_path.read_text())  # type: ignore[no-any-return]

    def _get_results_from_paths(self, paths: List[Path]) -> List[MagikaResult]:
        """Given a list of paths, returns a list of predictions. Each prediction
        is a dict with the relevant information, such as the file path, the
        output of the DL model, the output of the tool, and the associated
        metadata. The order of the predictions matches the order of the input
        paths."""

        start_time = time.time()

        # We do a first pass on all files: we collect features for the files
        # that need to be analyzed with the DL model, and we already determine
        # the output for the remaining ones.

        # We use a "str" instead of Path because it makes it easier later on to
        # serialize.
        all_outputs: Dict[str, MagikaResult] = {}  # {path: MagikaOutput, ...}

        # We use a list and not the dict because that's what we need later on
        # for inference.
        all_features: List[Tuple[Path, ModelFeatures]] = []

        self._log.debug(
            f"Processing input files and extracting features for {len(paths)} samples"
        )
        for path in tqdm(paths, disable=self._disable_progress_bar):
            output, features = self._get_result_or_features_from_path(path)
            if output is not None:
                all_outputs[str(path)] = output
            else:
                assert features is not None
                all_features.append((path, features))
        elapsed_time = time.time() - start_time
        self._log.debug(
            f"First pass and features extracted in {elapsed_time:.03f} seconds"
        )

        # Get the outputs via DL for the files that need it.
        outputs_with_dl = self._get_results_from_features(all_features)
        all_outputs.update(outputs_with_dl)

        # Finally, we collect the predictions in a final list, sorted by the
        # initial paths list (and not by insertion order).
        sorted_outputs = []
        for path in paths:
            sorted_outputs.append(all_outputs[str(path)])
        return sorted_outputs

    def _get_result_from_path(self, path: Path) -> MagikaResult:
        return self._get_results_from_paths([path])[0]

    def _get_result_from_bytes(self, content: bytes) -> MagikaResult:
        result, features = self._get_result_or_features_from_bytes(content)
        if result is not None:
            return result
        assert features is not None
        return self._get_result_from_features(features)

    @staticmethod
    def _extract_features_from_path(
        file_path: Path,
        beg_size: int,
        mid_size: int,
        end_size: int,
        padding_token: int,
        block_size: int,
    ) -> ModelFeatures:
        # TODO: reimplement this using a context manager
        seekable = File(file_path)
        mf = Magika._extract_features_from_seekable(
            seekable, beg_size, mid_size, end_size, padding_token, block_size
        )
        seekable.close()
        return mf

    @staticmethod
    def _extract_features_from_bytes(
        content: bytes,
        beg_size: int,
        mid_size: int,
        end_size: int,
        padding_token: int,
        block_size: int,
    ) -> ModelFeatures:
        buffer = Buffer(content)
        return Magika._extract_features_from_seekable(
            buffer, beg_size, mid_size, end_size, padding_token, block_size
        )

    @staticmethod
    def _extract_features_from_seekable(
        seekable: Seekable,
        beg_size: int,
        mid_size: int,
        end_size: int,
        padding_token: int,
        block_size: int,
    ) -> ModelFeatures:
        """This implement features extraction from a seekable, which is an
        abstraction about anything that can be "read_at" a specific offset, such
        as a file or buffer. This is implemented so that we do not need to load
        the entire content of the file in memory, and we do not need to scan the
        entire buffer.

        High-level overview on what we do:
        - beg: we read the first block in memory, we lstrip() it, and we use this as
        the basis to extract beg_size integers (we either truncate to beg_size
        or we add padding as suffix up to beg_size).
        - end: same as "beg", but we read the last block in memory, and the padding
        is prefixed (and not suffixed).
        - mid: we consider the remaining content (after stripping whitespace),
        and we take the mid_size bytes in the middle. If needed, we add padding
        to the left and to the right.
        """

        if seekable.size < (2 * block_size + mid_size):
            # If the content is small, we take this shortcut to avoid
            # checking for too many corner cases.
            content = seekable.read_at(0, seekable.size)
            content = content.strip()
            beg_content = content
            mid_content = content
            end_content = content

        else:  # seekable.size >= (2 * block_size + mid_size)
            # If the content is big enough, the implementation becomes much
            # simpler. In this path of the code, we know we have enough content
            # to strip up to "block_size" bytes from both sides, and still have
            # enough data for mid_size.

            beg_content = seekable.read_at(0, block_size).lstrip()

            end_content = seekable.read_at(
                seekable.size - block_size, block_size
            ).rstrip()

            # we extract "mid" from the middle of the content that we have not
            # trimmed
            trimmed_beg_bytes_num = block_size - len(beg_content)
            trimmed_end_bytes_num = block_size - len(end_content)
            # mid_idx points to the first byte of the middle block
            mid_idx = (
                trimmed_beg_bytes_num
                + (
                    seekable.size
                    - trimmed_beg_bytes_num
                    - trimmed_end_bytes_num
                    - mid_size
                )
                // 2
            )
            mid_content = seekable.read_at(mid_idx, mid_size)

        beg_ints = Magika._get_beg_ints_with_padding(
            beg_content, beg_size, padding_token
        )
        mid_ints = Magika._get_mid_ints_with_padding(
            mid_content, mid_size, padding_token
        )
        end_ints = Magika._get_end_ints_with_padding(
            end_content, end_size, padding_token
        )

        return ModelFeatures(beg=beg_ints, mid=mid_ints, end=end_ints)

    @staticmethod
    def _extract_features_from_seekable_v2(
        seekable: Seekable,
        beg_size: int,
        mid_size: int,
        end_size: int,
        padding_token: int,
        block_size: int,
    ) -> ModelFeaturesV2:
        """This implement v2 of the features extraction v2 from a seekable, which is an
        abstraction about anything that can be "read_at" a specific offset, such
        as a file or buffer. This is implemented so that we do not need to load
        the entire content of the file in memory, and we do not need to scan the
        entire buffer.

        This v2 is similar to v1; the main difference is that whether we strip
        some bytes from beg and end does not influence which bytes we pick for
        the middle part. This makes the implementation of v2 much simpler. And
        it makes it possible for a client to just read a block at the beginning,
        middle, and end, and send it to our backend for features extraction --
        no need for additional check on the client side.

        High-level overview on what we do:
        - beg: we read the first block in memory, we lstrip() it, and we use this as
        the basis to extract beg_size integers (we either truncate to beg_size
        or we add padding as suffix up to beg_size).
        - end: same as "beg", but we read the last block in memory, and the padding
        is prefixed (and not suffixed).
        - mid: we consider the entire content (note: for this v2, we do not care
        about whitespace stripping), and we take the mid_size bytes in the
        middle. If needed, we add padding to the left and to the right.
        """

        assert beg_size < block_size
        assert mid_size < block_size
        assert end_size < block_size

        if seekable.size < 2 * block_size:
            # If the content is small, we take this shortcut to avoid
            # checking for too many corner cases.
            content = seekable.read_at(0, seekable.size)
            content = content.strip()
            beg_content = content
            mid_content = content
            end_content = content

        else:  # seekable.size >= 2 * block_size
            # If the content is big enough, the implementation becomes much
            # simpler. In this path of the code, we know we have enough content
            # to strip up to "block_size" bytes from both sides.

            beg_content = seekable.read_at(0, block_size).lstrip()

            end_content = seekable.read_at(
                seekable.size - block_size, block_size
            ).rstrip()

            # we extract "mid" from the middle of the content
            # mid_idx points to the first byte of the middle block
            # == seekable.size//2 - mid_size//2
            mid_idx = (seekable.size - mid_size) // 2
            mid_content = seekable.read_at(mid_idx, mid_size)

        beg_ints = Magika._get_beg_ints_with_padding(
            beg_content, beg_size, padding_token
        )
        mid_ints = Magika._get_mid_ints_with_padding(
            mid_content, mid_size, padding_token
        )
        end_ints = Magika._get_end_ints_with_padding(
            end_content, end_size, padding_token
        )

        offset_0x8000_0x8007 = Magika._get_ints_at_offset_or_padding(
            seekable, 0x8000, 8, padding_token
        )
        offset_0x8800_0x8807 = Magika._get_ints_at_offset_or_padding(
            seekable, 0x8800, 8, padding_token
        )
        offset_0x9000_0x9007 = Magika._get_ints_at_offset_or_padding(
            seekable, 0x9000, 8, padding_token
        )
        offset_0x9800_0x9807 = Magika._get_ints_at_offset_or_padding(
            seekable, 0x9800, 8, padding_token
        )

        return ModelFeaturesV2(
            beg=beg_ints,
            mid=mid_ints,
            end=end_ints,
            offset_0x8000_0x8007=offset_0x8000_0x8007,
            offset_0x8800_0x8807=offset_0x8800_0x8807,
            offset_0x9000_0x9007=offset_0x9000_0x9007,
            offset_0x9800_0x9807=offset_0x9800_0x9807,
        )

    @staticmethod
    def _get_beg_ints_with_padding(
        beg_content: bytes, beg_size: int, padding_token: int
    ) -> List[int]:
        """Take an (already-stripped) buffer as input and extract beg ints. If
        the buffer is bigger than required, take only the initial portion. If
        the buffer is shorter, add padding at the end."""

        if beg_size <= len(beg_content):
            # we don't need so many bytes
            beg_content = beg_content[0:beg_size]

        beg_ints = list(map(int, beg_content))

        if len(beg_ints) < beg_size:
            # we don't have enough ints, add padding
            beg_ints = beg_ints + ([padding_token] * (beg_size - len(beg_ints)))

        assert len(beg_ints) == beg_size

        return beg_ints

    @staticmethod
    def _get_mid_ints_with_padding(
        mid_content: bytes, mid_size: int, padding_token: int
    ) -> List[int]:
        """Take an (already-stripped) buffer as input and extract mid ints. If
        the buffer is bigger than required, take only its middle part. If the
        buffer is shorter, add padding at its left and right.
        """

        if mid_size <= len(mid_content):
            mid_idx = (len(mid_content) - mid_size) // 2
            mid_content = mid_content[mid_idx : mid_idx + mid_size]

        mid_ints = list(map(int, mid_content))

        if len(mid_ints) < mid_size:
            # we don't have enough ints, add padding
            padding_size = mid_size - len(mid_ints)
            padding_size_left = padding_size // 2
            padding_size_right = padding_size - padding_size_left
            mid_ints = (
                ([padding_token] * padding_size_left)
                + mid_ints
                + ([padding_token] * padding_size_right)
            )

        assert len(mid_ints) == mid_size

        return mid_ints

    @staticmethod
    def _get_end_ints_with_padding(
        end_content: bytes, end_size: int, padding_token: int
    ) -> List[int]:
        """Take an (already-stripped) buffer as input and extract end ints. If
        the buffer is bigger than required, take only the last portion. If the
        buffer is shorter, add padding at the beginning.
        """

        if end_size <= len(end_content):
            # we don't need so many bytes
            end_content = end_content[len(end_content) - end_size : len(end_content)]

        end_ints = list(map(int, end_content))

        if len(end_ints) < end_size:
            # we don't have enough ints, add padding
            end_ints = ([padding_token] * (end_size - len(end_ints))) + end_ints

        assert len(end_ints) == end_size

        return end_ints

    @staticmethod
    def _get_ints_at_offset_or_padding(
        seekable: Seekable, offset: int, size: int, padding_token: int
    ) -> List[int]:
        if offset + size <= seekable.size:
            return list(map(int, seekable.read_at(offset, size)))
        return [padding_token] * size

    def _get_model_outputs_from_features(
        self, all_features: List[Tuple[Path, ModelFeatures]]
    ) -> List[Tuple[Path, ModelOutput]]:
        raw_preds = self._get_raw_predictions(all_features)
        top_preds_idxs = np.argmax(raw_preds, axis=1)
        preds_content_types_labels = self._target_labels_space_np[top_preds_idxs]
        scores = np.max(raw_preds, axis=1)

        return [
            (path, ModelOutput(ct_label=ct_label, score=float(score)))
            for (path, _), ct_label, score in zip(
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

        outputs: Dict[str, MagikaResult] = {}

        for path, model_output in self._get_model_outputs_from_features(all_features):
            # In additional to the content type label from the DL model, we
            # also allow for other logic to overwrite such result. For
            # debugging and information purposes, the JSON output stores
            # both the raw DL model output and the final output we return to
            # the user.

            output_ct_label = self._get_output_ct_label_from_dl_result(
                model_output.ct_label, model_output.score
            )

            outputs[str(path)] = self._get_result_from_labels_and_score(
                path,
                dl_ct_label=model_output.ct_label,
                output_ct_label=output_ct_label,
                score=model_output.score,
            )

        return outputs

    def _get_result_from_features(
        self, features: ModelFeatures, path: Optional[Path] = None
    ) -> MagikaResult:
        # This is useful to scan from stream of bytes
        if path is None:
            path = Path("-")
        all_features = [(Path("-"), features)]
        result_with_dl = self._get_results_from_features(all_features)["-"]
        return result_with_dl

    def _get_output_ct_label_from_dl_result(
        self, dl_ct_label: str, score: float
    ) -> str:
        # overwrite ct_label if specified in the config
        dl_ct_label = self._model_output_overwrite_map.get(dl_ct_label, dl_ct_label)

        if self._prediction_mode == PredictionMode.BEST_GUESS:
            # We take the model predictions, no matter what the score is.
            output_ct_label = dl_ct_label
        elif (
            self._prediction_mode == PredictionMode.HIGH_CONFIDENCE
            and score >= self._thresholds[dl_ct_label]
        ):
            # The model score is higher than the per-content-type
            # high-confidence threshold.
            output_ct_label = dl_ct_label
        elif (
            self._prediction_mode == PredictionMode.MEDIUM_CONFIDENCE
            and score >= self._medium_confidence_threshold
        ):
            # We take the model prediction only if the score is above a given
            # relatively loose threshold.
            output_ct_label = dl_ct_label
        else:
            # We are not in a condition to trust the model, we opt to return
            # generic labels. Note that here we use an implicit assumption that
            # the model has, at the very least, got the binary vs. text category
            # right. This allows us to pick between unknown and txt without the
            # need to read or scan the file bytes once again.
            if self._ctm.get_or_raise(dl_ct_label).is_text:
                output_ct_label = ContentType.GENERIC_TEXT
            else:
                output_ct_label = ContentType.UNKNOWN

        return output_ct_label

    def _get_result_from_labels_and_score(
        self, path: Path, dl_ct_label: Optional[str], score: float, output_ct_label: str
    ) -> MagikaResult:
        dl_score = None if dl_ct_label is None else score
        output_score = score

        # add group info
        dl_group = None if dl_ct_label is None else self._ctm.get_group(dl_ct_label)
        output_group = self._ctm.get_group(output_ct_label)

        # add mime type info
        dl_mime_type = (
            None if dl_ct_label is None else self._ctm.get_mime_type(dl_ct_label)
        )
        output_mime_type = self._ctm.get_mime_type(output_ct_label)

        # add magic
        dl_magic = None if dl_ct_label is None else self._ctm.get_magic(dl_ct_label)
        output_magic = self._ctm.get_magic(output_ct_label)

        # add description
        dl_description = (
            None if dl_ct_label is None else self._ctm.get_description(dl_ct_label)
        )
        output_description = self._ctm.get_description(output_ct_label)

        magika_result = MagikaResult(
            path=str(path),
            dl=ModelOutputFields(
                ct_label=dl_ct_label,
                score=dl_score,
                group=dl_group,
                mime_type=dl_mime_type,
                magic=dl_magic,
                description=dl_description,
            ),
            output=MagikaOutputFields(
                ct_label=output_ct_label,
                score=output_score,
                group=output_group,
                mime_type=output_mime_type,
                magic=output_magic,
                description=output_description,
            ),
        )

        return magika_result

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
                path, dl_ct_label=None, output_ct_label=ContentType.SYMLINK, score=1.0
            )
            # The magic and description fields for symlink contain a placeholder
            # for <path>; let's patch the output to reflect that.
            result.output.magic = result.output.magic.replace(
                "<path>", str(path.resolve())
            )
            result.output.description = result.output.description.replace(
                "<path>", str(path.resolve())
            )
            return result, None

        if not path.exists():
            result = self._get_result_from_labels_and_score(
                path,
                dl_ct_label=None,
                output_ct_label=ContentType.FILE_DOES_NOT_EXIST,
                score=1.0,
            )
            return result, None

        if path.is_file():
            if path.stat().st_size == 0:
                result = self._get_result_from_labels_and_score(
                    path, dl_ct_label=None, output_ct_label=ContentType.EMPTY, score=1.0
                )
                return result, None

            elif not os.access(path, os.R_OK):
                result = self._get_result_from_labels_and_score(
                    path,
                    dl_ct_label=None,
                    output_ct_label=ContentType.PERMISSION_ERROR,
                    score=1.0,
                )
                return result, None

            elif path.stat().st_size <= self._min_file_size_for_dl:
                result = self._get_result_from_first_block_of_file(path)
                return result, None

            else:
                file_features = Magika._extract_features_from_path(
                    path,
                    self._input_sizes["beg"],
                    self._input_sizes["mid"],
                    self._input_sizes["end"],
                    self._padding_token,
                    self._block_size,
                )
                # Check whether we have enough bytes for a meaningful
                # detection, and not just padding.
                if (
                    file_features.beg[self._min_file_size_for_dl - 1]
                    == self._padding_token
                ):
                    # If the n-th token is padding, then it means that,
                    # post-stripping, we do not have enough meaningful
                    # bytes.
                    result = self._get_result_from_first_block_of_file(path)
                    return result, None

                else:
                    # We have enough bytes, scheduling this file for model
                    # prediction.
                    # features.append((path, file_features))
                    return None, file_features

        elif path.is_dir():
            result = self._get_result_from_labels_and_score(
                path, dl_ct_label=None, output_ct_label=ContentType.DIRECTORY, score=1.0
            )
            return result, None

        else:
            result = self._get_result_from_labels_and_score(
                path, dl_ct_label=None, output_ct_label=ContentType.UNKNOWN, score=1.0
            )
            return result, None

        raise Exception("unreachable")

    def _get_result_or_features_from_bytes(
        self, content: bytes
    ) -> Tuple[Optional[MagikaResult], Optional[ModelFeatures]]:
        if len(content) == 0:
            output = self._get_result_from_labels_and_score(
                Path("-"),
                dl_ct_label=None,
                output_ct_label=ContentType.EMPTY,
                score=1.0,
            )
            return output, None

        elif len(content) <= self._min_file_size_for_dl:
            output = self._get_result_of_few_bytes(content)
            return output, None

        else:
            file_features = Magika._extract_features_from_bytes(
                content,
                self._input_sizes["beg"],
                self._input_sizes["mid"],
                self._input_sizes["end"],
                self._padding_token,
                self._block_size,
            )
            # Check whether we have enough bytes for a meaningful
            # detection, and not just padding.
            if file_features.beg[self._min_file_size_for_dl - 1] == self._padding_token:
                # If the n-th token is padding, then it means that,
                # post-stripping, we do not have enough meaningful
                # bytes.
                output = self._get_result_of_few_bytes(content)
                return output, None

            else:
                # We have enough bytes, scheduling this file for model
                # prediction.
                # features.append((path, file_features))
                return None, file_features

        raise Exception("unreachable")

    def _get_result_from_first_block_of_file(self, path: Path) -> MagikaResult:
        # We read at most "block_size" bytes
        with open(path, "rb") as f:
            content = f.read(self._block_size)
        return self._get_result_of_few_bytes(content, path)

    def _get_result_of_few_bytes(
        self, content: bytes, path: Path = Path("-")
    ) -> MagikaResult:
        assert len(content) <= 4 * self._block_size
        ct_label = self._get_ct_label_of_few_bytes(content)
        return self._get_result_from_labels_and_score(
            path, dl_ct_label=None, output_ct_label=ct_label, score=1.0
        )

    def _get_ct_label_of_few_bytes(self, content: bytes) -> str:
        try:
            ct_label = ContentType.GENERIC_TEXT
            _ = content.decode("utf-8")
        except UnicodeDecodeError:
            ct_label = ContentType.UNKNOWN
        return ct_label

    def _get_raw_predictions(
        self, features: List[Tuple[Path, ModelFeatures]]
    ) -> npt.NDArray:
        """
        Given a list of (path, features), return a (files_num, features_size)
        matrix encoding the predictions.
        """

        dataset_format = self._model_config["train_dataset_info"]["dataset_format"]
        assert dataset_format == "int-concat/one-hot"
        start_time = time.time()
        X_bytes = []
        for _, fs in features:
            sample_bytes = []
            if self._input_sizes["beg"] > 0:
                sample_bytes.extend(fs.beg[: self._input_sizes["beg"]])
            if self._input_sizes["mid"] > 0:
                sample_bytes.extend(fs.mid[: self._input_sizes["mid"]])
            if self._input_sizes["end"] > 0:
                sample_bytes.extend(fs.end[-self._input_sizes["end"] :])
            X_bytes.append(sample_bytes)
        X = np.array(X_bytes).astype(np.float32)
        elapsed_time = time.time() - start_time
        self._log.debug(f"DL input prepared in {elapsed_time:.03f} seconds")

        start_time = time.time()
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
            batch_raw_predictions = self._onnx_session.run(
                ["target_label"], {"bytes": X[start_idx:end_idx, :]}
            )[0]
            raw_predictions_list.append(batch_raw_predictions)
        elapsed_time = time.time() - start_time
        self._log.debug(f"DL raw prediction in {elapsed_time:.03f} seconds")
        return np.concatenate(raw_predictions_list)


class MagikaError(Exception):
    pass
