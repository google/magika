import * as tf from "@tensorflow/tfjs";
import { GraphModel } from "@tensorflow/tfjs";
import { ModelConfig } from "./model-config";
import { ModelPrediction } from "./model-prediction";
import { ModelFeatures } from "./model-features";
import { ContentTypeLabel } from "./content-type-label";

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
