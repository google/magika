<script lang="ts">
  import * as Tabs from "@/lib/components/ui/tabs";
  import { Card } from "@/lib/components/ui/card";
  import { Button } from "@/lib/components/ui/button";
  import { Label } from "@/lib/components/ui/label";
  import Textarea from "@/lib/components/ui/textarea/textarea.svelte";
  import {
    displaySize,
    FileDropZone,
    MEGABYTE,
    type FileDropZoneProps,
  } from "@/lib/components/ui/file-drop-zone";
  import { Progress } from "@/lib/components/ui/progress";
  import XIcon from "@lucide/svelte/icons/x";
  import { onDestroy, onMount } from "svelte";
  import { toast } from "svelte-sonner";
  import { SvelteDate } from "svelte/reactivity";
  import { Magika } from "magika";

  // Magika configuration and examples.
  const MAGIKA_MODEL_VERSION = "standard_v3_3";
  const MAGIKA_MODEL_URL = `https://google.github.io/magika/models/standard_v3_3/model.json`;
  const MAGIKA_MODEL_CONFIG_URL = `https://google.github.io/magika/models/standard_v3_3/config.min.json`;
  const EXAMPLE_JS_BYTES = `
function hello() {
  const name = document.querySelector('input').value;
  const hi = 'salutation ' + name;
  return hi;
}
`;
  type MagikaClassification = {
    modelVersion: string;
    topLabel: string;
    scores: [string, number][];
  };

  type UploadedFile = {
    name: string;
    type: string;
    size: number;
    uploadedAt: number;
    classification: MagikaClassification;
  };
  // Reactive state helper.
  // Holds the initialized model and config.
  let magika: Magika | undefined = undefined;
  // State.
  let text = $state<string>(EXAMPLE_JS_BYTES);
  let loadingProgress = $state(0);
  let fileClassifications = $state<MagikaClassification[]>([]);
  let textClassification = $state<MagikaClassification | null>(null);
  let scoresClippedAt = $state(5);

  // Initialize Magika model.
  async function initializeMagika() {
    loadingProgress = 5;
    const timer = setTimeout(() => (loadingProgress = 66), 500);
    try {
      console.log("initializing magika");

      magika = await Magika.create({
        modelURL: MAGIKA_MODEL_URL,
        configURL: MAGIKA_MODEL_CONFIG_URL,
      });
      loadingProgress = 100;
      classifyText();
    } catch (err) {
      console.error(err);
    } finally {
      clearTimeout(timer);
    }
    console.log("magika: " + magika);
  }

  // Load magika immediately.
  onMount(() => initializeMagika());

  const classifyText = async () => {
    if (!magika) {
      return;
    }
    const encoder = new TextEncoder();
    const textBytes = encoder.encode(text);
    const magikaResult = await magika.identifyBytes(textBytes);
    const prediction = magikaResult?.prediction;
    const topLabel = prediction?.output?.label ?? "unknown"; // Default to 'unknown' if not found
    const scoresMap = prediction?.scores_map ?? {};
    const scores = Object.entries(scoresMap).sort((a, b) => b[1] - a[1]);
    textClassification = {
      modelVersion: magika.getModelName(),
      topLabel: topLabel,
      scores: scores,
    };
  };

  $effect(() => {
    if (!text) {
      textClassification = null;
      return;
    }
    console.log("Text changed, classifying...");
    classifyText();
  });

  const toggleScoreClipping = () => {
    console.log("Toggling score clipping. Current:", scoresClippedAt);
    if (scoresClippedAt === 5) {
      scoresClippedAt = 10000;
    } else {
      scoresClippedAt = 5;
    }
  };

  const onUpload: FileDropZoneProps["onUpload"] = async (files) => {
    await Promise.allSettled(files.map((file) => uploadFile(file)));
  };
  const onFileRejected: FileDropZoneProps["onFileRejected"] = async ({
    reason,
    file,
  }) => {
    console.warn("File rejected:", file, reason);
    toast.error(`${file.name} failed to upload!`, { description: reason });
  };
  const uploadFile = async (file: File) => {
    console.log("Uploading file:", file);
    if (!magika) {
      toast.error("Magika model is not loaded yet.");
      return;
    }
    const fileBytes = new Uint8Array(await file.arrayBuffer());
    const magikaResult = await magika.identifyBytes(fileBytes);
    const prediction = magikaResult?.prediction;
    const topLabel = prediction?.output?.label ?? "unknown"; // Default to 'unknown' if not found
    const scoresMap = prediction?.scores_map ?? {};
    const scores = Object.entries(scoresMap).sort((a, b) => b[1] - a[1]);
    files.push({
      name: file.name,
      type: file.type,
      size: file.size,
      uploadedAt: Date.now(),
      classification: {
        modelVersion: magika.getModelName(),
        topLabel,
        scores: scores.slice(0, 5),
      },
    });
  };

  let files = $state<UploadedFile[]>([]);
  let date = new SvelteDate();

  $effect(() => {
    const interval = setInterval(() => {
      date.setTime(Date.now());
    }, 10);
    return () => {
      clearInterval(interval);
    };
  });
</script>

<div class="flex w-full flex-col gap-6">
  {#if loadingProgress === 100}
    <Tabs.Root value="text" class="w-full ">
      <Tabs.List>
        <Tabs.Trigger value="text">Text input</Tabs.Trigger>
        <Tabs.Trigger value="file">File upload</Tabs.Trigger>
      </Tabs.List>
      <Tabs.Content value="text" class="w-full flex flex-col">
        <div>
          Type or paste text below to test out Magika. The processing happens
          entirely in your browser - the text won't be uploaded anywhere else.
          The content type is detected as you type.
        </div>
        <Label for="message"
          >Type or paste text below to classify its content type</Label
        >
        <Textarea
          bind:value={text}
          placeholder="Enter some text..."
          class="min-h-36 w-full"
        />

        {#if textClassification}
          <div class="mt-2 flex gap-2 text-sm">
            <span>Detected as</span>
            <span class="font-semibold">{textClassification?.topLabel}</span>
            <span>using model</span>
            <span class="font-semibold"
              >{textClassification?.modelVersion ?? "unknown"}</span
            >
          </div>
        {/if}
        {#if textClassification?.scores}
          <div class="mt-2 flex flex-col gap-2">
            <span>Scores:</span>

            <div
              class="grid w-full grid-cols-[max-content_1fr] gap-y-[0.2rem] gap-x-4"
            >
              {#each textClassification.scores.slice(0, scoresClippedAt) as [label, score], index}
                <span>{label}</span>

                <Progress
                  value={score * 100}
                  showPercentage={true}
                  height="h-5"
                  striped={index === 0}
                />
              {/each}
            </div>

            <Button variant="outline" onclick={toggleScoreClipping}>
              {scoresClippedAt === 5 ? "Show more types" : "Show less"}
            </Button>
          </div>
        {/if}
      </Tabs.Content>
      <Tabs.Content value="file">
        <div class="flex w-full flex-col gap-2 p-6">
          <div>
            You can drop your files below to test out Magika. The processing
            happens entirely in your browser - the files won't be uploaded
            anywhere else.
          </div>
          <FileDropZone
            {onUpload}
            {onFileRejected}
            maxFileSize={1000 * MEGABYTE}
            maxFiles={100}
            fileCount={files.length}
          />
          <div class="flex flex-col gap-2">
            {#each files as file, i (file.name)}
              <Card class="flex px-5 justify-between gap-2 relative">
                <Button
                  variant="ghost"
                  size="icon"
                  class="absolute right-2 top-2"
                  onclick={() => files.splice(i, 1)}
                >
                  <XIcon class="size-5 text-muted-foreground" />
                </Button>
                <div class="flex place-items-center gap-2">
                  <div class="flex flex-col">
                    <span>{file.name}</span>
                    <span class="text-muted-foreground text-xs"
                      >{displaySize(file.size)}</span
                    >
                  </div>
                </div>

                {#if file.classification}
                  <div class="mt-2 flex gap-2 text-sm">
                    <span>Detected as</span>
                    <span
                      class="bg-green-100 text-green-800 text-xs font-medium me-2 px-2.5 py-0.5 rounded-full dark:bg-green-900 dark:text-green-300"
                      >{file.classification?.topLabel}</span
                    >
                    <span>using model</span>
                    <span class="font-semibold"
                      >{file.classification?.modelVersion ?? "unknown"}</span
                    >
                  </div>
                {/if}
                {#if file.classification?.scores}
                  <div class="mt-2 flex flex-col gap-2">
                    <span class="font-bold">Scores</span>

                    <div
                      class="grid w-full grid-cols-[max-content_1fr] gap-y-[0.2rem] gap-x-4 align-items-center"
                    >
                      {#each file.classification.scores as [label, score], index}
                        <span class={index === 0 ? "font-bold" : ""}
                          >{label}</span
                        >

                        <Progress
                          value={score * 100}
                          showPercentage={true}
                          height="h-5"
                          striped={index === 0}
                        />
                      {/each}
                    </div>
                  </div>
                {/if}
              </Card>
            {/each}
          </div>
        </div>
      </Tabs.Content>
    </Tabs.Root>
  {:else}
    <Progress value={loadingProgress} max={100} class="w-full" />
    <div>Loading Magika model...</div>
  {/if}
</div>

<style scoped>
  /* Fixing starlight messing with tabs. */
  :global([role="tab"]) {
    margin-top: 0;
  }
</style>
