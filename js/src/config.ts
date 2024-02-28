import {ContentType} from './contentType';

class ConfigLabel {

  name: ContentType;
  threshold: number;
  is_text: boolean

}

export class Config {

  labels: ConfigLabel[];
  minFileSizeForDl: number;
  paddingToken: number;
  begBytes: number;
  midBytes: number;
  endBytes: number;
  extractSize: number;

  async load(configURL: string): Promise<void> {
    if (this.labels != null) {
      return;
    }

    const config = await (await fetch(configURL)).json();
    this.minFileSizeForDl = config.min_file_size_for_dl;
    this.paddingToken = config.padding_token;
    this.labels = config.labels;
    this.begBytes = config.input_size_beg;
    this.midBytes = config.input_size_beg;
    this.endBytes = config.input_size_beg;
    if (this.begBytes > 0) {
      this.extractSize = this.begBytes
    } else if (this.begBytes > 0) {
      this.extractSize = this.midBytes
    } else {
      this.extractSize = this.endBytes;
    }
  }

}
