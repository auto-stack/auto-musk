import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

// musk web app (ported from auto-forge frontend). Dev server proxies /api →
// musk backend (:8888). In production, `musk serve` serves dist/ via ServeDir.
export default defineConfig({
  base: './',
  plugins: [vue()],
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
    },
  },
  optimizeDeps: {
    include: ['vue', 'vue-i18n', 'marked', 'mermaid', 'lucide-vue-next'],
  },
  server: {
    port: 3333,
    host: '127.0.0.1',
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:8888',
        changeOrigin: true,
      },
    },
  },
})
