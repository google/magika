<template>
  <v-card class="pt-3 pb-3 pl-2 pr-3 mx-auto" variant="outlined" color="primary">
    <v-card-text class="text-normal pb-6">
      You can drop your files below to test out Magika. The processing happens
      entirely in your browser - the files won't be uploaded anywhere else.
    </v-card-text>
    <v-file-input v-model="files" variant="solo-filled" multiple show-size counter
      label="Drop files here to classify them!" />
    <v-alert v-if="message" type="info" :text="message" variant="tonal" class="ml-9 mb-3"></v-alert>
    <bars-visualization v-for="(file, index) in files" :key="index" :file="file" :resultData="results[index]" />
  </v-card>
</template>

<script setup>
import { onMounted, ref, watch } from "vue";
import { Magika } from "magika";

import BarsVisualization from "./BarsVisualization.vue";

// Interface for the structure we'll pass to BarsVisualization.
/*
interface ProcessedResult {
  modelVersion: string;
  topLabel: string | null;
  scores: Record<string, number> | null;
  error?: string; // Optional: include an error message if processing fails
}
*/

const MAGIKA_MODEL_VERSION = "standard_v3_3"
const MAGIKA_MODEL_URL = `${import.meta.env.BASE_URL}models/${MAGIKA_MODEL_VERSION}/model.json`;
const MAGIKA_MODEL_CONFIG_URL = `${import.meta.env.BASE_URL}models/${MAGIKA_MODEL_VERSION}/config.min.json`;

const files = ref([]);
const results = ref([]);
const message = ref();

// Holds the initialized model and config.
let magika = undefined;

watch(files, async () => {
  if (!files.value || files.value.length === 0) {
    results.value = []; // Clear results if files are cleared
    return;
  }

  // Ensure magika is loaded.
  try {
    if (magika === undefined) {
      magika = await Magika.create({
        modelURL: MAGIKA_MODEL_URL,
        configURL: MAGIKA_MODEL_CONFIG_URL,
      });
    }
  } catch (error) {
    console.error("Failed to load Magika model:", error);
    message.value = "Error loading Magika model. Please refresh.";
    // Optionally clear results or show an error state for each file
    results.value = files.value.map(() => ({ topLabel: null, scores: null, error: "Model loading failed" }));
    return;
  }

  // Initialize results array with nulls or placeholders
  results.value = new Array(files.value.length).fill(null);

  // Process each file separately.
  const processingPromises = files.value.map(async (file, fileIndex) => {
    try {
      const fileBytes = new Uint8Array(await file.arrayBuffer());
      // Call identifyBytes which now returns a MagikaResult object
      const magikaResult = await magika.identifyBytes(fileBytes);

      // Extract the required information from the MagikaResult object
      // Add checks for nested properties to prevent errors if the structure is unexpected
      const prediction = magikaResult?.prediction;
      const topLabel = prediction?.output?.label ?? 'unknown'; // Default to 'unknown' if not found
      const scoresMap = prediction?.scores_map ?? {}; // Default to empty object if not found

      // Store the extracted data in the results array for the corresponding file
      // Adjust the structure based on what BarsVisualization expects
      console.log(magika)
      results.value[fileIndex] = {
        modelVersion: magika.getModelName(),
        topLabel: topLabel,
        scores: scoresMap
      };
    } catch (error) {
      console.error(`Error processing file ${file.name}:`, error);
      // Set an error state for this specific file's result
      results.value[fileIndex] = {
        topLabel: null,
        scores: null,
        error: `Failed to process: ${error.message}`
      };
    }
  });

  // Wait for all files to be processed
  await Promise.all(processingPromises);

  // Force reactivity update if needed (usually Vue handles this well with ref)
  // results.value = [...results.value]; // Uncomment if updates aren't showing
});


const EXAMPLE_JS = `
function hello() {
  const name = document.querySelector('input').value;
  const hi = 'salutation ' + name;
  return hi
}
`;

onMounted(async () => {
  message.value = "Initializing Magika...";
  try {
    // Start loading Magika immediately (prefetch it).
    if (magika === undefined) {
      magika = await Magika.create({
        modelURL: MAGIKA_MODEL_URL,
        configURL: MAGIKA_MODEL_CONFIG_URL,
      });
    }
    message.value = "Magika is loaded! Drop any file to classify it";
    setTimeout(() => {
      message.value = null;
    }, 2000); // Slightly longer timeout

    // Load example file.
    const exampleFile = new File([EXAMPLE_JS], "example.js", {
      type: "text/plain",
    });
    files.value = [exampleFile];

  } catch (error) {
    console.error("Failed to initialize Magika on mount:", error);
    message.value = "Failed to initialize Magika. Please refresh.";
  }
});
</script>