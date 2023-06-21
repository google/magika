import json
import os
from pathlib import Path
import logging
import requests
import subprocess
import time
from typing import List
from tqdm.auto import tqdm
import sys

import numpy as np
import treelite_runtime

from magician_ng.content_types import ContentTypesManager, ContentType

from magician_ng import get_logger

l = get_logger()


# TODO move these into a config file
GC_BUCKET_URL_PREFIX = 'https://storage.googleapis.com/magician-models/'
GBT_MODEL_HASH = '0c76e25287731933b1c082ff73537f58de967e36107aab8b0b6ef903c121a5e2'
DL_MODEL_HASH = '5fe074fed61f81413f4c1cbca86147ee51504cfcf31109720ceaa6dfb032e004'


class Magician():
    MODES = ['dl-only-if-needed', 'gbt-only', 'dl-only', 'both']
    COMBINE_POLICY = [
        'prioritize-dl',  # always pick DL, when available
        'highest',  # pick the one with the highest score
    ]
    DEFAULT_GBT_T = 0.9
    DEFAULT_DL_T = 0.9

    def __init__(
        self,
        gbt_model_dir=None,
        dl_model_dir=None,
        gbt_t=DEFAULT_GBT_T,
        dl_t=DEFAULT_DL_T,
        mode=MODES[0],
        combine_policy=COMBINE_POLICY[0],
        debug=False):

        if gbt_model_dir is None:
            if d := os.environ.get('DEFAULT_MAGICIAN_GBT_MODEL_DIR'):
                gbt_model_dir = Path(d)
                if not gbt_model_dir.is_dir():
                    l.error(f'DEFAULT_MAGICIAN_GBT_MODEL_DIR does not point to a valid directory')
                    sys.exit(1)
            else:
                gbt_model_dir = Path(f'{os.environ["HOME"]}/.cache/magician/models/gbt')
        if dl_model_dir is None:
            if d := os.environ.get('DEFAULT_MAGICIAN_DL_MODEL_DIR'):
                dl_model_dir = Path(d)
                if not dl_model_dir.is_dir():
                    l.error(f'DEFAULT_MAGICIAN_DL_MODEL_DIR does not point to a valid directory')
                    sys.exit(1)
            else:
                dl_model_dir = Path(f'{os.environ["HOME"]}/.cache/magician/models/dl')

        self.gbt_model_dir = gbt_model_dir
        self.dl_model_dir = dl_model_dir

        assert self.gbt_model_dir is not None
        assert self.dl_model_dir is not None

        # deal with the GBT model
        self.gbt_model = None
        self.gbt_model_path = self.gbt_model_dir / 'model.so'
        self.gbt_model_config_path = self.gbt_model_dir / 'model_config.json'

        if not self.gbt_model_path.is_file() or not self.gbt_model_config_path.is_file():
            self.gbt_model_dir.mkdir(parents=True, exist_ok=True)
            model_url = GC_BUCKET_URL_PREFIX + f'model-{GBT_MODEL_HASH}.so'
            l.info(f'Downloading GBT model {model_url} => {self.gbt_model_path}')
            with open(self.gbt_model_path, 'wb') as f:
                f.write(requests.get(model_url).content)
            model_config_url = GC_BUCKET_URL_PREFIX + f'model_config-{GBT_MODEL_HASH}.json'
            l.info(f'Downloading GBT model config {model_config_url} => {self.gbt_model_config_path}')
            with open(self.gbt_model_config_path, 'wb') as f:
                f.write(requests.get(model_config_url).content)

        self.gbt_model_config = json.loads(self.gbt_model_config_path.read_text())
        self.gbt_input_sizes = {
            'beg': self.gbt_model_config['cli_options']['input_sizes']['beg'],
            'mid': self.gbt_model_config['cli_options']['input_sizes']['mid'],
            'end': self.gbt_model_config['cli_options']['input_sizes']['end'],
        }
        # FIXME temporarily using target_labels vs. target_labels_space until we fix the related bug
        self.gbt_target_labels_space = np.array(self.gbt_model_config['dataset_info']['target_labels'])


        # deal with the DL model
        self.dl_model = None
        self.dl_model_path = dl_model_dir / 'model.h5'
        self.dl_model_config_path = dl_model_dir / 'model_config.json'

        if not self.dl_model_path.is_file() or not self.dl_model_config_path.is_file():
            self.dl_model_dir.mkdir(parents=True, exist_ok=True)
            model_url = GC_BUCKET_URL_PREFIX + f'model-{DL_MODEL_HASH}.h5'
            l.info(f'Downloading DL model {model_url} => {self.dl_model_path}')
            with open(self.dl_model_path, 'wb') as f:
                f.write(requests.get(model_url).content)
            model_config_url = GC_BUCKET_URL_PREFIX + f'model_config-{DL_MODEL_HASH}.json'
            l.info(f'Downloading GBT model config {model_config_url} => {self.dl_model_config_path}')
            with open(self.dl_model_config_path, 'wb') as f:
                f.write(requests.get(model_config_url).content)

        self.dl_model_config = json.loads(self.dl_model_config_path.read_text())
        self.dl_input_sizes = {
            'beg': self.dl_model_config['cfg']['input_sizes']['beg'],
            'mid': self.dl_model_config['cfg']['input_sizes']['mid'],
            'end': self.dl_model_config['cfg']['input_sizes']['end'],
        }
        self.dl_target_labels_space = np.array(self.dl_model_config['train_dataset_info']['target_labels_info']['target_labels_space'])

        self.gbt_t = gbt_t
        self.dl_t = dl_t

        assert mode in Magician.MODES
        assert combine_policy in Magician.COMBINE_POLICY
        self.mode = mode
        self.combine_policy = combine_policy

        self.ctm = ContentTypesManager()

        if self.mode == 'both' or self.mode == 'dl-only-if-needed':
            self.use_gbt = True
            self.use_dl = True
        elif self.mode == 'gbt-only':
            self.use_gbt = True
            self.use_dl = False
        elif self.mode == 'dl-only':
            self.use_gbt = False
            self.use_dl = True
        else:
            raise Exception(f'Mode "{self.mode}" is not supported')

        self.dl_only_if_needed = (self.mode == 'dl-only-if-needed')

        self.debug = debug

        if self.debug:
            l.setLevel(logging.DEBUG)

    def _load_gbt_model(self):
        if self.gbt_model is None:
            start_time = time.time()
            self.gbt_model = treelite_runtime.Predictor(str(self.gbt_model_path), verbose=self.debug)
            elapsed_time = time.time() - start_time
            l.debug(f'GBT model "{self.gbt_model_path}" loaded in {elapsed_time:.03f} seconds')

    def _load_dl_model(self):
        import tensorflow as tf
        if self.dl_model is None:
            start_time = time.time()
            self.dl_model = tf.keras.models.load_model(str(self.dl_model_path))
            elapsed_time = time.time() - start_time
            l.debug(f'DL model "{self.dl_model_path}" loaded in {elapsed_time:.03f} seconds')

    def get_content_types(self, paths: List[Path]):
        # extract the features and store them in a dict, keyed by path
        start_time = time.time()
        paths = sorted(paths)
        features = []
        l.debug(f'Extracting features for {len(paths)} samples')
        for path in tqdm(paths):
            features.append((path, self.extract_features(path)))
        elapsed_time = time.time() - start_time
        l.debug(f'Raw features extracted in {elapsed_time:.03f} seconds')

        predictions = {}
        for path in paths:
            predictions[str(path)] = {'path': str(path)}

        if self.use_gbt:
            # To take advantage of batching, we first obtain all GBT predictions
            self._load_gbt_model()
            X = self.prepare_gbt_input(features)
            raw_preds = self.get_gbt_raw_predictions(X)
            top_preds_idxs = np.argmax(raw_preds, axis=1)
            preds_content_types_labels = self.gbt_target_labels_space[top_preds_idxs]
            scores = np.max(raw_preds, axis=1)
            features_for_dl = []
            for (path, fs), ct_label, score in zip(features, preds_content_types_labels, scores):
                predictions[str(path)].update({
                    'gbt': {
                        'ct_label': ct_label,
                        'score': float(score),
                    }
                })
                if self.use_dl:
                    if self.dl_only_if_needed and float(score) < self.gbt_t:
                        l.debug(f'adding {path} for DL scanning')
                        features_for_dl.append((path, fs))

        if self.use_dl and not self.dl_only_if_needed:
            # scan all files with DL
            features_for_dl = features

        if self.use_dl and len(features_for_dl) > 0:
            self._load_dl_model()
            X = self.prepare_dl_input(features_for_dl)
            raw_preds = self.get_dl_raw_predictions(X)
            top_preds_idxs = np.argmax(raw_preds, axis=1)
            preds_content_types_labels = self.dl_target_labels_space[top_preds_idxs]
            scores = np.max(raw_preds, axis=1)
            for (path, _), ct_label, score in zip(features_for_dl, preds_content_types_labels, scores):
                l.debug(f'Adding DL result for {path}')
                predictions[str(path)].update({
                    'dl': {
                        'ct_label': ct_label,
                        'score': float(score),
                    }
                })

        # determine output content type
        for path, preds in predictions.items():
            if gbt_pred := preds.get('gbt'):
                gbt_output_ct_label, gbt_output_score = self.get_output_ct_label(
                    path,
                    gbt_pred['ct_label'],
                    gbt_pred['score'],
                    self.gbt_t,
                )
            else:
                gbt_output_ct_label, gbt_output_score = None, -1

            if dl_pred := preds.get('dl'):
                dl_output_ct_label, dl_output_score = self.get_output_ct_label(
                    path,
                    dl_pred['ct_label'],
                    dl_pred['score'],
                    self.dl_t,
                )
            else:
                dl_output_ct_label, dl_output_score = None, -1

            assert gbt_output_ct_label or dl_output_ct_label
            if gbt_output_ct_label and not dl_output_ct_label:
                output_ct_label = gbt_output_ct_label
                output_score = gbt_output_score
            elif not gbt_output_ct_label and dl_output_ct_label:
                output_ct_label = dl_output_ct_label
                output_score = dl_output_score
            else:
                # we need to combine the two output
                if self.combine_policy == 'highest':
                    if gbt_output_score > dl_output_score:
                        output_ct_label = gbt_output_ct_label
                        output_score = gbt_output_score
                    else:
                        output_ct_label = dl_output_ct_label
                        output_score = dl_output_score
                elif self.combine_policy == 'prioritize-dl':
                    output_ct_label = dl_output_ct_label
                    output_score = dl_output_score

            if Path(path).stat().st_size == 0:
                output_ct_label = ContentType.EMPTY
                output_score = 1

            predictions[str(path)].update({
                'output': {
                    'ct_label': output_ct_label,
                    'score': output_score,
                }
            })

        return predictions

    def get_content_type(self, path: Path):
        return self.get_content_types([path])[0]

    def get_output_ct_label(self, path, ct_label, score, threshold):
        if score >= threshold:
            output_ct_label = ct_label
        else:
            if 'text' in self.ctm.get(ct_label).tags:
                output_ct_label = ContentType.GENERIC_TEXT
            else:
                output_ct_label = ContentType.UNKNOWN
        output_score = score

        # if output_ct_label == 'cdf':
        #     output_ct_label = self.predict_cdf(path)
        # elif output_ct_label == 'zip':
        #     output_ct_label = self.predict_zip(path)

        output_score = float(output_score)

        return output_ct_label, output_score

    def extract_features(self, file_path: Path):
        file_size = file_path.stat().st_size

        with open(file_path, 'rb') as f:
            beg_size = max(self.gbt_input_sizes['beg'], self.dl_input_sizes['beg'])
            mid_size = max(self.gbt_input_sizes['mid'], self.dl_input_sizes['mid'])
            end_size = max(self.gbt_input_sizes['end'], self.dl_input_sizes['end'])

            assert mid_size == 0, 'mid != 0 is unsupported at the moment'

            beg_to_read = min(beg_size, file_size)
            end_to_read = min(end_size, file_size)

            beg = f.read(beg_to_read)
            beg_ints = list(map(int, beg)) + ([0] * (beg_size - beg_to_read))

            mid_ints = [0] * mid_size

            f.seek(-end_to_read, 2)
            end = f.read(end_to_read)
            end_ints = ([0] * (end_size - end_to_read)) + list(map(int, end))

        return {
            'beg': beg_ints,
            'mid': mid_ints,
            'end': end_ints,
        }

    def prepare_gbt_input(self, features: List):
        """
        Prepare input in a format suitable for GBT.

        Features is a list of (path, {'beg': XXX, ...}) tuples.

        The order of the list is not altered in any way.
        """

        assert type(features) is list

        start_time = time.time()
        X = []
        for _, fs in features:
            assert self.gbt_input_sizes['mid'] == 0
            X.append(fs['beg'][:self.gbt_input_sizes['beg']] + fs['mid'][:self.gbt_input_sizes['mid']] + fs['end'][-self.gbt_input_sizes['end']:])
        X = treelite_runtime.DMatrix(data=np.array(X), dtype='float32')
        elapsed_time = time.time() - start_time
        l.debug(f'GBT input prepared in {elapsed_time:.03f} seconds')
        return X

    def prepare_dl_input(self, features: List):
        """
        Prepare input in a format suitable for DL.

        Features is a list of (path, {'beg': XXX, ...}) tuples.

        The order of the list is not altered in any way.
        """

        assert type(features) is list

        start_time = time.time()
        X = {}
        if self.dl_input_sizes['beg'] > 0:
            X['beg'] = []
            for _, fs in features:
                X['beg'].append(fs['beg'][:self.dl_input_sizes['beg']])
            X['beg'] = np.array(X['beg'])
        assert self.dl_input_sizes['mid'] == 0
        if self.dl_input_sizes['end'] > 0:
            X['end'] = []
            for _, fs in features:
                X['end'].append(fs['end'][-self.dl_input_sizes['end']:])
            X['end'] = np.array(X['end'])
        elapsed_time = time.time() - start_time
        l.debug(f'DL input prepared in {elapsed_time:.03f} seconds')
        return X

    def get_gbt_raw_predictions(self, X):
        start_time = time.time()
        raw_predictions = self.gbt_model.predict(X)
        elapsed_time = time.time() - start_time
        l.debug(f'GBT raw prediction in {elapsed_time:.03f} seconds')
        return raw_predictions

    def get_dl_raw_predictions(self, X):
        start_time = time.time()
        raw_predictions = self.dl_model(X)
        elapsed_time = time.time() - start_time
        l.debug(f'DL raw prediction in {elapsed_time:.03f} seconds')
        return raw_predictions
