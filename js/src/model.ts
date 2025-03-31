import * as tf from "@tensorflow/tfjs";
import { GraphModel, DataTypeMap, NumericDataType } from "@tensorflow/tfjs";
import { Config } from "./config.js";
import { ContentType } from "./contentType.js";

export interface ContentTypeInfo {
  label: string;
  is_text: boolean;
}

export interface ModelProdiction {
  index: number;
  scores: DataTypeMap[NumericDataType];
}

export interface ModelResult {
  score: number;
  label: ContentType;
}

export interface ModelResultScores extends ModelResult {
  scores: DataTypeMap[NumericDataType];
}

export interface ModelResultLabels extends ModelResult {
  labels: Record<string, number>;
}

export class Model {
  model?: GraphModel;

  constructor(
    public config: Config,
    public ct_infos: Record<string, ContentTypeInfo>,
  ) {}

  async loadUrl(modelURL: string): Promise<void> {
    if (this.model == null) {
      this.model = await tf.loadGraphModel(modelURL);
    }
  }

  async predict(features: number[]): Promise<ModelProdiction> {
    if (this.model == null) {
      throw new Error("model has not been loaded");
    }
    const modelInput = tf.tensor([features], [1, features.length], "int32");
    const modelOutput = tf.squeeze(
      (await this.model.executeAsync(modelInput)) as any,
    );
    const maxProbability = tf.argMax(modelOutput);
    const index = maxProbability.dataSync()[0];
    const scores = modelOutput.dataSync();
    maxProbability.dispose();
    modelInput.dispose();
    modelOutput.dispose();
    return { index: index, scores: scores };
  }

  generateResultFromPrediction(prediction: ModelProdiction): ModelResultScores {
    const score = prediction.scores[prediction.index];
    const labelConfig = this.config.labels[prediction.index];
    if (score >= labelConfig.threshold) {
      return {
        score: score,
        label: labelConfig.name,
        scores: prediction.scores,
      };
    }
    let generic_type: ContentType;
    if (this.ct_infos[labelConfig.name] || null) {
      if (this.ct_infos[labelConfig.name].is_text) {
        generic_type = ContentType.GENERIC_TEXT;
      } else {
        generic_type = ContentType.UNKNOWN;
      }
    } else {
      generic_type = ContentType.UNKNOWN;
    }

    return { score: score, label: generic_type, scores: prediction.scores };
  }
}
