licenses:
  cargo about generate about.hbs > crates/frontend/public/assets/licenses.html

frontend-dev:
  cd crates/frontend/js-components && deno task dev

frontend-build:
  cd crates/frontend/js-components && deno task build

docs:
  mkdocs build

docs-dev:
  mkdocs serve

coverage:
  cargo tarpaulin --workspace --exclude xml_derive
