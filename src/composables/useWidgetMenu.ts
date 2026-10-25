import { defaultWindowIcon } from '@tauri-apps/api/app'
import { CheckMenuItem, Menu, MenuItem, PredefinedMenuItem } from '@tauri-apps/api/menu'
import { TrayIcon } from '@tauri-apps/api/tray'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { disable, enable, isEnabled } from '@tauri-apps/plugin-autostart'
import { onMounted, shallowRef } from 'vue'
import { useI18n } from 'vue-i18n'

const TRAY_ID = 'battery-widget'

/** One native menu, shown on right-click in the widget and from the tray icon. */
export const useWidgetMenu = () => {
  const menu = shallowRef<Menu | null>(null)
  const { t } = useI18n()

  const toggleAutostart = async (item: CheckMenuItem) => {
    const wasEnabled = await isEnabled()
    await (wasEnabled ? disable() : enable())
    await item.setChecked(!wasEnabled)
  }

  const buildMenu = async (): Promise<Menu> => {
    const autostart: CheckMenuItem = await CheckMenuItem.new({
      text: t('menu.autostart'),
      checked: await isEnabled(),
      action: () => toggleAutostart(autostart).catch((error) => console.error('toggling autostart failed', error)),
    })
    const quit = await MenuItem.new({ text: t('menu.quit'), action: () => getCurrentWindow().close() })
    const separator = await PredefinedMenuItem.new({ item: 'Separator' })
    return Menu.new({ items: [autostart, separator, quit] })
  }

  // Reuses the tray icon across hot reloads instead of stacking new ones
  const showTrayIcon = async (trayMenu: Menu) => {
    const existing = await TrayIcon.getById(TRAY_ID)
    if (existing) return existing.setMenu(trayMenu)
    await TrayIcon.new({
      id: TRAY_ID,
      icon: (await defaultWindowIcon()) ?? undefined,
      tooltip: t('menu.tooltip'),
      menu: trayMenu,
      showMenuOnLeftClick: true,
    })
  }

  const openMenu = () => {
    menu.value?.popup().catch((error) => console.error('opening the menu failed', error))
  }

  onMounted(async () => {
    menu.value = await buildMenu()
    await showTrayIcon(menu.value)
  })

  return { openMenu }
}
