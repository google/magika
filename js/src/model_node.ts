import * as tfn from '@tensorflow/tfjs-node';
import * as tf from '@tensorflow/tfjs';
import {Model} from './model.js';

export class ModelNode extends Model {

    async loadFile(modelPath: string): Promise<void> {
        if (this.model == null) {
            const handler = tfn.io.fileSystem(modelPath);
            this.model = await tf.loadGraphModel(handler);
        }
    }
}
