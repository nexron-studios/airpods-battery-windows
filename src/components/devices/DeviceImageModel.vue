<script setup lang="ts">
import { computed } from 'vue'

const { src } = defineProps<{ src: string }>()

// The highlight is clipped to the product's silhouette by masking with the same image
const sheenMask = computed(() => ({ maskImage: `url(${src})`, maskSize: '100% 100%' }))
</script>

<template>
  <div class="relative h-full">
    <img :src class="h-full w-auto" alt="" draggable="false" />
    <div class="pointer-events-none absolute inset-0 overflow-hidden" :style="sheenMask">
      <div
        class="absolute inset-y-0 w-1/2 bg-linear-to-r from-transparent via-device-glint to-transparent animate-device-sheen"
      />
    </div>
  </div>
</template>
