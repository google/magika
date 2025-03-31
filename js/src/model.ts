import * as tf from "@tensorflow/tfjs";
import { GraphModel } from "@tensorflow/tfjs";
import { Config } from "./config.js";
import { ModelPrediction } from "./model-prediction.js";
import { ModelFeatures } from "./module-features.js";

export class Model {
  model?: GraphModel;

  constructor(public config: Config) {}

  async loadUrl(modelURL: string): Promise<void> {
    if (this.model == null) {
      this.model = await tf.loadGraphModel(modelURL);
    }
  }

  async predict(features: ModelFeatures): Promise<ModelPrediction> {
    if (this.model == null) {
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
    const scores = modelOutput.dataSync();
    const maxScore = scores[maxScoreIndex];
    maxScoreIndexTensor.dispose();
    modelInput.dispose();
    modelOutput.dispose();

    let maxScoreLabel = this.config.target_labels_space[maxScoreIndex];

    return { label: maxScoreLabel, score: maxScore, scores: scores };
  }
}
