import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import type { Plugin } from 'vite'

// Build the musk config page as a standalone ESM bundle (no federation).
// The host (auto-os-config) loads it via dynamic import() from its real URL.
//
// `vue` is EXTERNAL: the bundle emits a bare `import 'vue'`, which the host's
// <script type="importmap"> resolves to the host's single Vue copy. Sharing one
// Vue runtime is what lets the component's reactivity (ref/onMounted/v-if) work
// when rendered inside the host — two separate Vue copies break reactivity.
//
// CSS is INLINED into the JS bundle (injected as a <style> tag at runtime)
// because the remote is loaded via bare import(), so the host never sees/loads
// a separate style.css — it would be orphaned and the component would render
// unstyled. (This was a latent bug once Vue was externalized; the old bundled-
// Vue build happened to inline scoped CSS.)

/** Inline all CSS emitted by the build into the JS as a runtime <style> injection.
 *  Collect CSS during `transform`, suppress the default CSS output, then inject
 *  it at the top of every JS chunk in `renderChunk`. */
function cssInjectedByJs(): Plugin {
  const cssChunks: string[] = []
  return {
    name: 'css-injected-by-js',
    apply: 'build',
    // Collect CSS content and mark it as pure JS so Vite doesn't emit a file.
    transform(code, id) {
      if (id.endsWith('.css')) {
        cssChunks.push(code)
        return { code: '', map: null }
      }
      return null
    },
    renderChunk(code) {
      if (cssChunks.length === 0) return null
      const css = cssChunks.join('\n')
      const escaped = JSON.stringify(css)
      const injector = `try{var s=document.createElement('style');s.textContent=${escaped};document.head.appendChild(s);}catch(e){}\n`
      return { code: injector + code, map: null }
    },
  }
}

export default defineConfig({
  plugins: [vue(), cssInjectedByJs()],
  build: {
    target: 'esnext',
    minify: true,
    lib: {
      // Standalone ESM entry bundles, one per auto-os-config module:
      //   config-page.js        → "AI Agent" module (modes + professions)
      //   skills-config-page.js → "AI Skills" module (skill registry)
      //   roles-config-page.js  → "AI Roles" module (Plan 004)
      //   app-config-page.js    → "AI Musk" module (runtime: daemon conn etc.)
      // Each keeps `vue` external and inlines its CSS, so each is self-contained
      // and shares the host's single Vue runtime via the import map.
      entry: {
        'config-page': './src/agents-config-page.vue',
        'skills-config-page': './src/skills-config-page.vue',
        'roles-config-page': './src/roles-config-page.vue',
        'app-config-page': './src/app-config-page.vue',
      },
      formats: ['es'],
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
