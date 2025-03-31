import * as fs from "fs/promises";
import { ContentType } from "./contentType.js";

interface ConfigLabel {
  name: ContentType;
  threshold: number;
  is_text: boolean;
}

export class Config {
  loaded: boolean = false;
  labels: ConfigLabel[] = [];
  minFileSizeForDl: number = 0;
  paddingToken: number = 0;
  begBytes: number = 0;
  midBytes: number = 0;
  endBytes: number = 0;
  async loadUrl(configURL: string): Promise<void> {
    if (this.loaded) {
      return;
    }
    const config = (await (await fetch(configURL)).json()) as Record<
      string,
      any
    >;
    this.setConfig(config);
    this.loaded = true;
  }

  async loadFile(configPath: string): Promise<void> {
    if (this.loaded) {
      return;
    }
    const config = JSON.parse((await fs.readFile(configPath)).toString());
    this.setConfig(config);
    this.loaded = true;
  }

  private setConfig(config: Record<string, any>): void {
    this.minFileSizeForDl = config.min_file_size_for_dl;
    this.paddingToken = config.padding_token;
    this.labels = [];
    for (const label of config.target_labels_space) {
      this.labels.push({
        name: label,
        threshold:
          label in config.thresholds
            ? config.thresholds[label]
            : config.medium_confidence_threshold,
        // This is clearly a bug - @reyammer, where is this stored now?
        is_text: false,
      });
    }
    this.begBytes = config.beg_size;
    this.midBytes = config.mid_size;
    this.endBytes = config.end_size;
  }
}
