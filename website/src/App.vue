<template>
  <v-app app>
    <v-main>
      <v-container class="px-6" fluid>
        <router-view></router-view>
      </v-container>
    </v-main>
  </v-app>
</template>

<style lang="scss">
@import url("https://fonts.googleapis.com/css?family=Google+Sans+Text:300,400,700&display=swap");
@import url("https://fonts.googleapis.com/css?family=Google+Sans&display=swap");

$body-font-family: "Google Sans Text";
$title-font: "Google Sans";

.v-application {
  font-family: $body-font-family, sans-serif !important;

  .title {
    // To pin point specific classes of some components
    font-family: $title-font, sans-serif !important;
  }
}

pre {
  background-color: #f5eaff;
  border-radius: 10px;
  padding: 1rem 0.5rem;
  margin: 0.5rem 0.5rem;
}
code {
  background-color: #f5eaff;
  border-radius: 5px;
  padding: 0.2rem 0.5rem;
}
</style>
