/**
 * @fileoverview Description of this file.
 */
import { createRouter, createWebHistory } from "vue-router";

import Homepage from "../components/Homepage.vue";

// Load each demo on a separate route.
export default createRouter({
  history: createWebHistory("/magika/"),
  routes: [{ path: "/", name: "Home", component: Homepage }],
});
