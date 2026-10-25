<script setup lang="ts">
import { computed } from 'vue'

const { src } = defineProps<{
  /** A horizontal strip of square frames, one full turn from front to front */
  src: string
}>()

/** Must match `--turn-frames` in main.css and the render script's FULL_TURN_FRAMES + 1 */
const FRAME_COUNT = 49

const stripWidth = computed(() => ({ width: `${FRAME_COUNT * 100}%` }))
</script>

<template>
  <!-- A window one frame wide; the strip slides through it frame by frame -->
  <div class="relative aspect-square h-full overflow-hidden">
    <img
      :src
      class="absolute inset-y-0 left-0 h-full max-w-none animate-device-turn"
      :style="stripWidth"
      alt=""
      draggable="false"
    />
  </div>
</template>
