import * as tf from "@tensorflow/tfjs";
import * as tfn from "@tensorflow/tfjs-node";
import { Model } from "./model";

export class ModelNode extends Model {
  async loadFile(modelPath: string): Promise<void> {
    if (!this.model) {
      const handler = tfn.io.fileSystem(modelPath);
      this.model = await tf.loadGraphModel(handler);
    }
  }
}
