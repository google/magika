// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

import * as tf from "@tensorflow/tfjs";
import { GraphModel } from "@tensorflow/tfjs";
import { ModelConfig } from "./model-config.js";
import { ModelPrediction } from "./model-prediction.js";
import { ModelFeatures } from "./model-features.js";
import { ContentTypeLabel } from "./content-type-label.js";

export class Model {
  model?: GraphModel;

  constructor(public model_config: ModelConfig) {}

  async loadUrl(modelURL: string): Promise<void> {
    if (!this.model) {
      this.model = await tf.loadGraphModel(modelURL);
    }
  }

  async predict(features: ModelFeatures): Promise<ModelPrediction> {
    if (!this.model) {
      throw new Error("model has not been loaded");
    }
    let features_array = features.toArray();
    const modelInput = tf.tensor(
      [features_array],
      [1, features_array.length],
      "int32",
    );
    const modelOutput = tf.squeeze(
      (await this.model.executeAsync(modelInput)) as any,
    );
    const maxScoreIndexTensor = tf.argMax(modelOutput);
    const maxScoreIndex = maxScoreIndexTensor.dataSync()[0];
    const rawScores = modelOutput.dataSync();
    maxScoreIndexTensor.dispose();
    modelInput.dispose();
    modelOutput.dispose();

    const maxScoreLabel = this.model_config.target_labels_space[maxScoreIndex];
    const maxScore = rawScores[maxScoreIndex];

    if (rawScores.length != this.model_config.target_labels_space.length) {
      throw new Error(
        `Assertion failed: Expected rawScores.length (${rawScores.length}) to have the same length of the targets_label_space (${this.model_config.target_labels_space.length})`,
      );
    }

    let scores_map: Partial<Record<ContentTypeLabel, number>> = {};
    for (let i = 0; i < rawScores.length; i++) {
      const label: ContentTypeLabel = this.model_config.target_labels_space[i];
      const score: number = rawScores[i];
      scores_map[label] = score;
    }

    return { label: maxScoreLabel, score: maxScore, scores_map: scores_map };
  }
}
