<template>
  <v-card class="mt-3 mb-3">
    <v-card-title class="title-h3"> {{ file.name }}</v-card-title>
    <v-card-subtitle v-if="labels">
      Detected as <v-chip rounded color="primary">{{ labels["label"] }}</v-chip>
    </v-card-subtitle>

    <v-card-subtitle v-else> Classifying...</v-card-subtitle>
    <v-card-text>
      <v-data-table
        :items-per-page="5"
        :headers="headers"
        :items="items"
        item-value="name"
        :loading="!labels"
        v-model:sort-by="sortBy"
      >
        <template v-slot:loading>
          <v-skeleton-loader type="table-row@10"
        /></template>
        <template v-slot:item="{ item, index }">
          <tr>
            <td>{{ item.name }}</td>

            <td>
              <v-progress-linear
                :model-value="item.probability * 100"
                bg-color="grey"
                :color="index === 0 ? 'primary' : 'black'"
                height="25"
                rounded-bar
                :striped="index === 0"
              >
                <template v-slot:default="{ value }">
                  <strong>{{ Math.ceil(value) }}%</strong>
                </template>
              </v-progress-linear>
            </td>
          </tr>
        </template>
      </v-data-table>
    </v-card-text>
  </v-card>
</template>

<script setup>
import { ref, computed } from "vue";
const props = defineProps(["file", "labels"]);

const headers = [
  { title: "Content type", sortable: true, key: "name" },
  { title: "Probability", sortable: true, key: "probability" },
];
const items = computed(() =>
  Object.entries(props.labels?.["labels"] || {}).map(([label, probability]) => {
    return {
      name: label,
      probability,
    };
  }),
);

const sortBy = ref([{ key: "probability", order: "desc" }]);
</script>

<style scoped>
.wrapper {
  display: grid;
  width: 100%;
  grid-template-columns: max-content max-content 1fr;
  grid-row-gap: 0.2rem;
  grid-column-gap: 1rem;
}
</style>
