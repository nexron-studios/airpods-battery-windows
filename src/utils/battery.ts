import type { AirPodsNotice, BatteryTileProps, BatteryTone, DeviceKind } from '@/types/battery.type'
import type { AirPodsBattery, AirPodsStatus, DeviceBattery, Level } from '@/types/snapshot.type'

export const LOW_LEVEL_PERCENT = 20
export const STALE_AFTER_MS = 10 * 60 * 1000
const MS_PER_MINUTE = 60 * 1000
const EMPTY_AIRPODS: AirPodsBattery = { left: null, right: null, case: null }

export const toneOf = (level: Level | null): BatteryTone => {
  if (level === null) return 'unknown'
  if (level.isCharging) return 'ok'
  return level.percent <= LOW_LEVEL_PERCENT ? 'low' : 'ok'
}

export const isAirPodsStale = (airpods: AirPodsStatus, nowMs: number): boolean =>
  airpods.status !== 'seen' || nowMs - airpods.lastSeenMs > STALE_AFTER_MS

export const deviceKindOf = (name: string): DeviceKind => (/iphone/i.test(name) ? 'iphone' : 'other')

export const airPodsTilesOf = (airpods: AirPodsStatus, nowMs: number): BatteryTileProps[] => {
  const battery = airpods.status === 'seen' ? airpods.battery : EMPTY_AIRPODS
  const isStale = isAirPodsStale(airpods, nowMs)
  return [
    { id: 'airpod_left', kind: 'airpod_left', name: null, level: battery.left, isStale },
    { id: 'airpod_right', kind: 'airpod_right', name: null, level: battery.right, isStale },
    { id: 'airpods_case', kind: 'airpods_case', name: null, level: battery.case, isStale },
  ]
}

export const deviceTilesOf = (devices: DeviceBattery[]): BatteryTileProps[] =>
  devices.map(({ name, percent }) => ({
    id: `device:${name}`,
    kind: deviceKindOf(name),
    name,
    level: { percent, isCharging: false },
    isStale: false,
  }))

export const airPodsNoticeOf = (airpods: AirPodsStatus, nowMs: number): AirPodsNotice => {
  switch (airpods.status) {
    case 'missing_keys':
      return { kind: 'missing_keys' }
    case 'unavailable':
      return { kind: 'unavailable' }
    case 'waiting':
      return { kind: 'waiting' }
    case 'seen':
      if (!isAirPodsStale(airpods, nowMs)) return { kind: 'none' }
      return { kind: 'stale', minutes: Math.floor((nowMs - airpods.lastSeenMs) / MS_PER_MINUTE) }
    default: {
      const unhandled: never = airpods
      throw new Error(`Unhandled AirPods status: ${JSON.stringify(unhandled)}`)
    }
  }
}
