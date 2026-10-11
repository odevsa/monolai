<div align="center">
  <img src="frontend/src/lib/assets/icon-dark.svg" alt="Monolai Logo" width="64" height="64" />

**Monolai**

**A lightweight, local AI model manager and OpenAI-compatible runtime.**

[![Release](https://img.shields.io/github/v/release/odevsa/monolai?label=Release&style=flat-square&color=blue&logo=github)](https://github.com/odevsa/monolai/releases/latest)
![Rust](https://img.shields.io/badge/Rust-1.80%2B-orange.svg?logo=rust&style=flat-square)
![Svelte](https://img.shields.io/badge/Svelte-5-FF3E00.svg?logo=svelte&style=flat-square)
![SQLite](https://img.shields.io/badge/SQLite-003B57.svg?logo=sqlite&style=flat-square)
![Docker](https://img.shields.io/badge/Docker-Ready-2496ED.svg?logo=docker&style=flat-square)

[Features](#features) •
[Themes](#themes) •
[Installation](#installation) •
[Getting Started](#getting-started) •
[CLI](#cli) •
[Docker](#docker) •
[Configuration](#configuration) •
[API Usage](#api-usage) •
[Development](#development) •
[Tech Stack](#tech-stack)

</div>

## Overview

<div align="center">
  <img src="assets/screenshot-1.png" alt="Chat" width="49%" />
  <img src="assets/screenshot-2.png" alt="System Info" width="49%" />
  <br />
  <img src="assets/screenshot-3.png" alt="Runtimes" width="49%" />
  <img src="assets/screenshot-4.png" alt="Models" width="49%" />
</div>

**Monolai** is a **local AI model manager** and **OpenAI-compatible runtime**. It is a **lightweight**, **offline** and **private** tool that allows you to **run local models** (GGUF) on your own hardware with **zero telemetry**.

## Features

- **Offline & Private**: Run local models (GGUF / diffusion checkpoints) on your own hardware with zero telemetry.
- **Hardware Acceleration**: Automatic GPU detection and support for **CUDA**, **ROCm**, and **Vulkan**, with multi-threaded CPU fallback.
- **Automated Runtime Management**: Automatic download, version tracking, and installation of upstream engine binaries (`llama.cpp`, `stable-diffusion.cpp`).
- **Image Generation**: Local text-to-image generation powered by `stable-diffusion.cpp` with support for custom resolutions, negative prompts, CFG scale, sampling steps, and seeds.
- **Reasoning & Thinking Process**: Real-time streaming support for reasoning models (e.g., DeepSeek-R1) with dedicated expandable `<think>` UI blocks in the chat view.
- **Memory & Process Lifecycle**: Automatic model unloading from VRAM/RAM after inactivity (`idle_timeout_seconds`) and orphan process cleanup.
- **OpenAI-Compatible API**: Drop-in `/v1/chat/completions`, `/v1/images/generations`, `/v1/embeddings`, and `/v1/models` endpoints compatible with standard SDKs and tools (OpenAI SDK, LangChain, Cursor, Continue).
- **Web UI**: Modern SvelteKit 5 dashboard with real-time hardware telemetry (CPU, RAM, GPU/VRAM, disk storage) via SSE, persistent chat history, and image gallery.
- **Log Management**: Built-in backend service logging and live frontend log viewer with service filtering, streaming, export, and diagnostics.
- **Internationalization (i18n)**: Out-of-the-box multi-language support (English and Spanish, more languages to come).
- **Native Desktop GUI**: Cross-platform desktop app (`monolai-gui`) built with egui 0.30, single-instance enforcement, system tray, and background daemon supervisor.
- **OpenAPI Documentation**: Interactive Swagger UI at `/api/swagger`.
- **Multi-platform**: Linux, macOS, Windows, and Docker.

## Themes

Monolai features dynamic theme support tailored for different AI model and UI preferences:

<div align="center">
  <img src="assets/theme-1.png" alt="Theme Preview 1" width="49%" />
  <img src="assets/theme-2.png" alt="Theme Preview 2" width="49%" />
  <br /><br />
  <img src="assets/theme-3.png" alt="Theme Preview 3" width="49%" />
  <img src="assets/theme-4.png" alt="Theme Preview 4" width="49%" />
</div>

## Installation

You can download pre-compiled release packages from the [Latest GitHub Releases](https://github.com/odevsa/monolai/releases/latest).

### Platform Packages

| Platform    | Packages / Formats             | Details                                                   |
| :---------- | :----------------------------- | :-------------------------------------------------------- |
| **Linux**   | `.deb`, `.AppImage`, `.tar.xz` | Pre-built binaries for `x86_64` and `aarch64`             |
| **Windows** | `.exe`                         | Pre-built Installer for `x86_64` and `aarch64`            |
| **macOS**   | `.dmg`                         | Apple Silicon (`arm64`) and Intel (`x86_64`) bundles      |
| **Docker**  | `odevsa/monolai:latest`        | Multi-arch container image with CUDA acceleration support |

> [!TIP]
> Each desktop release archive contains both the standalone server daemon (`monolai`) and the native desktop application with system tray management (`monolai-gui`).

---

## Getting Started

Follow these steps to set up Monolai and start running models in minutes:

### 1. Launch Monolai & Complete Setup (`/setup`)

Start Monolai using the desktop app or the standalone server:

```bash
# Start backend server (or run monolai-gui)
monolai
```

Open your browser at **http://localhost:8080/setup** (or open the desktop app).
On first launch, the configuration wizard guides you through:

- **Models Folder**: The local path where your model weights are saved (default: `~/models`).
- **Runtimes Folder**: Where engine binaries will be installed (default: `~/.local/share/monolai/runtimes`).
- **Hardware Acceleration**: Select your acceleration target (`auto`, `cuda`, `rocm`, `vulkan`, or `cpu`).

Once saved, Monolai automatically writes `config.yaml` and is ready to load inference engines.

### 2. Install Inference Runtimes (`/runtimes`)

Monolai uses modular inference engines that can be installed directly from the web UI:

1. Navigate to the **Runtimes** page (`/runtimes`).
2. Install the runtimes matching your use cases:
   - **`llama.cpp`**: Required for text generation, chat completion, and reasoning LLMs (`.gguf`).
   - **`stable-diffusion.cpp`**: Required for local text-to-image diffusion generation.
3. Select your preferred acceleration backend (e.g. CUDA, Vulkan, or CPU) and click **Install**. Download and installation progress is streamed in real time.

### 3. Download Models

Download your favorite `.gguf` models and place them directly into your configured models folder (e.g. `~/models`).

#### Recommended Places to Download Models:

- **[Hugging Face](https://huggingface.co/models?pipeline_tag=text-generation&sort=trending)** (filter by the `gguf` tag):
  - **[bartowski](https://huggingface.co/bartowski)**: Fast, high-quality quantizations of trending models (Llama 3, Qwen 2.5, DeepSeek-R1, Mistral).
  - **[TheBloke](https://huggingface.co/TheBloke)**: Extensive archive of classical LLM models in GGUF format.
  - Search for models like `Qwen2.5-Coder-7B-Instruct-GGUF`, `Llama-3.2-3B-Instruct-GGUF`, or `DeepSeek-R1-Distill-Qwen-7B-GGUF`.
- **[Civitai](https://civitai.com/)**: Source for diffusion checkpoints (`stable-diffusion.cpp` compatible models like SD 1.5, SDXL, and FLUX).

> [!TIP]
> **Quantization Recommendation**: For the best trade-off between memory and speed, look for **`Q4_K_M`** or **`Q5_K_M`** quantizations.

### 4. Register & Manage Models (`/models`)

1. Go to the **Models** page (`/models`).
2. Click **Scan Directory** to automatically discover `.gguf` files in your models folder, or click **Add Model** to configure a specific file manually.
3. Set model parameters such as:
   - **Model Alias / ID** (e.g., `qwen2.5-7b` or `deepseek-r1-7b`)
   - **Context Size** (e.g., `4096`, `8192`, `32768`)
   - **GPU Offload Layers** (e.g., offload all layers to VRAM for maximum speed)
   - **Inference Runtime** (`llama-cpp` or `sd-cpp`)

### 5. Start Using Monolai

- **Chat Interface (`/chat`)**: Chat with your local models, featuring full markdown rendering and live streaming of thinking processes (`<think>` tags) for reasoning models.
- **Image Generation (`/image`)**: Create images locally with prompt tuning, negative prompts, aspect ratio presets, seeds, and generation history.
- **OpenAI-Compatible Tools**: Use Monolai as a drop-in local replacement for OpenAI in **Cursor**, **Continue**, **Open WebUI**, **LangChain**, or any standard OpenAI SDK client by setting the base URL to `http://localhost:8080/v1`.
- **Telemetry & Diagnostics**: Check real-time GPU/VRAM, CPU, and RAM consumption under **System Info** (`/sysinfo`) and monitor live activity in **Logs** (`/logs`).

## CLI

```text
Usage: monolai [OPTIONS] [MODELS_FOLDER]

Arguments:
  [MODELS_FOLDER]  Path to models folder [default: ~/models]

Options:
  -c, --config <CONFIG>             Path to config YAML file (-c or --config)
  -l, --listen <LISTEN>             Listen address (e.g. 0.0.0.0:8080)
      --host <HOST>                 Host address to bind
  -p, --port <PORT>                 Port to bind
      --db <DB>                     Path to SQLite database
      --runtimes <RUNTIMES_FOLDER>  Path to extra runtime manifests folder
      --clean-storage               Clean storage: delete SQLite database and active process cache
      --factory-reset               Factory reset: delete database, process cache, and configuration
  -h, --help                        Print help
  -V, --version                     Print version
```

## Docker

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

# Network configuration
host: 0.0.0.0
port: 8080
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

### 3. Image Generation (`POST /v1/images/generations`)

#### cURL

```bash
curl http://localhost:8080/v1/images/generations \
  -H "Content-Type: application/json" \
  -d '{
    "model": "your-sd-model",
    "prompt": "A serene mountain landscape at sunset, cinematic lighting, 8k",
    "n": 1,
    "size": "512x512"
  }'
```

#### Python (OpenAI SDK)

```python
from openai import OpenAI

client = OpenAI(base_url="http://localhost:8080/v1", api_key="not-needed")

response = client.images.generate(
    model="your-sd-model",
    prompt="A serene mountain landscape at sunset, cinematic lighting, 8k",
    size="512x512",
    n=1
)

print(response.data[0].url)
```

Interactive Swagger documentation is available at **http://localhost:8080/api/swagger**.

## Development

Tasks are managed using [`just`](https://github.com/casey/just) (a modern task runner written in Rust). A backward-compatible `Makefile` wrapper is also provided.

| Command                | Description                                                    |
| :--------------------- | :------------------------------------------------------------- |
| `just install`         | Install frontend dependencies and verify Rust workspaces       |
| `just dev`             | Start frontend and backend concurrently in dev mode            |
| `just dev-host`        | Start frontend (`--host`) and backend concurrently             |
| `just dev-desktop`     | Start native Rust desktop GUI app (`monolai-gui`)              |
| `just dev-backend`     | Run backend server only                                        |
| `just dev-frontend`    | Run frontend development server only                           |
| `just check`           | Run type checks (SvelteKit) and compilation checks for Rust    |
| `just test`            | Run backend unit and integration tests                         |
| `just lint`            | Validate code formatting and style rules                       |
| `just format`          | Auto-format code across the monorepo (Prettier and rustfmt)    |
| `just build`           | Build default release bundle for host platform (`build-linux`) |
| `just build-backend`   | Compile frontend and build release server binary (`monolai`)   |
| `just build-desktop`   | Build release desktop binary (`monolai-gui`)                   |
| `just build-linux`     | Build complete Linux release bundle (server + desktop GUI)     |
| `just build-windows`   | Cross-compile Windows release binaries into `dist/windows/`    |
| `just build-macos`     | Build macOS release binaries and `.app` bundle                 |
| `just build-all`       | Build all supported multi-platform release binaries            |
| `just docker-build`    | Build local Docker image (`monolai:latest`)                    |
| `just docker-up`       | Start containerized services with Docker Compose               |
| `just docker-down`     | Stop containerized services with Docker Compose                |
| `just icons`           | Generate all multi-resolution icons (PNG, ICO, ICNS)           |
| `just clean`           | Remove build artifacts and temporary files                     |
| `just version [x.y.z]` | Synchronize project version across manifests                   |

## Tech Stack

- **Backend**: [Rust](https://www.rust-lang.org/)
- **Desktop**: [Rust](https://www.rust-lang.org/) + [egui](https://github.com/emilk/egui)
- **Frontend**: [Vite](https://vitejs.dev/) + [SvelteKit 5](https://svelte.dev/) + [Tailwind CSS v4](https://tailwindcss.com/)
- **Database**: [SQLite](https://www.sqlite.org/) with embedded SQLx migrations
- **Inference Engines**: Available for download as plugin
