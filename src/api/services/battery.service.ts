import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { Snapshot } from '@/types/snapshot.type'

/** Emitted by `monitor::update` whenever a level or the AirPods status changes. */
const SNAPSHOT_EVENT = 'snapshot-changed'

export const BatteryService = {
  getSnapshot: (): Promise<Snapshot> => invoke<Snapshot>('get_snapshot'),
  onSnapshotChanged: (handler: (snapshot: Snapshot) => void): Promise<UnlistenFn> =>
    listen<Snapshot>(SNAPSHOT_EVENT, (event) => handler(event.payload)),
}
