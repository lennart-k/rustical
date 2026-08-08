licenses:
  cargo about generate about.hbs > crates/frontend/public/assets/licenses.html

frontend-dev:
  cd crates/frontend/js-components && deno task dev

frontend-build:
  cd crates/frontend/js-components && deno task build

docs:
  mkdocs build --strict

docs-dev:
  mkdocs serve

coverage:
  cargo tarpaulin --workspace --exclude xml_derive

test-setup:
  #!/usr/bin/env bash
  echo "password" | cargo run principals create user --password --overwrite
  export APP_TOKEN=`cargo run principals app-token create user --name "Test Token"`
  echo $APP_TOKEN

