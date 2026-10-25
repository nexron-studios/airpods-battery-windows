<script setup lang="ts">
import { useIntervalFn, useNow } from '@vueuse/core'
import { computed, useTemplateRef } from 'vue'
import { useI18n } from 'vue-i18n'
import BatteryTile from '@/components/battery/BatteryTile.vue'
import { useBatterySnapshot } from '@/composables/useBatterySnapshot'
import { useWidgetMenu } from '@/composables/useWidgetMenu'
import { useWidgetWindow } from '@/composables/useWidgetWindow'
import type { AirPodsStatus } from '@/types/snapshot.type'
import { airPodsNoticeOf, airPodsTilesOf, deviceTilesOf } from '@/utils/battery'

/** Staleness is measured in minutes, so the clock does not need to tick faster */
const CLOCK_INTERVAL_MS = 15_000
const WAITING: AirPodsStatus = { status: 'waiting' }

const root = useTemplateRef<HTMLElement>('root')

const { t } = useI18n()
const now = useNow({ scheduler: (tick) => useIntervalFn(tick, CLOCK_INTERVAL_MS) })
const { snapshot, hasFailed } = useBatterySnapshot()
const { startDrag } = useWidgetWindow(root)
const { openMenu } = useWidgetMenu()

const airpods = computed(() => snapshot.value?.airpods ?? WAITING)
const airPodsTiles = computed(() => airPodsTilesOf(airpods.value, now.value.getTime()))
const deviceTiles = computed(() => deviceTilesOf(snapshot.value?.devices ?? []))

const noticeText = computed((): string | null => {
  if (hasFailed.value) return t('errors.snapshot_failed')
  const notice = airPodsNoticeOf(airpods.value, now.value.getTime())
  switch (notice.kind) {
    case 'none':
      return null
    case 'missing_keys':
    case 'invalid_config':
    case 'unavailable':
    case 'waiting':
    case 'parts_quiet':
      return t(`notice.${notice.kind}`)
    case 'stale':
      return t('notice.stale', notice.minutes)
    default: {
      const unhandled: never = notice
      throw new Error(`Unhandled notice: ${JSON.stringify(unhandled)}`)
    }
  }
})
</script>

<template>
  <!-- The padding is room for the card shadow inside the transparent window -->
  <div ref="root" class="w-max p-4">
    <main
      class="rounded-card border border-surface-edge bg-surface px-3 pt-3.5 pb-3 shadow-card"
      @pointerdown="startDrag"
      @contextmenu.prevent="openMenu"
    >
      <div class="flex items-start">
        <BatteryTile
          v-for="(tile, index) in airPodsTiles"
          :key="tile.id"
          v-bind="tile"
          :order="index"
        />
        <div v-if="deviceTiles.length" class="mx-1.5 mt-2 h-10 w-px bg-surface-edge" aria-hidden="true" />
        <BatteryTile
          v-for="(tile, index) in deviceTiles"
          :key="tile.id"
          v-bind="tile"
          :order="index + airPodsTiles.length"
        />
      </div>
      <p v-if="noticeText" class="mt-2 w-0 min-w-full text-center text-caption text-ink-muted">
        {{ noticeText }}
      </p>
    </main>
  </div>
</template>
