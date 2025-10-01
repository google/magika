import { Magika } from "magika";

// Suppress TensorFlow logging messages.

async function main() {
  const magika = await Magika.create();
  const data = new TextEncoder().encode("import os; print(os.uname())");
  const prediction = await magika.identifyBytes(data);
  const statusDiv = document.createElement("div");
  statusDiv.className = "status";
  statusDiv.textContent = prediction.status;
  document.body.appendChild(statusDiv);

  const labelDiv = document.createElement("div");
  labelDiv.className = "label";
  labelDiv.textContent = prediction.prediction.output.label;
  document.body.appendChild(labelDiv);
}

document.addEventListener("DOMContentLoaded", main);
