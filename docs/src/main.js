import { createApp } from "vue";

import App from "./App.vue";

import { registerPlugins } from "@/plugins";

// Create the vue app.
const app = createApp(App);
registerPlugins(app);
app.mount("#app");
