import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

// Build the musk config page as a standalone ESM bundle (no federation).
// The host (auto-os-config) loads it via dynamic import() from its real URL.
//
// `vue` is EXTERNAL: the bundle emits a bare `import 'vue'`, which the host's
// <script type="importmap"> resolves to the host's single Vue copy. Sharing one
// Vue runtime is what lets the component's reactivity (ref/onMounted/v-if) work
// when rendered inside the host — two separate Vue copies break reactivity.
export default defineConfig({
  plugins: [vue()],
  build: {
    target: 'esnext',
    minify: true,
    lib: {
      entry: './src/config-page.vue',
      formats: ['es'],
      fileName: 'config-page',
    },
    rollupOptions: {
      // Externalize vue so the bundle emits `import { ref } from 'vue'` (a bare
      // specifier). The host's <script type="importmap"> resolves it to the
      // host's single Vue copy — sharing one Vue runtime keeps reactivity alive.
      //
      // The import map URL must match the EXACT URL the host's own `import
      // 'vue'` resolves to (via vite resolve.alias), otherwise the browser
      // loads two Vue copies and reactivity silently breaks.
      external: ['vue'],
    },
    outDir: '../backend/crates/musk/frontend-dist',
    emptyOutDir: true,
  },
})
