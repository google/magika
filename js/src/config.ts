import * as fs from 'fs/promises';
import {ContentType} from './contentType.js';

interface ConfigLabel {

    name: ContentType;
    threshold: number;
    is_text: boolean

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
        const config = await (await fetch(configURL)).json() as Record<string, any>;
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
        this.labels = config.labels;
        this.begBytes = config.input_size_beg;
        this.midBytes = config.input_size_beg;
        this.endBytes = config.input_size_beg;
    }

}
