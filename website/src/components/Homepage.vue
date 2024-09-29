<template>
  <header>
    <v-img src="@/../assets/logo.svg" />
    <div>
      <div class="text-h2 pb-3 text-primary">Magika</div>
      <div class="text-normal">
        Magika is a tool to detect common file content types, using deep
        learning.
      </div>
      <a href="https://github.com/google/magika" class="button"
        ><small>View project on</small> GitHub</a
      >
    </div>
  </header>

  <div class="text-normal pt-6 mt-6 pb-3 pr-3 pl-3">
    <p>
      Magika leverages the power of cutting-edge deep learning to enhance the
      world of file type detection. It provides increased accuracy and support
      for a comprehensive range of content types, outperforming traditional
      tools with 99%+ average precision and recall.
    </p>
    <p>
      Designed for efficiency, Magika runs quickly even on a single CPU. A
      similar model currently scans millions of files per second at Google (see <a href='https://opensource.googleblog.com/2024/02/magika-ai-powered-fast-and-efficient-file-type-identification.html' class='text-primary'>blog post</a>).
    </p>
  </div>

  <div class="text-h3 pt-6 pb-3">Demo</div>
  <FileClassifierDemo cols="6" />

  <div class="text-h3 pt-6 mt-6 pb-3">Get Magika in your command line</div>
  <div class="text-normal pt-3 pr-3 pl-3">
    You can start using Magika by installing it as a Python package:
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

  <div class="text-h3 pt-6 mt-6">Libraries!</div>
  <div class="text-normal pt-3 pb-3 pr-3 pl-3">
    You can use Magika in your
    <a href="https://github.com/google/magika/tree/main/python">Python</a>
    code, or your
    <a href="https://github.com/google/magika/tree/main/js">JavaScript</a> (in
    Node or client side). In fact, this page is using Magika's JavaScript
    library!
  </div>

  <div class="text-h3 pt-6 pb-3">Paper</div>
  <div class="text-normal pb-6 pl-3">
    You can read <a href="https://arxiv.org/abs/2409.13768" target="_blank">our research paper</a> on how the Magika model was
    trained and its performance on large datasets.
  </div>
  <a href="https://arxiv.org/abs/2409.13768" target="_blank">
    <v-img src="@/../assets/paper.png" class="paper pt-3 pb-3" />
  </a>

  <div class="text-normal pt-6 pb-3 pl-3">
    If you use Magika, please cite it like this:
  </div>
  <pre>
@misc{magika,
      title={{Magika: AI-Powered Content-Type Detection}},
      author={{Fratantonio, Yanick and Invernizzi, Luca and Farah, Loua and Kurt, Thomas and Zhang, Marina and Albertini, Ange and Galilee, Francois and Metitieri, Giancarlo and Cretin, Julien and Petit-Bianco, Alexandre and Tao, David and Bursztein, Elie}},
      year={2024},
      eprint={2409.13768},
      archivePrefix={arXiv},
      primaryClass={cs.CR},
      url={https://arxiv.org/abs/2409.13768},
}
    </pre
  >

  <div class="text-h3 pt-6 mt-6 pb-3">
    Need more info? See our
    <a class="text-primary" href="https://github.com/google/magika/">README</a>
    on GitHub!
  </div>

  <div class="text-h3 pt-12 mt-6 pb-3 model-card">Model card</div>
  <v-card>
    <v-card-text>
      <div v-html="ModelCard" />
    </v-card-text>
  </v-card>
</template>

<script setup>
import FileClassifierDemo from "@/components/FileClassifierDemo.vue";

import ModelCard from "@/../assets/model_card.html?raw";
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
  mask-image: linear-gradient(
    to bottom,
    rgba(0, 0, 0, 1),
    80%,
    rgba(0, 0, 0, 0)
  );
}

pre {
  white-space: pre-line;
}
</style>
