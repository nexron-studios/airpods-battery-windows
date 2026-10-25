/** Mirrors `monitor::Snapshot` on the Rust side. */
export interface Level {
  percent: number
  isCharging: boolean
}

/** A part is `null` until an advert has reported it at least once. */
export interface AirPodsBattery {
  left: Level | null
  right: Level | null
  case: Level | null
}

export type AirPodsStatus =
  | { status: 'missing_keys' }
  | { status: 'unavailable' }
  | { status: 'waiting' }
  | { status: 'seen'; battery: AirPodsBattery; lastSeenMs: number }

/** A Bluetooth device whose battery Windows itself knows, e.g. the iPhone. */
export interface DeviceBattery {
  name: string
  percent: number
}

export interface Snapshot {
  airpods: AirPodsStatus
  devices: DeviceBattery[]
}
