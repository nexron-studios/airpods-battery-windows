import type { UnlistenFn } from '@tauri-apps/api/event'
import { onMounted, onUnmounted, ref, shallowRef } from 'vue'
import { BatteryService } from '@/api/services/battery.service'
import type { Snapshot } from '@/types/snapshot.type'

/** The current battery snapshot from the Rust side, live via its change event. */
export const useBatterySnapshot = () => {
  const snapshot = shallowRef<Snapshot | null>(null)
  const hasFailed = ref(false)
  const unlisten = shallowRef<UnlistenFn | null>(null)

  const loadSnapshot = async () => {
    // Subscribe first, so no change between the two calls is lost
    unlisten.value = await BatteryService.onSnapshotChanged((next) => {
      snapshot.value = next
      hasFailed.value = false
    })
    try {
      snapshot.value = await BatteryService.getSnapshot()
    } catch (error) {
      console.error('get_snapshot failed', error)
      hasFailed.value = true
    }
  }

  onMounted(loadSnapshot)
  onUnmounted(() => unlisten.value?.())

  return { snapshot, hasFailed }
}
