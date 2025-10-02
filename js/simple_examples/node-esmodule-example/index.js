import { MagikaNode as Magika } from "magika/node";
import { TextEncoder } from "util";

const magika = await Magika.create();
const text = "import os; print(os.uname())";
const bytes = new TextEncoder().encode(text);
const prediction = await magika.identifyBytes(bytes);
console.log(prediction.status, prediction.prediction.output.label);
