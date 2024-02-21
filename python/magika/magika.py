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
from magika.types import (
    MagikaOutputFields,
    MagikaResult,
    ModelFeatures,
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
        onnx_session = rt.InferenceSession(self._model_path, providers=['CPUExecutionProvider'])
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

    def _extract_features_from_path(
        self,
        file_path: Path,
        beg_size: Optional[int] = None,
        mid_size: Optional[int] = None,
        end_size: Optional[int] = None,
        padding_token: Optional[int] = None,
    ) -> ModelFeatures:
        # Note: it is critical that this reflects exactly how we are extracting
        # features for training.

        # Ideally, we should seek around the file instead of reading the full
        # content. In practice, however, this is not trivial as we need to strip
        # whitespace-like characters first. Now, it's possible to code something
        # like this in a simple way, but it turns out it's challenging to write
        # a seek-only algorithm that is 100% the same as how we extracted
        # features during training. For example, consider a 129-byte file with
        # 128 ASCII letters + a space, and beg feature size of 512. Let's say
        # you read the first 129 characters, the question is: what should you do
        # with the trailing space? We know we should strip it, but that's the
        # case only because it's the end of the file and there are no other
        # non-whitespace characters left. In the general case, this means we
        # would need to read more bytes (more than 129) to determine what to do
        # with the trailing space (whether to keep it or to strip + consider it
        # as padding).  For all these reasons, for now we implement the exact
        # clone of what we do for features extraction, even if not ideal.

        if beg_size is None:
            beg_size = self._input_sizes["beg"]
        if mid_size is None:
            mid_size = self._input_sizes["mid"]
        if end_size is None:
            end_size = self._input_sizes["end"]
        if padding_token is None:
            padding_token = self._padding_token

        assert beg_size <= 512
        assert mid_size <= 512
        assert end_size <= 512

        block_size = 4096

        file_size = file_path.stat().st_size

        if file_size < 2 * block_size:
            # fast path for small files
            with open(file_path, "rb") as f:
                content = f.read()
            return self._extract_features_from_bytes(
                content, beg_size, mid_size, end_size, padding_token
            )
        else:
            # avoid reading the entire file
            with open(file_path, "rb") as f:
                if beg_size > 0:
                    beg_full = f.read(block_size)
                    beg_full_orig_size = len(beg_full)
                    beg_full = beg_full.lstrip()
                    beg_trimmed_size = beg_full_orig_size - len(beg_full)
                    if len(beg_full) < beg_size:
                        # Note that this is an approximation with respect what we do
                        # at feature extraction. What we do is different if, for
                        # example, the first two blocks of content are whitespaces:
                        # for feature extraction, we would keep reading content,
                        # while here we stop after two blocks.
                        beg_full += f.read(block_size)
                    beg = beg_full[:beg_size]
                else:
                    beg = b""
                    beg_trimmed_size = 0

                if end_size > 0:
                    f.seek(-block_size, 2)  # whence = 2: end of the file
                    end_full = f.read(block_size)
                    end_full_orig_size = len(end_full)
                    end_full = end_full.rstrip()
                    end_trimmed_size = end_full_orig_size - len(end_full)
                    if len(end_full) < end_size:
                        # Same as above
                        f.seek(-2 * block_size, 2)  # whence = 2: end of the file
                        end_full = f.read(block_size) + end_full
                    end = end_full[-end_size:]
                else:
                    end = b""
                    end_trimmed_size = 0

                if mid_size > 0:
                    mid_idx = (file_size - beg_trimmed_size - end_trimmed_size) // 2
                    mid_left_idx = mid_idx - mid_size // 2
                    f.seek(mid_left_idx, 0)  # whence = 0: start of the file
                    mid = f.read(mid_size)
                else:
                    mid = b""

            beg_ints = list(map(int, beg))
            mid_ints = list(map(int, mid))
            end_ints = list(map(int, end))

        assert len(beg_ints) == beg_size
        assert len(mid_ints) == mid_size
        assert len(end_ints) == end_size

        return ModelFeatures(beg=beg_ints, mid=mid_ints, end=end_ints)

    def _extract_features_from_bytes(
        self,
        content: bytes,
        beg_size: Optional[int] = None,
        mid_size: Optional[int] = None,
        end_size: Optional[int] = None,
        padding_token: Optional[int] = None,
    ) -> ModelFeatures:
        if beg_size is None:
            beg_size = self._input_sizes["beg"]
        if mid_size is None:
            mid_size = self._input_sizes["mid"]
        if end_size is None:
            end_size = self._input_sizes["end"]
        if padding_token is None:
            padding_token = self._padding_token

        assert beg_size <= 512
        assert mid_size <= 512
        assert end_size <= 512

        content = content.strip()

        if beg_size > 0:
            if beg_size < len(content):
                # we have enough bytes, no need for padding
                beg_ints = list(map(int, content[:beg_size]))
            else:
                padding_size = beg_size - len(content)
                beg_ints = list(map(int, content)) + ([padding_token] * padding_size)
        else:
            beg_ints = []

        if mid_size > 0:
            mid_idx = len(content) // 2
            if mid_size < len(content):
                left_idx = mid_idx - mid_size // 2
                right_idx = mid_idx + mid_size // 2
                if mid_size % 2 != 0:
                    right_idx += 1
                mid_ints = list(map(int, content[left_idx:right_idx]))
            else:
                padding_size = mid_size - len(content)
                left_padding_size = padding_size // 2
                right_padding_size = padding_size // 2
                if padding_size % 2 != 0:
                    right_padding_size += 1

                mid_ints = (
                    ([padding_token] * left_padding_size)
                    + list(map(int, content))
                    + ([padding_token] * right_padding_size)
                )
        else:
            mid_ints = []

        if end_size > 0:
            if len(content) > end_size:
                end_ints = list(map(int, content[-end_size:]))
            else:
                padding_size = end_size - len(content)
                end_ints = ([padding_token] * padding_size) + list(map(int, content))
        else:
            end_ints = []

        return ModelFeatures(beg=beg_ints, mid=mid_ints, end=end_ints)

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
                result = self._get_result_of_small_file(path)
                return result, None

            else:
                file_features = self._extract_features_from_path(path)
                # Check whether we have enough bytes for a meaningful
                # detection, and not just padding.
                if (
                    file_features.beg[self._min_file_size_for_dl - 1]
                    == self._padding_token
                ):
                    # If the n-th token is padding, then it means that,
                    # post-stripping, we do not have enough meaningful
                    # bytes.
                    result = self._get_result_of_small_file(path)
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
            file_features = self._extract_features_from_bytes(content)
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

    def _get_result_of_small_file(self, path: Path) -> MagikaResult:
        content = path.read_bytes()
        return self._get_result_of_few_bytes(content, path)

    def _get_result_of_few_bytes(
        self, content: bytes, path: Path = Path("-")
    ) -> MagikaResult:
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
