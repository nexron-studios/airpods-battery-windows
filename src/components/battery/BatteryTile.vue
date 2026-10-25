<script setup lang="ts">
import { Zap } from '@lucide/vue'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import BatteryRing from '@/components/battery/BatteryRing.vue'
import DeviceModel from '@/components/devices/DeviceModel.vue'
import type { BatteryTileProps } from '@/types/battery.type'
import { toneOf } from '@/utils/battery'
import { formatPercent } from '@/utils/format'

const { kind, name, level, isStale, order } = defineProps<Omit<BatteryTileProps, 'id'> & { order: number }>()

/** Lets the turn travel across the row as a wave instead of every model spinning at once */
const TURN_STAGGER_MS = 220

const { t, locale } = useI18n()

const label = computed(() => name ?? t(`devices.${kind}`))
const valueText = computed(() => (level ? formatPercent(locale.value, level.percent) : '–'))
const isCharging = computed(() => level?.isCharging ?? false)

// Read by the turn, bloom and ring glint animations of everything in the tile
const turnDelay = computed(() => ({ '--turn-delay': `${order * TURN_STAGGER_MS}ms` }))

const ariaLabel = computed(() => {
  if (!level) return t('tile.level_unknown', { name: label.value })
  const key = isCharging.value ? 'tile.level_charging' : 'tile.level'
  return t(key, { name: label.value, value: valueText.value })
})
</script>

<template>
  <div class="flex w-16 flex-col items-center gap-1" role="img" :aria-label="ariaLabel">
    <div class="relative size-14" :class="{ 'opacity-45': isStale }" :style="turnDelay">
      <BatteryRing
        class="absolute inset-0 size-full"
        :percent="level?.percent ?? null"
        :tone="toneOf(level)"
        :hasGlint="kind !== 'other'"
      />
      <div class="absolute inset-1">
        <DeviceModel :kind :order />
      </div>
      <div
        v-if="isCharging"
        class="absolute -top-0.5 -right-0.5 grid size-4.5 place-items-center rounded-full bg-surface text-level-ok"
      >
        <Zap class="size-3 animate-charge-pulse fill-current" aria-hidden="true" />
      </div>
    </div>
    <span class="text-value font-semibold tabular-nums" :class="isStale ? 'text-ink-muted' : 'text-ink'">
      {{ valueText }}
    </span>
    <span class="max-w-full truncate text-caption text-ink-muted">{{ label }}</span>
  </div>
</template>
