import { defineConfig } from 'vitest/config'
import vue from '@vitejs/plugin-vue'

export default defineConfig({
  plugins: [vue()],
  clearScreen: false,
  server: { port: 1425, strictPort: true, watch: { ignored: ['**/src-tauri/**'] } },
  test: { environment: 'happy-dom', include: ['src/**/*.test.ts'] },
})
