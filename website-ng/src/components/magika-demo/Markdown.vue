<template>
  <div class="container">
    <div v-html="content" />
  </div>
</template>
<script setup>
import { computed } from "vue";
import MarkdownIt from "markdown-it";
import hljs from "highlight.js";
import "highlight.js/styles/github.css";

const props = defineProps(["text"]);

const md = new MarkdownIt({
  html: true,
  linkify: true,
  typographer: true,
  highlight: function (str, lang) {
    if (lang && hljs.getLanguage(lang)) {
      try {
        return hljs.highlight(str, { language: lang }).value;
      } catch (__) {}
    }

    return ""; // use external default escaping
  },
});

const content = computed(() => md.render(props.text));
</script>

<style scoped>
@import "highlight.js/styles/github.css";
.container {
  overflow-wrap: break-word;
  word-break: break-all;
  line-height: 1.5;
}
.container :deep(h1) {
  display: none;
}
.container :deep(p) {
  padding-bottom: 0.5rem;
}
.container :deep(h2) {
  padding-top: 0.5rem;
  padding-bottom: 0.5rem;
}
</style>
