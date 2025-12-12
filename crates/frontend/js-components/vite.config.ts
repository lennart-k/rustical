import { defineConfig } from 'vite'

export default defineConfig({
  build: {
    minify: false,
    modulePreload: {
      polyfill: false
    },
    copyPublicDir: false,
    lib: {
      entry: 'lib/index.ts',
      formats: ['es'],
    },

    rollupOptions: {
      input: [
        "lib/bundle.ts",
      ],
      output: {
        dir: "../public/assets/js/",
        format: "es",
        // manualChunks: {
        //   lit: ["lit"],
        // }
      }
    },
  },
})
