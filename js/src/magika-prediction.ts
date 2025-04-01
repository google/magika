import { ContentTypeInfo } from "./content-type-info";
import { ContentTypeLabel } from "./content-type-label";
import { OverwriteReason } from "./overwrite-reason";

export interface MagikaPrediction {
  dl: ContentTypeInfo;
  output: ContentTypeInfo;
  score: number;
  overwrite_reason: OverwriteReason;
  scores_map?: Partial<Record<ContentTypeLabel, number>>;
}
