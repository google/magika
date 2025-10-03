/**
 * plugins/vuetify.js
 *
 * Framework documentation: https://vuetifyjs.com`
 */

// Styles
import "@mdi/font/css/materialdesignicons.css";
// import "vuetify/styles";
// import { VDataTable } from "vuetify/components/VDataTable";
// import { VSkeletonLoader } from "vuetify/components/VSkeletonLoader";

// Composables
import { createVuetify } from "vuetify";
import { md3 } from "vuetify/blueprints";

import * as components from "vuetify/components";
import * as directives from "vuetify/directives";

// https://vuetifyjs.com/en/introduction/why-vuetify/#feature-guides
export default createVuetify({
  blueprint: md3,
  ssr: true,
  components,
  directives,
  theme: {
    theme: {
      defaultTheme: "light",
    },
    themes: {
      light: {
        colors: {
          primary: "#a142f4",
          secondary: "#2b984a",
        },
      },
    },
  },
});
