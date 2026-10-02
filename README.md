<div align="center">
  <img src="frontend/src/lib/assets/icon-dark.svg" alt="Monolai Logo" width="64" height="64" />

**Monolai**

**A lightweight, cross-platform application for managing and running large language models locally.**

[![Release](https://img.shields.io/github/v/release/odevsa/monolai?label=Release&style=flat-square&color=blue&logo=github)](https://github.com/odevsa/monolai/releases/latest)
![Rust](https://img.shields.io/badge/Rust-1.80%2B-orange.svg?logo=rust&style=flat-square)
![Svelte](https://img.shields.io/badge/Svelte-5-FF3E00.svg?logo=svelte&style=flat-square)
![SQLite](https://img.shields.io/badge/SQLite-FTS5-003B57.svg?logo=sqlite&style=flat-square)
![Docker](https://img.shields.io/badge/Docker-Ready-2496ED.svg?logo=docker&style=flat-square)

[Features](#features) • [Quick Start](#quick-start) • [Development](#development) • [Tech Stack](#tech-stack)

</div>

## Features

- **Real-time Dashboard**: Live CPU, RAM, and GPU utilization metrics streaming.
- **Model Management**: Download, manage, swap, and run local LLMs on-demand.
- **Hardware Acceleration**: Automatic detection and support for Vulkan, CUDA, and ROCm.
- **OpenAPI & Swagger UI**: Interactive API documentation at `/api/swagger`.
- **OpenAI Compatible**: Drop-in `/v1/chat/completions` proxy for standard OpenAI client libraries.
- **Modern Web UI**: Sleek SvelteKit 5 interface with multi-theme support.

## Quick Start

### Docker

Run with Docker:

```bash
docker run -d -p 8080:3000 --name monolai odevsa/monolai
```

Or using Docker Compose:

```bash
docker compose up -d
```

Access the application at `http://localhost:8080`.

## Development

Prerequisites:

- [Rust](https://rustup.rs/) (1.80+)
- [Node.js](https://nodejs.org/) (20+)
- GNU Make

### Commands

| Command                | Description                                                |
| :--------------------- | :--------------------------------------------------------- |
| `make install`         | Install frontend dependencies and verify Rust backend      |
| `make dev`             | Start frontend and backend concurrently                    |
| `make dev-host`        | Start frontend (`--host`) and backend concurrently         |
| `make build`           | Build frontend assets and compile backend release binary   |
| `make clean`           | Remove build artifacts                                     |
| `make version [x.y.z]` | Synchronize project version across all manifests and files |
| `make help`            | Display available targets                                  |

## Tech Stack

- **Backend**: Rust (Axum, Tokio, SQLx, RustEmbed)
- **Frontend**: SvelteKit 5, TypeScript, Tailwind CSS v4
- **Database**: SQLite (WAL mode + FTS5)
- **Runtime Engines**: llama.cpp, stable-diffusion.cpp
