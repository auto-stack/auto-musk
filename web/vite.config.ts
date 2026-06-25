import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

// musk standalone web app. Dev server proxies /api → musk backend (:8080).
// In production, `musk serve` serves this dist/ via ServeDir.
export default defineConfig({
  plugins: [vue()],
  server: {
    port: 8090,
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:8080',
        changeOrigin: true,
      },
    },
  },
  build: {
    target: 'esnext',
    outDir: 'dist',
  },
})
