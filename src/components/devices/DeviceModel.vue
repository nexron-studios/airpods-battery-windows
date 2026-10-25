<script setup lang="ts">
import { Bluetooth } from '@lucide/vue'
import { computed } from 'vue'
import airpodLeft from '@/assets/devices/airpod-left.png'
import airpodRight from '@/assets/devices/airpod-right.png'
import airpodsCase from '@/assets/devices/airpods-case.png'
import DeviceImageModel from '@/components/devices/DeviceImageModel.vue'
import IPhoneModel from '@/components/devices/IPhoneModel.vue'
import type { DeviceKind } from '@/types/battery.type'

const { kind, phaseMs } = defineProps<{
  kind: DeviceKind
  /** Offsets the idle loop, so neighbouring models do not move in lockstep */
  phaseMs: number
}>()

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

const phase = computed(() => ({ animationDelay: `-${phaseMs}ms` }))
</script>

<template>
  <div class="relative flex size-full items-center justify-center">
    <div
      class="absolute bottom-0 left-1/2 h-1 w-3/5 -translate-x-1/2 rounded-full bg-device-ground blur-[2px] animate-device-ground"
      :style="phase"
    />
    <div class="h-[86%] animate-device-float transform-3d" :style="phase">
      <DeviceImageModel v-if="imageSrc" :src="imageSrc" />
      <IPhoneModel v-else-if="kind === 'iphone'" />
      <Bluetooth v-else class="size-full text-ink-muted" aria-hidden="true" />
    </div>
  </div>
</template>
