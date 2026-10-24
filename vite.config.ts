import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

// https://vite.dev/config/
export default defineConfig({
  plugins: [vue()],
  // Tauri 固定端口，避免 devUrl 失联
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
  },
})
