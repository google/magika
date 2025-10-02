const { MagikaNode: Magika } = require("magika/node");

async function main() {
  const magika = await Magika.create();
  const prediction = await magika.identifyBytes(
    Buffer.from("import os; print(os.uname())")
  );
  console.log(prediction.status, prediction.prediction.output.label);
}

main();
