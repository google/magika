<template>
  <v-card
    class="pt-3 pb-3 pl-2 pr-3 mx-auto"
    variant="outlined"
    color="primary"
  >
    <v-card-text class="text-normal pb-6">
      You can drop your files below to test out Magika. The processing happens
      entirely in your browser - the files won't be uploaded anywhere else.
    </v-card-text>
    <v-file-input
      v-model="files"
      variant="solo-filled"
      multiple
      show-size
      counter
      label="Drop files here to classify them!"
    />
    <v-alert
      v-if="message"
      type="info"
      :text="message"
      variant="tonal"
      class="ml-9 mb-3"
    ></v-alert>
    <bars-visualization
      v-for="(file, index) in files"
      :key="index"
      :labels="labels[index]"
      :file="file"
    />
  </v-card>
</template>

<script setup>
import { onMounted, ref, watch } from "vue";
import { Magika } from "magika";

import BarsVisualization from "./BarsVisualization.vue";

const MAGIKA_MODEL_URL = `${import.meta.env.BASE_URL}model/model.json`;
const MAGIKA_CONFIG_URL = `${import.meta.env.BASE_URL}model/config.json`;

const files = ref([]);
const labels = ref([]);
const message = ref();

// Holds the initialized model and config.
const magika = new Magika();

watch(files, async () => {
  // Ensure magika is loaded.
  await magika.load({
    modelURL: MAGIKA_MODEL_URL,
    configURL: MAGIKA_CONFIG_URL,
  });
  // Process each file separately. Note that magika supports batching, but we
  // are keeping the code simple here.
  labels.value = new Array(files.value.length);
  for (const fileIndex in files.value) {
    const file = files.value[fileIndex];
    const content = await file.text();
    labels.value[fileIndex] = await magika.identifyBytesFull(content);
  }
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
  // Start loading Magika immediately (prefetch it).
  await magika.load({
    modelURL: MAGIKA_MODEL_URL,
    configURL: MAGIKA_CONFIG_URL,
  });
  message.value = "Magika is loaded! Drop any file to classify it";
  setTimeout(() => {
    message.value = null;
  }, 1000);

  // Load example file.
  const exampleFile = new File([EXAMPLE_JS], "example.js", {
    type: "text/plain",
  });
  files.value = [exampleFile];
});
</script>
