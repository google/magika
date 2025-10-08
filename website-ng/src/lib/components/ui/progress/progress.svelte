<script lang="ts">
  import { Progress as ProgressPrimitive } from "bits-ui";
  import { cn, type WithoutChildrenOrChild } from "@/lib/utils.js";

  let {
    ref = $bindable(null),
    class: className,
    max = 100,
    value,
    showPercentage = false,
    height = "h-1",
    striped = false,
    ...restProps
  }: WithoutChildrenOrChild<ProgressPrimitive.RootProps> & {
    showPercentage?: boolean;
    height?: string;
    striped?: boolean;
  } = $props();

  const percentage = $derived.by(() =>
    Math.round(((value ?? 0) / (max ?? 1)) * 100)
  );
</script>

<ProgressPrimitive.Root
  bind:ref
  data-slot="progress"
  class={cn(
    "bg-primary/20 relative w-full overflow-hidden rounded-full",
    height,
    className
  )}
  {value}
  {max}
  {...restProps}
>
  <div
    data-slot="progress-indicator"
    class={cn(
      "bg-primary h-full w-full flex-1 transition-all relative overflow-hidden ",
      striped && "progress-striped"
    )}
    style="transform: translateX(-{100 - (100 * (value ?? 0)) / (max ?? 1)}%)"
  ></div>
  {#if showPercentage}
    <div
      class="absolute inset-0 flex mt-0-important font-size sl-markdown-content items-center justify-center text-xs font-medium text-foreground z-10 text-2xl mix-blend-difference text-white"
    >
      {percentage}%
    </div>
  {/if}
</ProgressPrimitive.Root>

<style>
  :global(.progress-striped) {
    background-color: #a142f4;
    background-image: linear-gradient(
      135deg,
      hsla(0, 0%, 100%, 0.25) 25%,
      transparent 0,
      transparent 50%,
      hsla(0, 0%, 100%, 0.25) 0,
      hsla(0, 0%, 100%, 0.25) 75%,
      transparent 0,
      transparent
    );
    background-size: 24px 24px;
    animation: progress-stripes 1s linear infinite;
  }

  @keyframes progress-stripes {
    0% {
      background-position: 0 0;
    }
    100% {
      background-position: 24px 0;
    }
  }

  /* Fix for Tailwind's mt-0 class being overridden by starlight */
  :global(.mt-0-important) {
    margin-top: 0 !important;
  }
</style>
