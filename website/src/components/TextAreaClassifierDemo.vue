<template>
    <v-card class="pt-3 pb-3 pl-2 pr-3 mx-auto" variant="outlined" color="primary">
        <v-card-text class="text-normal pb-4">
            Type or paste text below to test out Magika. The processing happens
            entirely in your browser - the text won't be uploaded anywhere else. The
            content type is detected as you type (debounced).
        </v-card-text>

        <v-textarea v-model="inputText" variant="solo-filled"
            label="Type or paste text here to classify its content type" rows="8" auto-grow clearable />

        <v-alert v-if="message" type="info" :text="message" variant="tonal" class="ml-9 mb-3"></v-alert>

        <bars-visualization v-if="resultData" :file="{ name: 'Live Text Input' }" :resultData="resultData" />
        <v-card-text v-else-if="!message && inputText" class="text-center text-medium-emphasis">
            Waiting for input or processing...
        </v-card-text>
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

const inputText = ref(""); // Holds the text from the textarea
const resultData = ref(null); // Holds the analysis result for the text
const message = ref(); // For loading/status messages

// Holds the initialized model and config.
let magika = undefined;
// Timer ID for debouncing the input processing
let debounceTimeout = null;
const DEBOUNCE_DELAY = 300; // milliseconds

// Example initial text
const EXAMPLE_JS = `
function hello() {
  const name = document.querySelector('input').value;
  const hi = 'salutation ' + name;
  return hi;
}
`;

// Function to process the text input
const processText = async (text) => {
    if (!text) {
        resultData.value = null; // Clear results if text is empty
        return;
    }

    // Ensure magika is loaded.
    if (!magika) {
        message.value = "Magika model is still loading...";
        console.warn("Magika instance not ready yet.");
        // Optionally clear results or show loading state
        resultData.value = { topLabel: 'loading', scores: null, error: "Model loading", modelVersion: 'N/A' };
        // Attempt to load if not already loading
        await loadMagika(); // Make sure loadMagika handles multiple calls gracefully
        if (!magika) { // Check again after attempting load
            resultData.value = { topLabel: null, scores: null, error: "Model failed to load", modelVersion: 'N/A' };
            message.value = "Error loading Magika model. Please refresh.";
            return;
        }
    }

    try {
        // Convert the text string to Uint8Array
        const textBytes = new TextEncoder().encode(text.trim());

        // Identify content type using Magika
        const magikaResult = await magika.identifyBytes(textBytes);

        // Extract the required information
        const prediction = magikaResult?.prediction;
        const topLabel = prediction?.output?.label ?? 'unknown';
        const scoresMap = prediction?.scores_map ?? {};

        // Update the resultData ref
        resultData.value = {
            modelVersion: magika.getModelName(),
            topLabel: topLabel,
            scores: scoresMap
        };
        // Clear any temporary messages once processed
        if (message.value === "Magika model is still loading...") {
            message.value = null;
        }

    } catch (error) {
        console.error(`Error processing text input:`, error);
        // Set an error state for the result
        resultData.value = {
            modelVersion: magika?.getModelName() ?? 'N/A',
            topLabel: null,
            scores: null,
            error: `Failed to process: ${error.message}`
        };
    }
};

// Watch the inputText ref for changes
watch(inputText, (newText) => {
    // Clear the previous debounce timer
    if (debounceTimeout) {
        clearTimeout(debounceTimeout);
    }
    // Set a new timer to process the text after DEBOUNCE_DELAY ms
    debounceTimeout = setTimeout(() => {
        processText(newText);
    }, DEBOUNCE_DELAY);
});

// Function to load Magika (can be called from onMounted or watcher)
async function loadMagika() {
    // Avoid redundant loading if already loaded or loading initiated
    if (magika || message.value === "Initializing Magika...") return;

    message.value = "Initializing Magika...";
    try {
        magika = await Magika.create({
            modelURL: MAGIKA_MODEL_URL,
            configURL: MAGIKA_MODEL_CONFIG_URL,
        });
        message.value = "Magika is loaded! Ready for input.";
        setTimeout(() => {
            // Clear the message only if it's the success one
            if (message.value === "Magika is loaded! Ready for input.") {
                message.value = null;
            }
        }, 2000);
    } catch (error) {
        console.error("Failed to initialize Magika:", error);
        message.value = "Failed to initialize Magika. Please refresh.";
        resultData.value = { topLabel: null, scores: null, error: "Model loading failed", modelVersion: 'N/A' };
    }
}

onMounted(async () => {
    await loadMagika(); // Start loading Magika

    // Set initial text content
    inputText.value = EXAMPLE_JS.trim();

    // Trigger initial analysis if Magika loaded successfully
    if (magika) {
        await processText(inputText.value);
    } else {
        // If Magika didn't load immediately, the watcher might handle it,
        // or we wait for the loadMagika promise to resolve if needed.
        // Setting a placeholder might be good UX.
        resultData.value = { topLabel: 'loading', scores: null, error: "Model initializing", modelVersion: 'N/A' };
    }

});

</script>

<style scoped>
/* Add any specific styles if needed */
.text-normal {
    line-height: 1.5;
}
</style>