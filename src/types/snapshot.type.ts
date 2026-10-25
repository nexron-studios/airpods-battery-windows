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

/** When each part last reported, in ms since the epoch; `null` if it never has. */
export interface PartsSeen {
  left: number | null
  right: number | null
  case: number | null
}

export type AirPodsStatus =
  | { status: 'missing_keys' }
  | { status: 'invalid_config' }
  | { status: 'unavailable' }
  | { status: 'waiting' }
  | { status: 'seen'; battery: AirPodsBattery; partsSeenMs: PartsSeen; lastSeenMs: number }

/** A Bluetooth device whose battery Windows itself knows, e.g. the iPhone. */
export interface DeviceBattery {
  name: string
  percent: number
}

export interface Snapshot {
  airpods: AirPodsStatus
  devices: DeviceBattery[]
}
