<template>
  <v-card class="mt-3 mb-3">
    <v-card-title class="title-h3"> {{ file.name }}</v-card-title>

    <v-card-subtitle v-if="props.resultData && props.resultData.topLabel">
      Detected as <v-chip rounded color="primary">{{ props.resultData.topLabel }}</v-chip> (model: <code>{{ props.resultData.modelVersion }}</code>)
    </v-card-subtitle>

    <v-card-subtitle v-else-if="props.resultData && props.resultData.error">
       <v-chip rounded color="error">Error: {{ props.resultData.error }}</v-chip>
    </v-card-subtitle>

    <v-card-subtitle v-else> Classifying...</v-card-subtitle>

    <v-card-text>
      <v-data-table
        :items-per-page="5"
        :headers="headers"
        :items="items"
        item-value="name"
        :loading="!props.resultData" v-model:sort-by="sortBy"
      >
        <template v-slot:loading>
          <v-skeleton-loader type="table-row@5" /> </template>

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

         <template v-slot:no-data>
           <div v-if="!props.resultData">Loading scores...</div>
           <div v-else>No scoring data available.</div>
         </template>

      </v-data-table>
    </v-card-text>
  </v-card>
</template>

<script setup>
import { ref, computed } from "vue";

const props = defineProps(["file", "resultData"]);

const headers = [
  { title: "Content type", sortable: true, key: "name" },
  { title: "Probability", sortable: true, key: "probability" },
];

const items = computed(() => {
  // Access the 'scores' map from the 'resultData' prop.
  const scoresMap = props.resultData?.scores ?? {};
  return Object.entries(scoresMap).map(([label, probability]) => {
    return {
      name: label,
      probability,
    };
  });
});

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
