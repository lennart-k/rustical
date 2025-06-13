import { defineConfig } from 'vite'

export default defineConfig({
  optimizeDeps: {
    // include: ["lit"]
  },
  build: {
    copyPublicDir: false,
    lib: {
      entry: 'lib/index.ts',
      formats: ['es'],
    },

    rollupOptions: {
      input: [
        "lib/create-calendar-form.ts",
        "lib/create-addressbook-form.ts",
      ],
      output: {
        dir: "../public/assets/js/",
        format: "es",
        manualChunks: {
          lit: ["lit"],
          webdav: ["webdav"],
        }
      }
    },
  },
})
