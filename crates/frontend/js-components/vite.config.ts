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
        "lib/create-calendar-form.ts",
        "lib/edit-calendar-form.ts",
        "lib/create-addressbook-form.ts",
        "lib/edit-addressbook-form.ts",
        "lib/delete-button.ts",
      ],
      output: {
        dir: "../public/assets/js/",
        format: "es",
        manualChunks: {
          lit: ["lit"],
          // webdav: ["webdav"],
        }
      }
    },
  },
})
