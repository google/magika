import { MagikaNode as Magika } from "magika/node";

async function main(): Promise<void> {
  const magika = await Magika.create();
  const data = Buffer.from("import os; print(os.uname())");
  const prediction = await magika.identifyBytes(data);
  console.log(prediction.status, prediction.prediction.output.label);
}

main();
