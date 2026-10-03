<div align="center">
  <img src="frontend/src/lib/assets/icon-dark.svg" alt="Monolai Logo" width="64" height="64" />

**Monolai**

**A lightweight, local AI model manager and OpenAI-compatible runtime.**

[![Release](https://img.shields.io/github/v/release/odevsa/monolai?label=Release&style=flat-square&color=blue&logo=github)](https://github.com/odevsa/monolai/releases/latest)
![Rust](https://img.shields.io/badge/Rust-1.80%2B-orange.svg?logo=rust&style=flat-square)
![Svelte](https://img.shields.io/badge/Svelte-5-FF3E00.svg?logo=svelte&style=flat-square)
![SQLite](https://img.shields.io/badge/SQLite-003B57.svg?logo=sqlite&style=flat-square)
![Docker](https://img.shields.io/badge/Docker-Ready-2496ED.svg?logo=docker&style=flat-square)

[Features](#features) • [Quick Start](#quick-start) • [Configuration](#configuration) • [API Usage](#api-usage) • [Development](#development) • [Tech Stack](#tech-stack)

</div>

## Features

- **Offline & Private**: Run local models (GGUF) on your own hardware with zero telemetry.
- **Hardware Acceleration**: Automatic GPU detection and support for **CUDA**, **ROCm**, and **Vulkan**, with multi-threaded CPU fallback.
- **Automated Runtime Management**: Automatic download and installation of upstream engine binaries (`llama.cpp`, `stable-diffusion.cpp`).
- **Memory & Process Lifecycle**: Automatic model unloading from VRAM/RAM after inactivity (`idle_timeout_seconds`) and orphan process cleanup.
- **OpenAI-Compatible API**: Drop-in `/v1/chat/completions` and `/v1/models` endpoints compatible with standard SDKs and tools (OpenAI SDK, LangChain, Cursor, Continue).
- **Web UI**: SvelteKit 5 dashboard with real-time hardware telemetry (CPU, RAM, GPU) via SSE and persistent chat history.
- **GUI Interfaces**: Native Desktop App (`monolai-gui`) with system tray, background daemon supervisor.
- **OpenAPI Documentation**: Interactive Swagger UI at `/api/swagger`.

## Quick Start

### 1. Native Desktop App

```bash
cargo run --manifest-path desktop/Cargo.toml
# Or with Make:
make dev-desktop
```

### 2. Standalone Server (CLI / Web)

```bash
# Starts backend server (web UI accessible at http://localhost:8080)
cargo run --manifest-path backend/Cargo.toml -- --config ~/.config/monolai/config.yaml
# Or with Make:
make dev-backend
```

#### CLI Options

```text
Usage: monolai [OPTIONS] [MODELS_FOLDER]

Arguments:
  [MODELS_FOLDER]  Path to models directory [default: ~/models]

Options:
  -c, --config <PATH>           Path to config YAML file
  -l, --listen <ADDR>           Listen address (e.g. 0.0.0.0:8080)
      --db <PATH>               Custom path to SQLite database
      --runtimes <DIR>          Path to runtime manifests and binaries folder
  -h, --help                    Print help
  -V, --version                 Print version
```

### 3. Docker

```bash
docker run -d \
  -p 8080:8080 \
  -v ~/.local/share/monolai/models:/app/models \
  -v ~/.local/share/monolai/runtimes:/app/runtimes \
  -v ~/.local/share/monolai/data:/app/data \
  --name monolai \
  odevsa/monolai:latest
```

For NVIDIA GPU acceleration (CUDA):

```bash
docker run -d --gpus all \
  -p 8080:8080 \
  -v ~/.local/share/monolai/models:/app/models \
  -v ~/.local/share/monolai/runtimes:/app/runtimes \
  -v ~/.local/share/monolai/data:/app/data \
  --name monolai \
  odevsa/monolai:latest
```

Using Docker Compose:

```bash
docker compose up -d
```

## Configuration

Monolai uses `config.yaml` for storage paths and acceleration target preferences.

### Default File Path

| Platform    | Path                                                |
| :---------- | :-------------------------------------------------- |
| **Linux**   | `~/.config/monolai/config.yaml`                     |
| **macOS**   | `~/Library/Application Support/monolai/config.yaml` |
| **Windows** | `%APPDATA%\monolai\config.yaml`                     |

### Sample `config.yaml`

```yaml
# Storage directories (supports ~ tilde expansion)
models: ~/models
runtimes: ~/.local/share/monolai/runtimes

# Acceleration target: auto, cpu, cuda, rocm, vulkan, oneapi
hardware: auto
```

### Environment Variables

| Variable       | Description                  | Default       |
| :------------- | :--------------------------- | :------------ |
| `MODELS_DIR`   | Models storage directory     | `~/models`    |
| `RUNTIMES_DIR` | Runtime engines directory    | OS Data Dir   |
| `DATA_DIR`     | Database / data directory    | OS Config Dir |
| `HARDWARE`     | Acceleration target override | `auto`        |
| `HOST`         | Server bind address          | `0.0.0.0`     |
| `PORT`         | Server bind port             | `8080`        |

## API Usage

Monolai provides standard OpenAI-compatible endpoints at `/v1`.

### 1. List Models (`GET /v1/models`)

#### cURL

```bash
curl http://localhost:8080/v1/models
```

#### Python (OpenAI SDK)

You need to have the OpenAI SDK installed: `pip install openai`

```python
from openai import OpenAI

client = OpenAI(base_url="http://localhost:8080/v1", api_key="not-needed")

for model in client.models.list():
    print(model.id)
```

### 2. Chat Completions (`POST /v1/chat/completions`)

#### cURL

```bash
curl http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "your-model-name",
    "messages": [
      {"role": "user", "content": "Hello Monolai!"}
    ],
    "temperature": 0.7
  }'
```

#### Python (Streaming)

```python
from openai import OpenAI

client = OpenAI(
    base_url="http://localhost:8080/v1",
    api_key="not-needed"
)

response = client.chat.completions.create(
    model="your-model-name",
    messages=[{"role": "user", "content": "Hello Monolai!"}],
    stream=True
)

for chunk in response:
    print(chunk.choices[0].delta.content or "", end="", flush=True)
```

Interactive Swagger documentation is available at **http://localhost:8080/api/swagger**.

## Development

| Command                | Description                                                  |
| :--------------------- | :----------------------------------------------------------- |
| `make install`         | Install frontend dependencies and verify Rust backend        |
| `make dev`             | Start frontend and backend concurrently in dev mode          |
| `make dev-host`        | Start frontend (`--host`) and backend concurrently           |
| `make dev-desktop`     | Start native Rust desktop GUI app (`monolai-gui`)            |
| `make dev-backend`     | Run backend server only                                      |
| `make dev-frontend`    | Run frontend development server only                         |
| `make build`           | Build release binaries for both server and desktop           |
| `make build-server`    | Compile frontend and build release server binary (`monolai`) |
| `make build-desktop`   | Build release desktop binary (`monolai-gui`)                 |
| `make clean`           | Remove build artifacts                                       |
| `make version [x.y.z]` | Synchronize project version across manifests                 |

## Tech Stack

- **Backend**: [Rust](https://www.rust-lang.org/)
- **Desktop**: [Rust](https://www.rust-lang.org/) + [egui](https://github.com/emilk/egui)
- **Frontend**: [vite](https://vitejs.dev/) + [SvelteKit 5](https://svelte.dev/) + [Tailwind CSS v4](https://tailwindcss.com/)
- **Database**: [SQLite](https://www.sqlite.org/) (SQLx migrations)
- **Inference Engines**: Available as plugin
