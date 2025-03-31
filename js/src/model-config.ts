import * as fs from "fs/promises";
import { ContentTypeLabel } from "./content-type-label.js";
import assert from "assert";

export class ModelConfig {
  beg_size: number = 0;
  mid_size: number = 0;
  end_size: number = 0;
  use_inputs_at_offsets: boolean = false;
  medium_confidence_threshold: number = 0;
  min_file_size_for_dl: number = 0;
  padding_token: number = 0;
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

    assert(this.beg_size > 0);
    assert(this.mid_size == 0);
    assert(this.end_size > 0);
    assert(!this.use_inputs_at_offsets);
    assert(this.block_size > 0);
  }
}
