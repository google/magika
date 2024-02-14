import * as tf from "@tensorflow/tfjs";

export  class Magika {
  async load(modelUrlOrPath, configUrlOrPath) {
    await Promise.all([this.loadModel(modelUrlOrPath), this.loadConfig(configUrlOrPath)]);
  }

  async loadModel(modelUrlOrPath) {
    if (this.model) return;
    this.model = await tf.loadGraphModel(modelUrlOrPath);
  }

  async loadConfig(configUrlOrPath) {
    if (this.begBytes) return;
    const config = await (await fetch(configUrlOrPath)).json();
    this.begBytes = config["cfg"]["input_sizes"]["beg"];
    this.midBytes = config["cfg"]["input_sizes"]["mid"];
    this.endBytes = config["cfg"]["input_sizes"]["end"];
    this.extractSize =
      this.begBytes > 0
        ? this.begBytes
        : this.midBytes > 0
          ? this.midBytes
          : this.endBytes;
    this.paddingToken = 256;
    this.targetLabels =
      config["train_dataset_info"]["target_labels_info"]["target_labels_space"];
  }

  async extractFeaturesFromFile(fileBytes) {
    fileBytes = fileBytes.trim().split("");
    let beg, mid, end;
    if (fileBytes.length > this.extractSize) {
      beg = fileBytes
        .slice(0, this.begBytes)
        .map((char) => parseFloat(char.charCodeAt(0)));
      mid = fileBytes
        .slice(
          fileBytes.length / 2 - this.midBytes / 2,
          fileBytes.length / 2 + this.midBytes / 2,
        )
        .map((char) => parseFloat(char.charCodeAt(0)));
      end = fileBytes
        .slice(fileBytes.length - this.endBytes, fileBytes.length)
        .map((char) => parseFloat(char.charCodeAt(0)));
    } else {
      const mappedData = fileBytes.map((char) =>
        parseFloat(char.charCodeAt(0)),
      );
      const paddingCount = this.extractSize - fileBytes.length;
      console.log("p", paddingCount, this.extractSize, fileBytes.length);
      beg = mappedData.concat(new Array(paddingCount).fill(this.paddingToken));
      mid = mappedData.concat(
        new Array(parseInt(paddingCount / 2)).fill(this.paddingToken),
      );
      mid = new Array(this.extractSize - mid.length)
        .fill(paddingCount)
        .concat(mid);
      end = new Array(paddingCount).fill(this.paddingToken).concat(mappedData);
    }
    return beg.concat(mid).concat(end);
  }

  async predictFromFeatures(fileFeatures) {
    const modelInput = tf.tensor([fileFeatures]);
    const modelOutput = tf.squeeze(this.model.predict(modelInput));
    const labels = this.predictionToLabels(modelOutput);
    modelInput.dispose();
    modelOutput.dispose();
    return labels;
  }

  predictionToLabels(modelOutput) {
    const startedAt= performance.now()
    const maxProbability = tf.argMax(modelOutput);
    const labelIndex = maxProbability.dataSync()[0];
    maxProbability.dispose();
    const probabilities = modelOutput.dataSync();
    const labelProbability = probabilities[labelIndex];
    const labels = this.targetLabels;
    const label = labels[labelIndex];
    const completedAt = performance.now()
    // Needs thresholding.
    return {
      label,
      labelProbability,
      probabilities,
      labels,
      inferenceTime: completedAt - startedAt
    };
  }
}
