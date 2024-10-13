import { defineConfig } from 'vite'
import { globSync } from 'glob'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

export default defineConfig({
  root: "src",
  base: "/frontend",
  build: {
    modulePreload: {
      polyfill: false
    },
    minify: 'esbuild',
    rollupOptions: {
      input: Object.fromEntries(
        globSync('src/templates/**/*.html').map(file => [
          path.relative(
            'src',
            file.slice(0, file.length - path.extname(file).length)
          ),
          // This expands the relative paths to absolute paths, so e.g.
          // src/nested/foo becomes /project/src/nested/foo.js
          fileURLToPath(new URL(file, import.meta.url))
        ]),
      ),
      output: {
        format: 'esm',
        dir: 'dist'
      }
    },
  },
  resolve: {
    alias: {
      "$lib": path.resolve(__dirname, "./src/lib")
    }
  },
  plugins: [
    {
      name: "jinja-fix-imports",
      transformIndexHtml(html) {
        let startLocation = html.search("{% extends")
        if (startLocation === -1) { return html }

        let strayImports = html.substring(0, startLocation)
        if (strayImports.trim().length === 0) {
          // Nothin broken :)
          return html
        }
        let template = html.substring(startLocation)

        let importTarget = '{% block imports %}\n'
        if (template.search(importTarget) === -1) {
          throw new Error("Cannot properly place imports :(")
        }
        let importLocation = template.search(importTarget) + importTarget.length

        return (
          template.substring(0, importLocation)
          + strayImports
          + template.substring(importLocation)
        )
      }
    }
  ],
})
