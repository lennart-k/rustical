# Debugging

Set the log level with following environment variables:

```sh
RUST_LOG="debug"
RUST_BACKTRACE=1
```

RustiCal also supports exporting opentelemetry traces to inspect with tools like [Jaeger](https://www.jaegertracing.io/).
You can export traces to an opentelemetry endpoint via grpc by setting

```toml
[tracing]
endpoint = "grpc://localhost:4317"
```
