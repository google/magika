#!/usr/bin/env python

import json
import pathlib
import subprocess
import dataclasses
import assets_generation.tfjs_config as tfjs_config

# Location of AssetsGeneration.
MAGIKA_AG_ROOT_DIR = pathlib.Path(__file__).resolve().parent.parent
# Location of the Magika repo.
MAGIKA_REPO_ROOT_DIR = MAGIKA_AG_ROOT_DIR.parent
# Location of Magika JS.
MAGIKA_JS_ROOT_DIR = MAGIKA_REPO_ROOT_DIR / "js"
# Location of Magika website.
MAGIKA_WEB_ROOT_DIR = MAGIKA_REPO_ROOT_DIR / "docs"
# Location of the Magika models directory.
MODELS_ROOT_DIR = MAGIKA_AG_ROOT_DIR / "models"
# Current model directory.
MODEL_DIR = [f for f in MODELS_ROOT_DIR.glob("*") if f.is_dir()].pop()
# Model file.
MODEL_TENSORFLOW_FILE_PATH = MODEL_DIR / "model.h5"
# Model config file.
MODEL_CONFIG_FILE_PATH = MODEL_DIR / "model_config.json"
# Thresholds config file.
THRESHOLDS_FILE_PATH = MODEL_DIR / "thresholds.json"
# Content types config file.
CONTENT_TYPES_FILE_PATH = MODEL_DIR / "content_types_config.json"
# Thresholds config file.
CONSTANTS_FILE_PATH = MODEL_DIR / "magika_config.json"
# TensorflowJS model dir.
MODEL_TENSORFLOWJS_DIR = MAGIKA_WEB_ROOT_DIR / 'public' / 'model' 
# TensorflowJS config path.
CONFIG_TENSORFLOWJS_PATH = MODEL_TENSORFLOWJS_DIR / 'config.json'


def print_environment():
    for k, v in globals().items():
        if not k.isupper():
            continue
        print(f"{k}: {v}")


def convert_to_tfjs_model():
    for file in MODEL_TENSORFLOWJS_DIR.glob('*'):
        file.unlink()
    subprocess.check_call([
        'tensorflowjs_converter',
       '--input_format=keras',
    '--output_format=tfjs_graph_model',
    '--saved_model_tags=serve',
    MODEL_TENSORFLOW_FILE_PATH,
    MODEL_TENSORFLOWJS_DIR
    ])


def load_json_file(path):
    with open(path, 'r') as f:
        return json.load(f)
    

def generate_tfjs_config():
    
    model_config = load_json_file(MODEL_CONFIG_FILE_PATH)
    constants = load_json_file(CONSTANTS_FILE_PATH)
    thresholds = load_json_file(THRESHOLDS_FILE_PATH)
    content_types = load_json_file(CONTENT_TYPES_FILE_PATH)
    labels = []
    for label_index, label  in enumerate(model_config["train_dataset_info"]["target_labels_info"]["target_labels_space"]):
        labels.append(tfjs_config.Label(
             name=label,
             threshold=thresholds['thresholds'][label],
             is_text='text' in content_types[label]['tags']
        )          
        )
    config = tfjs_config.Config(
        input_size_beg=model_config['cfg']["input_sizes"]['beg'],
        input_size_mid=model_config['cfg']["input_sizes"]['mid'],
        input_size_end=model_config['cfg']["input_sizes"]['end'],
            min_file_size_for_dl = constants['min_file_size_for_dl'],
    padding_token=constants["padding_token"],
        labels=labels
    )
    json_config = json.dumps(dataclasses.asdict(config), indent=4)
    print(json_config)
    with open(CONFIG_TENSORFLOWJS_PATH, 'w') as f:
        f.write(json_config)
 

def main():
    print_environment()
    #convert_to_tfjs_model()
    generate_tfjs_config()


if __name__ == "__main__":
    main()
