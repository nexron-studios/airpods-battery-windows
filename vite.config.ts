import { fileURLToPath, URL } from 'node:url'
import process from 'node:process'
import tailwindcss from '@tailwindcss/vite'
import vue from '@vitejs/plugin-vue'
import { defineConfig } from 'vite'

// Own port pair instead of Tauri's default 1420, which Mellow and Mimi already use
// (see "Lokale Ports" in the vault)
const DEV_PORT = 1440
const HMR_PORT = 1441
const host = process.env.TAURI_DEV_HOST

export default defineConfig({
  plugins: [vue(), tailwindcss()],
  resolve: {
    alias: { '@': fileURLToPath(new URL('./src', import.meta.url)) },
  },
  // Keep Rust errors visible in the terminal
  clearScreen: false,
  server: {
    port: DEV_PORT,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: 'ws', host, port: HMR_PORT } : undefined,
    watch: { ignored: ['**/src-tauri/**'] },
  },
  test: {
    environment: 'node',
  },
})
