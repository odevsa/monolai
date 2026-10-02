<div align="center">
  <img src="frontend/src/lib/assets/icon-dark.svg" alt="Logo" width="64" height="64" />

**Monolai**

**A cross platform desktop application for managing and running large language models locally.**

[![Release](https://img.shields.io/github/v/release/odevsa/monolai?label=Release&style=flat-square&color=blue&logo=github)](https://github.com/odevsa/monolai/releases/latest)
![Rust](https://img.shields.io/badge/Rust-1.80%2B-orange.svg?logo=rust&style=flat-square)
![Svelte](https://img.shields.io/badge/Svelte-5-FF3E00.svg?logo=svelte&style=flat-square)
![SQLite](https://img.shields.io/badge/SQLite-FTS5-003B57.svg?logo=sqlite&style=flat-square)
![Docker](https://img.shields.io/badge/Docker-Ready-2496ED.svg?logo=docker&style=flat-square)

[Features](#features) •
[Quick Start](#quick-start) •
[Tech Stack](#tech-stack)

</div>

## Features

- **Real-time Dashboard**: Monitor CPU, RAM, and GPU usage in real-time.
- **Model Management**: Download, manage, and run LLMs locally.
- **OpenAPI & Swagger UI**: Interactive API documentation at `/swagger-ui/`.
- **Web UI**: Clean and modern web interface.
- **Tauri**: Desktop application with GPU acceleration.

## Quick Start

### Docker

```bash
docker run -d -p 3000:3000 --name monolai odevsa/monolai
```

## Tech Stack

- **Backend**: Rust (Axum +Tokio + GPU)
- **Frontend**: SvelteKit + TypeScript + Tailwind CSS
- **Desktop**: Tauri
- **Database**: SQLite
