import { LogicalSize } from '@tauri-apps/api/dpi'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { saveWindowState, StateFlags } from '@tauri-apps/plugin-window-state'
import { useDebounceFn, useResizeObserver } from '@vueuse/core'
import { onMounted, onUnmounted, shallowRef, type Ref } from 'vue'

const PRIMARY_BUTTON = 0
/** Dragging fires a move event per pixel; save once the widget has come to rest. */
const SAVE_POSITION_DELAY_MS = 500

const logFailure = (action: string) => (error: unknown) => console.error(`${action} failed`, error)

/** Frameless window behaviour: fit the window to `root`, drag anywhere, remember the position. */
export const useWidgetWindow = (root: Ref<HTMLElement | null>) => {
  const appWindow = getCurrentWindow()
  const unlistenMoved = shallowRef<UnlistenFn | null>(null)

  const savePosition = useDebounceFn(
    () => saveWindowState(StateFlags.POSITION).catch(logFailure('saving the position')),
    SAVE_POSITION_DELAY_MS,
  )

  const startDrag = (event: PointerEvent) => {
    if (event.button !== PRIMARY_BUTTON) return
    appWindow.startDragging().catch(logFailure('dragging'))
  }

  useResizeObserver(root, ([entry]) => {
    const [box] = entry.borderBoxSize
    const size = new LogicalSize(Math.ceil(box.inlineSize), Math.ceil(box.blockSize))
    appWindow.setSize(size).catch(logFailure('resizing the window'))
  })

  onMounted(async () => {
    unlistenMoved.value = await appWindow.onMoved(savePosition)
  })
  onUnmounted(() => unlistenMoved.value?.())

  return { startDrag }
}
