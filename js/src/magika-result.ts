import { MagikaPrediction } from "./magika-prediction";
import { Status } from "./status";

export interface MagikaResult {
  path: string;
  status: Status;
  prediction: MagikaPrediction;
}
