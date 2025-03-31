import { DataTypeMap, NumericDataType } from "@tensorflow/tfjs";
import { ContentTypeLabel } from "./content-type-label.js";

export interface ModelPrediction {
  label: ContentTypeLabel;
  score: number;
  scores: DataTypeMap[NumericDataType];
}
