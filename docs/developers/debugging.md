# Debugging

Set the log level with following environment variables:

```sh
RUST_LOG="debug"
RUST_BACKTRACE=1
```

RustiCal also supports exporting opentelemetry traces to inspect with tools like [Jaeger](https://www.jaegertracing.io/).
To enable you need to compile with the `opentelemtry` (or `debug`) feature and enable opentelemetry in the config with

```toml
[tracing]
opentelemetry = true
```
