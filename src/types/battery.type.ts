import type { Level } from '@/types/snapshot.type'

export const deviceKindList = ['airpod_left', 'airpod_right', 'airpods_case', 'iphone', 'other'] as const
export type DeviceKind = (typeof deviceKindList)[number]

export const batteryToneList = ['ok', 'low', 'unknown'] as const
export type BatteryTone = (typeof batteryToneList)[number]

export interface BatteryTileProps {
  id: string
  kind: DeviceKind
  /** Shown as the label for Windows devices; AirPods parts use a translated label */
  name: string | null
  level: Level | null
  /** The value is older than the stale threshold and drawn dimmed */
  isStale: boolean
}

export type AirPodsNotice =
  | { kind: 'none' }
  | { kind: 'missing_keys' }
  | { kind: 'invalid_config' }
  | { kind: 'unavailable' }
  | { kind: 'waiting' }
  | { kind: 'parts_quiet' }
  | { kind: 'stale'; minutes: number }
