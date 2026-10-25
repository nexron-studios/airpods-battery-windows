<script setup lang="ts">
import { Bluetooth } from '@lucide/vue'
import { computed } from 'vue'
import airpodLeftTurn from '@/assets/devices/airpod-left-turn.webp'
import airpodRightTurn from '@/assets/devices/airpod-right-turn.webp'
import airpodsCaseTurn from '@/assets/devices/airpods-case-turn.webp'
import iphoneTurn from '@/assets/devices/iphone-turn.webp'
import DeviceTurnModel from '@/components/devices/DeviceTurnModel.vue'
import type { DeviceKind } from '@/types/battery.type'

const { kind, order } = defineProps<{
  kind: DeviceKind
  /** Position in the row: offsets the idle float */
  order: number
}>()

/** Spreads the floats of neighbouring models across one cycle */
const FLOAT_PHASE_STEP_MS = 1300

const turnSrc = computed((): string | null => {
  switch (kind) {
    case 'airpod_left':
      return airpodLeftTurn
    case 'airpod_right':
      return airpodRightTurn
    case 'airpods_case':
      return airpodsCaseTurn
    case 'iphone':
      return iphoneTurn
    case 'other':
      return null
    default: {
      const unhandled: never = kind
      throw new Error(`Unhandled device kind: ${unhandled}`)
    }
  }
})

const floatPhase = computed(() => ({ animationDelay: `-${order * FLOAT_PHASE_STEP_MS}ms` }))
</script>

<template>
  <div class="relative flex size-full items-center justify-center">
    <div
      v-if="turnSrc"
      class="absolute inset-0 rounded-full bg-radial from-device-bloom to-transparent to-70% opacity-0 animate-device-bloom"
    />
    <div
      class="absolute bottom-0 left-1/2 h-1 w-3/5 -translate-x-1/2 rounded-full bg-device-ground blur-[2px] animate-device-ground"
      :style="floatPhase"
    />
    <div class="h-full animate-device-float" :style="floatPhase">
      <DeviceTurnModel v-if="turnSrc" :src="turnSrc" />
      <Bluetooth v-else class="size-full p-1 text-ink-muted" aria-hidden="true" />
    </div>
  </div>
</template>
