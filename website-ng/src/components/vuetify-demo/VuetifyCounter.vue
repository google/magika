<template>
  <v-card class="counter-card" variant="outlined">
    <v-card-title>Vuetify Counter</v-card-title>
    <v-card-subtitle>A simple interactive component</v-card-subtitle>

    <v-card-text class="text-h6">
      You have clicked the button <strong>{{ count }}</strong> times.
    </v-card-text>

    <v-card-actions>
      <v-btn
        @click="increment"
        color="primary"
        variant="tonal"
        prepend-icon="mdi-plus-circle"
      >
        Click Me
      </v-btn>
    </v-card-actions>
  </v-card>
</template>

<script setup>
import { ref } from 'vue';

// Reactive state for the counter
const count = ref(0);

// Function to increment the counter
function increment() {
  count.value++;
}
</script>

<style scoped>
/* These styles will ONLY apply to this component and won't leak out */
.counter-card {
  max-width: 450px;
  margin-top: 1rem;
  border-color: rgba(128, 128, 128, 0.3); /* Optional: add a subtle border */
}

strong {
  color: rgb(37, 99, 235); /* A nice blue color for the number */
}
</style>