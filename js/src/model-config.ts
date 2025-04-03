import * as fs from "fs/promises";
import { ContentTypeLabel } from "./content-type-label";

export class ModelConfig {
  beg_size: number = 0;
  mid_size: number = 0;
  end_size: number = 0;
  use_inputs_at_offsets: boolean = false;
  medium_confidence_threshold: number = 0;
  min_file_size_for_dl: number = 0;
  padding_token: number = -1;
  block_size: number = 0;
  target_labels_space: ContentTypeLabel[] = [];
  thresholds: Partial<Record<ContentTypeLabel, number>> = {};
  overwrite_map: Partial<Record<ContentTypeLabel, ContentTypeLabel>> = {};
  loaded: boolean = false;

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
    this.beg_size = config.beg_size;
    this.mid_size = config.mid_size;
    this.end_size = config.end_size;
    this.use_inputs_at_offsets = config.use_inputs_at_offsets;
    this.medium_confidence_threshold = config.medium_confidence_threshold;
    this.min_file_size_for_dl = config.min_file_size_for_dl;
    this.padding_token = config.padding_token;
    this.block_size = config.block_size;
    this.target_labels_space = [];
    for (const label of config.target_labels_space as string[]) {
      this.target_labels_space.push(label as ContentTypeLabel);
    }
    for (const [label, th] of Object.entries(
      config.thresholds as Record<ContentTypeLabel, number>,
    )) {
      this.thresholds[label as ContentTypeLabel] = th;
    }
    for (const [label, target_label] of Object.entries(
      config.overwrite_map as Record<ContentTypeLabel, ContentTypeLabel>,
    )) {
      this.overwrite_map[label as ContentTypeLabel] =
        target_label as ContentTypeLabel;
    }

    if (
      !(
        this.beg_size > 0 &&
        this.mid_size === 0 &&
        this.end_size > 0 &&
        !this.use_inputs_at_offsets &&
        this.medium_confidence_threshold > 0 &&
        this.min_file_size_for_dl > 0 &&
        this.padding_token != -1 &&
        this.block_size > 0
      )
    ) {
      throw new Error("Invalid config");
    }
  }
}
