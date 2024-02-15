# KerasTuner

[![](https://github.com/keras-team/keras-tuner/workflows/Tests/badge.svg?branch=master)](https://github.com/keras-team/keras-tuner/actions?query=workflow%3ATests+branch%3Amaster)
[![codecov](https://codecov.io/gh/keras-team/keras-tuner/branch/master/graph/badge.svg)](https://codecov.io/gh/keras-team/keras-tuner)
[![PyPI version](https://badge.fury.io/py/keras-tuner.svg)](https://badge.fury.io/py/keras-tuner)

KerasTuner is an easy-to-use, scalable hyperparameter optimization framework
that solves the pain points of hyperparameter search. Easily configure your
search space with a define-by-run syntax, then leverage one of the available
search algorithms to find the best hyperparameter values for your models.
KerasTuner comes with Bayesian Optimization, Hyperband, and Random Search algorithms
built-in, and is also designed to be easy for researchers to extend in order to
experiment with new search algorithms.

Official Website: [https://keras.io/keras_tuner/](https://keras.io/keras_tuner/)

## Quick links

* [Getting started with KerasTuner](https://keras.io/guides/keras_tuner/getting_started)
* [KerasTuner developer guides](https://keras.io/guides/keras_tuner/)
* [KerasTuner API reference](https://keras.io/api/keras_tuner/)


## Installation

KerasTuner requires **Python 3.8+** and **TensorFlow 2.0+**.

Install the latest release:

```
pip install keras-tuner
```

You can also check out other versions in our
[GitHub repository](https://github.com/keras-team/keras-tuner).


## Quick introduction

Import KerasTuner and TensorFlow:

```python
import keras_tuner
from tensorflow import keras
```

Write a function that creates and returns a Keras model.
Use the `hp` argument to define the hyperparameters during model creation.

```python
def build_model(hp):
  model = keras.Sequential()
  model.add(keras.layers.Dense(
      hp.Choice('units', [8, 16, 32]),
      activation='relu'))
  model.add(keras.layers.Dense(1, activation='relu'))
  model.compile(loss='mse')
  return model
```

Initialize a tuner (here, `RandomSearch`).
We use `objective` to specify the objective to select the best models,
and we use `max_trials` to specify the number of different models to try.

```python
tuner = keras_tuner.RandomSearch(
    build_model,
    objective='val_loss',
    max_trials=5)
```

Start the search and get the best model:

```python
tuner.search(x_train, y_train, epochs=5, validation_data=(x_val, y_val))
best_model = tuner.get_best_models()[0]
```

To learn more about KerasTuner, check out [this starter guide](https://keras.io/guides/keras_tuner/getting_started/).

## Contributing Guide

Please refer to the [CONTRIBUTING.md](https://github.com/keras-team/keras-tuner/blob/master/CONTRIBUTING.md) for the contributing guide.

Thank all the contributors!

[![The contributors](https://raw.githubusercontent.com/keras-team/keras-tuner/master/docs/contributors.svg)](https://github.com/keras-team/keras-tuner/graphs/contributors)

## Community

Ask your questions on our [GitHub Discussions](https://github.com/keras-team/keras-tuner/discussions).

## Citing KerasTuner

If KerasTuner helps your research, we appreciate your citations.
Here is the BibTeX entry:

```bibtex
@misc{omalley2019kerastuner,
	title        = {KerasTuner},
	author       = {O'Malley, Tom and Bursztein, Elie and Long, James and Chollet, Fran\c{c}ois and Jin, Haifeng and Invernizzi, Luca and others},
	year         = 2019,
	howpublished = {\url{https://github.com/keras-team/keras-tuner}}
}
```
