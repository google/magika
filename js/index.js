#! /usr/bin/env node

// To run this, you need to install the optional dependencies too.
import {Magika} from './magika.js';
import { program } from 'commander';
import { readFile } from 'fs/promises';
// Load the node version of tensorflow, since we're running in the command line.
import * as tf from '@tensorflow/tfjs-node';
import chalk from 'chalk';
import {Magika} from './magika.js';


program
.description('Magika JS - file type detection with ml. https://google.github.io/magika')
.option('--model-url <model-url>', 'Model URL', 'https://google.github.io/magika/model/model.json')
.option('--config-url <config-url>', 'Config  URL', 'https://google.github.io/magika/model/config.json')
.argument('<paths...>', 'Paths of the files to detect');

program.parse();

const flags = program.opts();
const magika = new Magika();
await magika.load( flags.modelUrl, flags.configUrl );
await Promise.all(
  program.args
  .map(async path => {
    let data = null;
    try {
      data = (await readFile(path)).toString()
    } catch (error) {
      console.error('Skipping file', path, error)
    }
    const features = await magika.extractFeaturesFromFile(data)
    const prediction = await magika.predictFromFeatures(features)
    console.log(chalk.blue(path), chalk.green(prediction['label']))
  })
)
