import type { AirPodsNotice, BatteryTileProps, BatteryTone, DeviceKind } from '@/types/battery.type'
import type { AirPodsBattery, AirPodsStatus, DeviceBattery, Level, PartsSeen } from '@/types/snapshot.type'

export const LOW_LEVEL_PERCENT = 20
export const STALE_AFTER_MS = 10 * 60 * 1000
const MS_PER_MINUTE = 60 * 1000
const EMPTY_AIRPODS: AirPodsBattery = { left: null, right: null, case: null }
const NEVER_SEEN: PartsSeen = { left: null, right: null, case: null }

export const toneOf = (level: Level | null): BatteryTone => {
  if (level === null) return 'unknown'
  if (level.isCharging) return 'ok'
  return level.percent <= LOW_LEVEL_PERCENT ? 'low' : 'ok'
}

export const isAirPodsStale = (airpods: AirPodsStatus, nowMs: number): boolean =>
  airpods.status !== 'seen' || nowMs - airpods.lastSeenMs > STALE_AFTER_MS

export const deviceKindOf = (name: string): DeviceKind => (/iphone/i.test(name) ? 'iphone' : 'other')

const isPartStale = (seenMs: number | null, nowMs: number): boolean =>
  seenMs === null || nowMs - seenMs > STALE_AFTER_MS

/** Each part dims on its own: a pod in the closed case goes quiet while the other keeps reporting. */
export const airPodsTilesOf = (airpods: AirPodsStatus, nowMs: number): BatteryTileProps[] => {
  const battery = airpods.status === 'seen' ? airpods.battery : EMPTY_AIRPODS
  const seen = airpods.status === 'seen' ? airpods.partsSeenMs : NEVER_SEEN
  const tileOf = (kind: DeviceKind, level: Level | null, seenMs: number | null): BatteryTileProps => ({
    id: kind,
    kind,
    name: null,
    level,
    isStale: isPartStale(seenMs, nowMs),
  })
  return [
    tileOf('airpod_left', battery.left, seen.left),
    tileOf('airpod_right', battery.right, seen.right),
    tileOf('airpods_case', battery.case, seen.case),
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
    case 'invalid_config':
      return { kind: 'invalid_config' }
    case 'unavailable':
      return { kind: 'unavailable' }
    case 'waiting':
      return { kind: 'waiting' }
    case 'seen': {
      if (isAirPodsStale(airpods, nowMs)) {
        return { kind: 'stale', minutes: Math.floor((nowMs - airpods.lastSeenMs) / MS_PER_MINUTE) }
      }
      // The case only reports while a pod sits in it, so a quiet case alone is normal
      const { left, right } = airpods.partsSeenMs
      const isAPodQuiet = [left, right].some((seenMs) => isPartStale(seenMs, nowMs))
      return isAPodQuiet ? { kind: 'parts_quiet' } : { kind: 'none' }
    }
    default: {
      const unhandled: never = airpods
      throw new Error(`Unhandled AirPods status: ${JSON.stringify(unhandled)}`)
    }
  }
}
