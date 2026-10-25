<script setup lang="ts">
import { computed } from 'vue'
import type { BatteryTone } from '@/types/battery.type'

const { percent, tone, hasGlint } = defineProps<{
  percent: number | null
  tone: BatteryTone
  /** A light runs once around the ring while the device model turns */
  hasGlint: boolean
}>()

const VIEWBOX = 60
const STROKE = 4
const RADIUS = (VIEWBOX - STROKE) / 2
const CIRCUMFERENCE = 2 * Math.PI * RADIUS
const CENTER = VIEWBOX / 2
const GLINT_ARC_SHARE = 0.14

const glintDashArray = `${CIRCUMFERENCE * GLINT_ARC_SHARE} ${CIRCUMFERENCE}`

const dashArray = computed(() => `${(CIRCUMFERENCE * (percent ?? 0)) / 100} ${CIRCUMFERENCE}`)

const strokeClass = computed(() => {
  switch (tone) {
    case 'ok':
      return 'stroke-level-ok'
    case 'low':
      return 'stroke-level-low'
    case 'unknown':
      return 'stroke-transparent'
    default: {
      const unhandled: never = tone
      throw new Error(`Unhandled battery tone: ${unhandled}`)
    }
  }
})
</script>

<template>
  <svg :viewBox="`0 0 ${VIEWBOX} ${VIEWBOX}`" class="-rotate-90" aria-hidden="true">
    <circle :cx="CENTER" :cy="CENTER" :r="RADIUS" fill="none" :stroke-width="STROKE" class="stroke-track" />
    <circle
      v-if="percent !== null"
      :cx="CENTER"
      :cy="CENTER"
      :r="RADIUS"
      fill="none"
      :stroke-width="STROKE"
      stroke-linecap="round"
      :stroke-dasharray="dashArray"
      :class="strokeClass"
    />
    <circle
      v-if="hasGlint"
      :cx="CENTER"
      :cy="CENTER"
      :r="RADIUS"
      fill="none"
      :stroke-width="STROKE"
      stroke-linecap="round"
      :stroke-dasharray="glintDashArray"
      class="origin-center stroke-device-glint opacity-0 animate-ring-glint [transform-box:fill-box]"
    />
  </svg>
</template>
