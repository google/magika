<template>
  <v-card variant="plain">
    <v-card-item>
      <v-card-title>
        Live Text Demo
      </v-card-title>
      <v-card-subtitle>
        Type in the box below to see the result update instantly.
      </v-card-subtitle>
    </v-card-item>

    <v-card-text>
      <v-textarea
        v-model="inputText"
        label=" "
        variant="default"
        rows="3"
        auto-grow
        clearable
      ></v-textarea>

      <div class="mt-4">
        <p class="text-overline">Reversed Output:</p>
        <p class="text-h6 text-medium-emphasis bg-surface-variant pa-4 rounded">
          {{ processedText }}
        </p>
      </div>
    </v-card-text>
  </v-card>
</template>

<script setup>
import { ref, computed } from 'vue';

// A "ref" is used to hold reactive state.
const inputText = ref('Hello Vue and Astro!');

// 3. THE CALLBACK/FUNCTION
// A `computed` property is the ideal "callback" for this use case.
// It's a function that automatically re-runs whenever its dependencies (`inputText`) change.
// The return value is cached and only updated when needed.
const processedText = computed(() => {
  // You can put any logic you want here!
  return inputText.value.split('').reverse().join('');
});
</script>