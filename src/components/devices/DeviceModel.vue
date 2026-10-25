<script setup lang="ts">
import { Bluetooth } from '@lucide/vue'
import { computed } from 'vue'
import airpodLeft from '@/assets/devices/airpod-left.png'
import airpodRight from '@/assets/devices/airpod-right.png'
import airpodsCase from '@/assets/devices/airpods-case.png'
import DeviceImageModel from '@/components/devices/DeviceImageModel.vue'
import IPhoneModel from '@/components/devices/IPhoneModel.vue'
import type { DeviceKind } from '@/types/battery.type'

const { kind, order } = defineProps<{
  kind: DeviceKind
  /** Position in the row: offsets the idle float and places the model in the glint wave */
  order: number
}>()

/** Spreads the floats of neighbouring models across one cycle */
const FLOAT_PHASE_STEP_MS = 1300
/** Lets the glint travel across the row as a wave instead of hitting every tile at once */
const GLINT_STAGGER_MS = 220

const imageSrc = computed((): string | null => {
  switch (kind) {
    case 'airpod_left':
      return airpodLeft
    case 'airpod_right':
      return airpodRight
    case 'airpods_case':
      return airpodsCase
    case 'iphone':
    case 'other':
      return null
    default: {
      const unhandled: never = kind
      throw new Error(`Unhandled device kind: ${unhandled}`)
    }
  }
})

const floatPhase = computed(() => ({ animationDelay: `-${order * FLOAT_PHASE_STEP_MS}ms` }))
const glintDelay = computed(() => ({ '--glint-delay': `${order * GLINT_STAGGER_MS}ms` }))
</script>

<template>
  <div class="relative flex size-full items-center justify-center" :style="glintDelay">
    <div
      class="absolute bottom-0 left-1/2 h-1 w-3/5 -translate-x-1/2 rounded-full bg-device-ground blur-[2px] animate-device-ground"
      :style="floatPhase"
    />
    <div class="h-[86%] animate-device-float transform-3d" :style="floatPhase">
      <!-- Own element for the glint turn: one element can only run one transform animation -->
      <div class="h-full animate-device-pop transform-3d">
        <DeviceImageModel v-if="imageSrc" :src="imageSrc" />
        <IPhoneModel v-else-if="kind === 'iphone'" />
        <Bluetooth v-else class="size-full text-ink-muted" aria-hidden="true" />
      </div>
    </div>
  </div>
</template>
