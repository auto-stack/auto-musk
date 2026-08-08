import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

// musk web app. Dev server proxies /api → musk backend.
// In production, `musk serve` serves dist/ via ServeDir.
//
// 所有地址均可通过环境变量配置（与 Auto 版 gen/front/vue 约定一致）：
//   MUSK_FRONT_HOST   — 前端 dev server 绑定地址（默认 127.0.0.1）
//   MUSK_FRONT_PORT   — 前端 dev server 端口（默认 3333）
//   MUSK_BACKEND_HOST — 后端地址（默认 127.0.0.1，与 musk serve --addr 对齐）
//   MUSK_BACKEND_PORT — 后端端口（默认 8080，与 musk serve --addr 对齐）
// 示例：MUSK_BACKEND_PORT=9090 MUSK_FRONT_PORT=4444 npm run dev
const backendHost = process.env.MUSK_BACKEND_HOST || '127.0.0.1'
const backendPort = process.env.MUSK_BACKEND_PORT || '8080'
const frontendHost = process.env.MUSK_FRONT_HOST || '127.0.0.1'
const frontendPort = Number(process.env.MUSK_FRONT_PORT || 3333)

export default defineConfig({
  base: '/',
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
    port: frontendPort,
    host: frontendHost,
    proxy: {
      '/api': {
        target: `http://${backendHost}:${backendPort}`,
        changeOrigin: true,
      },
    },
  },
})
