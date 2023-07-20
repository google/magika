import json
import os
from pathlib import Path
import logging
import requests
import subprocess
import time
from typing import List, Dict, Tuple, Any, Union, Optional
from tqdm.auto import tqdm
import sys

import numpy as np
import onnxruntime as rt

from magician_ng.content_types import ContentTypesManager, ContentType
from magician_ng import get_logger

l = get_logger()


# TODO move these into a config file
GC_BUCKET_URL_PREFIX = 'https://storage.googleapis.com/magician-ng/'
MODEL_HASH = '0e9ecda1a7f66430f3ffe7f14423b388c67b140679c46c74d54e17fd1214c67a'


class Magician():
    def __init__(
        self,
        model_dir: Optional[Path] = None,
        threshold: float = 0.0,
        verbose: bool = False,
        debug: bool = False):

        self.verbose = verbose
        self.debug = debug

        if verbose:
            l.setLevel(logging.INFO)
        if debug:
            l.setLevel(logging.DEBUG)

        if model_dir is None:
            if os.environ.get('MAGICIAN_MODEL_DIR') not in [None, '']:
                model_dir = Path(os.environ.get('MAGICIAN_MODEL_DIR'))

        if model_dir is not None:
            self.model_dir = model_dir
            self.model_path = self.model_dir / 'model.onnx'
            self.model_config_path = self.model_dir / 'model_config.json'

            # check that we have both the model and the config
            if not self.model_dir.is_dir():
                raise Exception(f'User-specified model dir "{str(self.model_dir)}" not found')
            if not self.model_path.is_file():
                raise Exception(f'Model not found at "{str(self.model_path)}"')
            if not self.model_config_path.is_file():
                raise Exception(f'Model config not found at "{str(self.model_config)}"')
        else:
            # use default model_dir, and download if not found
            self.model_dir = Path(f'{os.environ["HOME"]}/.cache/magician/model/')
            self.model_path = self.model_dir / 'model.onnx'
            self.model_config_path = self.model_dir / 'model_config.json'
            if not self.model_path.is_file() or not self.model_config_path.is_file():
                self.model_dir.mkdir(parents=True, exist_ok=True)
                model_url = GC_BUCKET_URL_PREFIX + f'model-{MODEL_HASH}.onnx'
                l.info(f'Downloading DL model {model_url} => {self.model_path}')
                with open(self.model_path, 'wb') as f:
                    f.write(requests.get(model_url).content)
                model_config_url = GC_BUCKET_URL_PREFIX + f'model_config-{MODEL_HASH}.json'
                l.info(f'Downloading DL model config {model_config_url} => {self.model_config_path}')
                with open(self.model_config_path, 'wb') as f:
                    f.write(requests.get(model_config_url).content)

        assert self.model_dir.is_dir()
        assert self.model_path.is_file()
        assert self.model_config_path.is_file()

        self.model_config = json.loads(self.model_config_path.read_text())

        self.input_sizes = {
            'beg': self.model_config['cfg']['input_sizes']['beg'],
            'mid': self.model_config['cfg']['input_sizes']['mid'],
            'end': self.model_config['cfg']['input_sizes']['end'],
        }
        self.target_labels_space = np.array(self.model_config['train_dataset_info']['target_labels_info']['target_labels_space'])

        self.threshold = threshold

        self.ctm = ContentTypesManager()
        self.onnx_session = None

    def _load_model(self):
        if self.onnx_session is None:
            start_time = time.time()
            self.onnx_session = rt.InferenceSession(self.model_path)
            elapsed_time = time.time() - start_time
            l.debug(f'ONNX DL model "{self.model_path}" loaded in {elapsed_time:.03f} seconds')

    def get_content_types(self, paths: List[Path]) -> Dict[str, Any]:
        # extract the features and store them in a dict, keyed by path
        start_time = time.time()
        beg_size = self.input_sizes['beg']
        mid_size = self.input_sizes['mid']
        end_size = self.input_sizes['end']
        paths = sorted(paths)
        features = []
        l.debug(f'Extracting features for {len(paths)} samples')
        for path in tqdm(paths, disable=not (self.verbose or self.debug)):
            features.append((path, self.extract_features(path, beg_size, mid_size, end_size)))
        elapsed_time = time.time() - start_time
        l.debug(f'Raw features extracted in {elapsed_time:.03f} seconds')

        predictions: Dict[str, Any] = {}
        for path in paths:
            predictions[str(path)] = {'path': str(path)}

        if len(features) > 0:
            self._load_model()
            X = self.prepare_input(features)
            raw_preds = self.get_raw_predictions(X)
            top_preds_idxs = np.argmax(raw_preds, axis=1)
            preds_content_types_labels = self.target_labels_space[top_preds_idxs]
            scores = np.max(raw_preds, axis=1)
            for (path, _), ct_label, score in zip(features, preds_content_types_labels, scores):
                # In additional to the content type label from the DL model, we
                # also allow for other logic to overwrite such result. For
                # debugging and information purposes, the JSON output stores
                # both the raw DL model output and the final output we return to
                # the user.
                output_ct_label, output_score = self.get_output_ct_label(
                    path,
                    ct_label,
                    score,
                    self.threshold,
                )
                l.debug(f'Adding DL result for {path}')
                predictions[str(path)].update({
                    'dl': {
                        'ct_label': ct_label,
                        'score': float(score),
                    },
                    'output': {
                        'ct_label': output_ct_label,
                        'score': float(output_score),
                    }
                })

        return predictions

    def get_content_type(self, path: Path) -> Dict[str, Dict[str, Union[int, float]]]:
        return self.get_content_types([path])[str(path)]

    def get_output_ct_label(self, path: Path, ct_label: str, score: float, threshold: float) -> Tuple[str, float]:
        if path.stat().st_size == 0:
            output_ct_label = ContentType.EMPTY
            output_score = 1.0
        elif score >= threshold:
            output_ct_label = ct_label
            output_score = score
        else:
            if 'text' in self.ctm.get(ct_label).tags:
                output_ct_label = ContentType.GENERIC_TEXT
                output_score = score
            else:
                output_ct_label = ContentType.UNKNOWN
                output_score = score

        return output_ct_label, output_score

    def extract_features(self, file_path: Path, beg_size: int, mid_size: int, end_size: int) -> Dict[str, List[int]]:
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

        with open(file_path, 'rb') as f:
            content = f.read()

        content = content.strip()

        if beg_size > 0:
            if beg_size < len(content):
                beg = content[:beg_size]
            else:
                padding_size = beg_size - len(content)
                beg = content + (b'\x00' * padding_size)
        else:
            beg = b''
        # convert to ints and, if needed, add padding
        beg_ints = list(map(int, beg))
        assert len(beg_ints) == beg_size

        if mid_size > 0:
            mid_idx = len(content) // 2
            if mid_size < len(content):
                left_idx = mid_idx - mid_size // 2
                right_idx = mid_idx + mid_size // 2
                if mid_size % 2 != 0:
                    right_idx += 1
                mid = content[left_idx:right_idx]
            else:
                padding_size = mid_size - len(content)
                left_padding_size = padding_size // 2
                right_padding_size = padding_size // 2
                if padding_size % 2 != 0:
                    right_padding_size += 1
                left_padding = b'\x00' * left_padding_size
                right_padding = b'\x00' * right_padding_size
                mid = left_padding + content + right_padding
        else:
            mid = b''
        mid_ints = list(map(int, mid))
        assert len(mid_ints) == mid_size

        if end_size > 0:
            if len(content) > end_size:
                end = content[-end_size:]
            else:
                padding_size = end_size - len(content)
                end = (b'\x00' * padding_size) + content
        else:
            end = b''
        end_ints = list(map(int, end))
        assert len(end_ints) == end_size

        return {
            'beg': beg_ints,
            'mid': mid_ints,
            'end': end_ints,
        }

    def prepare_input(self, features: List):
        """
        Prepare input in a format suitable for DL.

        Features is a list of (path, {'beg': [...], ...}) tuples.

        Output format depends on the dataset_format specified in the model
        config.

        The order of the list is not altered in any way.
        """

        assert type(features) is list

        dataset_format = self.model_config['train_dataset_info']['dataset_format']
        if dataset_format == 'int/one-hot':
            # TODO: deprecate this dataset format
            start_time = time.time()
            X: Dict[str, Any] = {}
            if self.input_sizes['beg'] > 0:
                X['beg'] = []
                for _, fs in features:
                    X['beg'].append(fs['beg'])
                X['beg'] = np.array(X['beg']).astype(np.float32)
            if self.input_sizes['mid'] > 0:
                X['mid'] = []
                for _, fs in features:
                    X['mid'].append(fs['mid'])
                X['mid'] = np.array(X['mid']).astype(np.float32)
            if self.input_sizes['end'] > 0:
                X['end'] = []
                for _, fs in features:
                    X['end'].append(fs['end'])
                X['end'] = np.array(X['end']).astype(np.float32)
            elapsed_time = time.time() - start_time
        elif dataset_format == 'int-concat/one-hot':
            start_time = time.time()
            X_bytes = []
            for _, fs in features:
                sample_bytes = []
                if self.input_sizes['beg'] > 0:
                    sample_bytes.extend(fs['beg'][:self.input_sizes['beg']])
                if self.input_sizes['mid'] > 0:
                    sample_bytes.extend(fs['mid'][:self.input_sizes['mid']])
                if self.input_sizes['end'] > 0:
                    sample_bytes.extend(fs['end'][-self.input_sizes['end']:])
                X_bytes.append(sample_bytes)
            X = {'bytes': np.array(X_bytes).astype(np.float32)}
            elapsed_time = time.time() - start_time
        else:
            raise Exception(f'Dataset format "{dataset_format}" not supported')
        l.debug(f'DL input prepared in {elapsed_time:.03f} seconds')
        return X

    def get_raw_predictions(self, X):
        assert self.onnx_session is not None
        start_time = time.time()
        raw_predictions = self.onnx_session.run(['target_label'], X)[0]
        elapsed_time = time.time() - start_time
        l.debug(f'DL raw prediction in {elapsed_time:.03f} seconds')
        return raw_predictions
