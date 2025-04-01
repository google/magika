import { ContentTypeLabel } from "./content-type-label.js";

export interface ModelPrediction {
  label: ContentTypeLabel;
  score: number;
  scores_map: Partial<Record<ContentTypeLabel, number>>;
}
