# DNeyeS

DNeyeS is a DNS and HTTP monitoring toolkit written in Rust. It can collect
resolution data from world-wide resolvers, persist measurements either as
NDJSON files or directly into ClickHouse and exposes the collected metrics via
an OpenAPI documented REST API. A lightweight React dashboard is provided to
visualise ClickHouse backed metrics.

> **Status**: The project is still young but the build, packaging and
> distribution story is now production ready.

## Features

- ⚡️ Tokio powered asynchronous DNS and HTTP checks.
- 🗄️ Configurable persistence backends (NDJSON files, ClickHouse or both).
- 📡 REST API implemented with Axum + OpenAPI documentation via Utoipa.
- 📊 React dashboard that consumes the REST API for quick visualisation.
- 📦 Debian packaging helper that installs a systemd unit on Debian/Ubuntu.

## Getting started

### Configuration

DNeyeS reads configuration from a YAML file (default: `config.yaml`). The
structure is documented inline in [`src/config.rs`](src/config.rs). The default
file shipped with the repository contains the full country catalogue. Important
sections:

- `dns`: resolver source, default country list, concurrency and timeout.
- `output`: persistence mode (`file`, `clickhouse` or `both`) plus the per-mode
  options.
- `api`: bind address, page size and CORS policy for the REST server.

You can override the configuration path with `--config` or via the
`DNEYES_CONFIG` environment variable.

### CLI usage

```bash
# DNS monitoring run (writes according to output.mode)
cargo run -- dns

# HTTP uptime checks
cargo run -- http

# REST API only
cargo run -- api
```

The Makefile contains shortcuts:

```bash
make build      # Debug build
make release    # Release build
make lint       # fmt + clippy
make test       # cargo test
make api        # Start the REST API with the default config
```

### REST API & OpenAPI

Run `cargo run -- api` (or `make api`) and navigate to
`http://localhost:8080/docs` to explore the Swagger UI. The primary endpoint is
`GET /api/v1/dns/resolutions` which accepts query parameters for domain,
country, status and time range filtering. Health checks are exposed via
`GET /healthz`.

### ClickHouse schema

On start DNeyeS can ensure the ClickHouse database and tables exist (controlled
via `output.clickhouse.ensure_schema`). The schema consists of two tables:
`dns_resolutions` and `http_availability` storing the normalised events defined
in [`src/telemetry/models.rs`](src/telemetry/models.rs).

### Debian packaging & systemd unit

The repository ships with a helper script that produces a `.deb` package. It
installs the DNeyeS binary, default configuration and a `systemd` unit that runs
`dneyes dns` on boot.

```bash
make deb
ls target/debian
```

The `postinst` script creates a dedicated `dneyes` user, reloads systemd and
starts the service. Adjust `/etc/dneyes/config.yaml` after installation.

### Dashboard

The React dashboard lives in `web/dashboard`. It consumes the REST API and
plots DNS latency and success rates.

```bash
cd web/dashboard
npm install
npm run dev
```

Set `VITE_API_BASE` to point to the REST server when running behind a proxy.

## Development

- Format: `cargo fmt`
- Lint: `cargo clippy --all-targets --all-features`
- Tests: `cargo test`

The repository includes extensive inline Rust documentation. Run `cargo doc`
for API docs if needed.

## License

Licensed under the MIT License. See [LICENSE](LICENSE).
