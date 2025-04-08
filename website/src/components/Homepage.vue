<template>
  <header>
    <v-img src="@/../assets/logo.svg" />
    <div>
      <div class="text-h2 pb-3 text-primary">Magika</div>
      <div class="text-normal">
        Magika is a tool to detect common file content types, using deep
        learning.
      </div>
      <a href="https://github.com/google/magika" class="button"><small>View project on</small> GitHub</a>
    </div>
  </header>

  <div class="text-normal pt-6 mt-6 pb-3 pr-3 pl-3 border rounded-lg bg-primary">
    <h2> This website now uses the latest model, `standard_v3_2`!</h2>
    <p>
      The new model supports 200+ content types (2x from the previous version), with the same overall average accuracy
      of 99%, and it is 20% faster.</p>
    <p>
      Our `magika` command line tool (now written in Rust), as well as our Python and Rust libraries support this new
      model as well.
    </p>
    <p>
      For more information and the latest news, go check <a href="https://github.com/google/magika">the Magika GitHub
        repository</a>!
    </p>
  </div>

  <div class="text-normal pt-6 mt-6 pb-3 pr-3 pl-3">
    <p>
      Magika is a novel AI-powered file type detection tool that relies on the
      recent advance of deep learning to provide accurate detection. Under the
      hood, Magika employs a custom, highly optimized model that only weighs
      about a few MBs, and enables precise file identification within
      milliseconds. Magika has been trained and evaluated on a dataset of ~100M
      samples across 200+ content types (covering both binary and textual file
      formats), and it achieves an average ~99% accuracy on our test set.
      Designed for efficiency, Magika runs quickly even on a single CPU. A
      similar model currently scans hundreds of billions of files every week at
      Google (see our announcment
      <a href='https://opensource.googleblog.com/2024/02/magika-ai-powered-fast-and-efficient-file-type-identification.html'
        class='text-primary'>blog post</a>).
    </p>
  </div>

  <div class="text-h3 pt-6 pb-3">Demo (Magika runs in your browser!)</div>

  <v-tabs v-model="activeTab" align-tabs="center" class="mb-4" grow>
    <v-tab value="text">Text Input Demo</v-tab>
    <v-tab value="file">File Upload Demo</v-tab>
  </v-tabs>

  <v-window v-if="isSupported === true" v-model="activeTab">
    <v-window-item value="text">
      <div class="pa-4">
        <TextAreaClassifierDemo />
      </div>
    </v-window-item>

    <v-window-item value="file">
      <div class="pa-4">
        <FileClassifierDemo />
      </div>
    </v-window-item>
  </v-window>

  <v-alert v-else-if="isSupported === false" type="warning" variant="tonal" class="mt-4 mx-4" title="Demo Not Supported"
    text="Unfortunately, this version of the interactive demo requires browser
    features that are not available or enabled on your current device/browser.
    But we are working on a fix, stay tuned!">
  </v-alert>

  <div class="text-h3 pt-6 mt-6 pb-3">Get Magika in your command line</div>
  <div class="text-normal pt-3 pr-3 pl-3">
    The Magika client is written in written in Rust, and you can install it with:
    <code>pip install magika</code>
  </div>
  <div class="text-normal pt-3 pb-3 pr-3 pl-3">
    Then, you can run it by executing <code>magika</code> like so:
  </div>
  <pre>
  $ magika examples/*
  code.asm: Assembly (code)
  code.py: Python source (code)
  doc.docx: Microsoft Word 2007+ document (document)
  doc.ini: INI configuration file (text)
  elf64.elf: ELF executable (executable)
  flac.flac: FLAC audio bitstream data (audio)
  image.bmp: BMP image data (image)
  java.class: Java compiled bytecode (executable)
  jpg.jpg: JPEG image data (image)
  pdf.pdf: PDF document (document)
  pe32.exe: PE executable (executable)
  png.png: PNG image data (image)
  README.md: Markdown document (text)
  tar.tar: POSIX tar archive (archive)
  webm.webm: WebM data (video)
  </pre>

  <div class="text-h3 pt-6 mt-6">Libraries</div>
  <div class="text-normal pt-3 pb-3 pr-3 pl-3">
    You can use Magika in your
    <a href="https://github.com/google/magika/tree/main/python">Python</a>
    code, in
    <a href="https://github.com/google/magika/tree/main/js">JavaScript</a> (in
    Node or client side; In fact, this page is using Magika's JavaScript
    library), in <a href="https://github.com/google/magika/tree/main/rust">Rust</a>, and soon in
    <a href="https://github.com/google/magika/tree/main/go">GoLang</a>!
    Check the <a href="https://github.com/google/magika">Magika GitHub repository</a> for more details.

  </div>

  <div class="text-h3 pt-6 pb-3">Paper</div>
  <div class="text-normal pb-6 pl-3">
    You can read <a href="https://arxiv.org/abs/2409.13768" target="_blank">our research paper</a> on how the Magika
    model was
    trained and its performance on large datasets.
  </div>
  <a href="https://arxiv.org/abs/2409.13768" target="_blank">
    <v-img src="@/../assets/paper.png" class="paper pt-3 pb-3" />
  </a>

  <div class="text-normal pt-6 pb-3 pl-3">
    If you use Magika, please cite it like this:
  </div>
  <pre>
@InProceedings{fratantonio25:magika,
  author = {Yanick Fratantonio and Luca Invernizzi and Loua Farah and Kurt Thomas and Marina Zhang and Ange Albertini and Francois Galilee and Giancarlo Metitieri and Julien Cretin and Alexandre Petit-Bianco and David Tao and Elie Bursztein},
  title = { {Magika: AI-Powered Content-Type Detection} },
  booktitle = {Proceedings of the International Conference on Software Engineering (ICSE)},
  month = {April},
  year = {2025}
}
    </pre>

  <div class="text-h3 pt-6 mt-6 pb-3">
    Need more info? See our
    <a class="text-primary" href="https://github.com/google/magika/">README</a>
    on GitHub!
  </div>

</template>

<script setup>
import { ref, onMounted } from 'vue';
import FileClassifierDemo from "@/components/FileClassifierDemo.vue";
import TextAreaClassifierDemo from "./TextAreaClassifierDemo.vue";

function getMaxTexture() {
  var canvas = document.createElement('canvas');
  var gl = canvas.getContext('webgl');
  return gl.getParameter(gl.MAX_TEXTURE_SIZE);
}

function isInferenceSupported() {
  return getMaxTexture() >= 5804;
}


// Reactive variable to control the active tab
const activeTab = ref('text');

// Reactive variable to store whether the demo is supported
// null = undetermined, true = supported, false = not supported
const isSupported = ref(null);

// Check for inference support after the component is mounted
onMounted(() => {
  // Call your support checking function
  // Ensure isInferenceSupported is defined and returns a boolean
  try {
    // Make sure isInferenceSupported is defined in your scope
    isSupported.value = isInferenceSupported();
  } catch (error) {
    console.error("Error calling or finding isInferenceSupported():", error);
    isSupported.value = false; // Assume not supported if check fails
  }
});

</script>

<style scoped lang="scss">
body {
  font-family: "Google Sans Text", Helvetica, Arial, serif;
  font-weight: 400;
  line-height: 1.2;
}

h1,
h2,
h3,
h4 {
  font-family: "Google Sans", Helvetica, Arial, serif;
}

header {
  padding: 3rem;
  background: #f1f3f4;
  display: grid;
  grid-gap: 2rem;
  grid-template-columns: max-content 1fr;

  & div.v-img {
    width: 100%;
    min-width: clamp(2rem, 10vw, 7rem);
    height: 100%;
  }

  & h1 {
    text-transform: capitalize;
  }

  & h2 {
    font-weight: 400;
  }
}


.paper {
  max-height: 30rem;
  mask-image: linear-gradient(to bottom,
      rgba(0, 0, 0, 1),
      80%,
      rgba(0, 0, 0, 0));
}

pre {
  white-space: pre-line;
}
</style>