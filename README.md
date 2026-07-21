# panza

[![Crates.io](https://img.shields.io/crates/v/panza.svg)](https://crates.io/crates/panza)
[![docs.rs](https://img.shields.io/docsrs/panza)](https://docs.rs/panza)
![Build](https://github.com/Latka-Industries/panza/workflows/Build/badge.svg)
![Rust](https://img.shields.io/badge/rust-1.95-orange.svg)

*The squire for your CLI — clap `serve`, bind, health, static; you bring the quest.*

Named for **Sancho Panza**: each Latka (or other) CLI is the knight — domain API + frontend — and panza handles the practical serving.

**In active development — expect breaking changes.**

## What it owns

| Piece | Behavior |
| --- | --- |
| `ServeArgs` | clap: `--host` (default `127.0.0.1`), `--port` / `-p` (default `8787`), `--open` |
| `ServeMeta` | `service` + `version` for health JSON |
| `run` | tokio + axum: bind, Ctrl-C shutdown, log listen URL |
| `GET /health` | `{ "ok", "service", "version" }` |
| `StaticMount` | `None` / `Dir` / `Embedded` with SPA `index.html` fallback |
| Router merge | host supplies an `axum::Router`; panza merges it |

## What it does **not** own

- Domain routes / DB access
- Frontend framework or shared UI
- Auth (warns if bind ≠ loopback)

## Usage

```rust
use clap::Parser;
use panza::{ServeArgs, ServeMeta, StaticMount, run};

#[derive(Parser)]
struct ServeCli {
    #[command(flatten)]
    serve: ServeArgs,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = ServeCli::parse();
    let api = axum::Router::new(); // host routes here
    run(
        ServeMeta {
            service: "my-tool",
            version: env!("CARGO_PKG_VERSION"),
        },
        cli.serve,
        api,
        StaticMount::None, // or Dir(ui_dist) / Embedded(...)
    )
    .await
}
```

For tests or custom bind loops, use [`serve_router`](https://docs.rs/panza/latest/panza/fn.serve_router.html) instead of `run`.

## License

MIT OR Apache-2.0
