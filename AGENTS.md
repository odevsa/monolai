# Monolai Development Guidelines & AI Agent Rules

This document outlines the mandatory architecture, engineering standards, and best practices for any AI agent or contributor working on the **Monolai** monorepo.

All code, comments, documentation, and commit messages MUST be written in **English**.

---

## 1. Monorepo Overview & Project Structure

The Monolai repository is structured as a full-stack local AI platform comprising three core applications:

```
monolai/
├── backend/    # Rust (Axum, Tokio, SQLx SQLite, Process Supervisor & LLM Proxy)
├── frontend/   # Svelte 5 (Runes), TypeScript, SvelteKit, Tailwind CSS v4, Vite
├── desktop/    # Rust GUI (eframe/egui 0.30, Tray-Icon, Native Process Supervisor)
├── .agents/    # AI Agent workspace customizations and modular rule sets
└── AGENTS.md   # Canonical repository guidelines and architecture standards
```

---

## 2. General Engineering Standards

1. **Language & Documentation**:
   - Everything in the codebase must be in **English**: variable names, comments, rustdoc/JSDoc, error messages, and git commits.
2. **Predictable Code Evolution**:
   - Do not add arbitrary unneeded dependencies.
   - Preserve existing explanatory comments and public API contracts.
   - Ensure clean compilation and test passes (`cargo check`, `cargo test`, `npm run check`, `npm run lint`) before finishing tasks.
3. **No Hardcoded Secrets or Paths**:
   - Never hardcode absolute user paths (e.g., `/home/...` or `C:\Users\...`). Always use path resolvers or environment variables.

---

## 3. Backend Guidelines (`backend/`)

### 3.1 Strict Layered Architecture
The backend is structured into four primary layers:
1. **`core` & `domain`**:
   - Pure domain models, value objects, enums (`ModelState`, `GpuInfo`, `ChatRecord`), and centralized error handling (`AppError`).
   - Must NOT depend on HTTP frameworks (Axum), SQL queries, or system process calls.
2. **`infrastructure`**:
   - `db`: Typed repositories (`ModelRepository`, `ChatRepository`, `SettingRepository`) over SQLite with embedded migrations.
   - `process`: OS child process spawning, port discovery, process caching, and signal handling (`SIGTERM` / `SIGKILL`).
   - `hardware`: GPU detection (NVIDIA, ROCm, Metal, Vulkan) and host metrics (`sysinfo`).
   - `downloader`: Asset downloads, archive extraction (`zip`, `tar.gz`), and SSE installation progress broadcasting.
3. **`services`**:
   - Application use cases and business orchestration (`ModelService`, `ChatService`, `RuntimeService`, `HostService`, `ProxyService`).
   - Decoupled from HTTP types (`StatusCode`, `Path`, `Query`).
4. **`api`**:
   - Axum presentation layer. Handlers must be thin: parse request DTO -> delegate to service -> return response DTO.
   - Centralized error response via `AppError::into_response`.
   - OpenAPI specifications (`utoipa`) kept clean with DTO schemas.

### 3.2 Backend Concurrency & Code Standards
- **Never block Tokio runtime worker threads**: Never use `std::sync::Mutex` across `.await` points or for long calculations. Use `tokio::sync::Mutex` or `tokio::sync::RwLock`.
- **Standard UUIDs**: Always use the official `uuid` crate (`uuid::Uuid::new_v4()`). Homegrown pseudo-UUIDs are strictly forbidden.
- **Resource Cleanliness**: Child processes must be tracked and cleaned up on shutdown or error. Background monitoring loops must use cached state to avoid tight database polling.

---

## 4. Frontend Guidelines (`frontend/`)

### 4.1 Modern Svelte 5 & TypeScript Architecture
1. **Svelte 5 Runes**:
   - Always use Svelte 5 Runes: `$state`, `$derived`, `$props`, `$effect`.
   - Do NOT use legacy Svelte 3/4 reactivity (`let` assignments for reactive state, `$:` statements).
   - Reactive singleton state belongs in `src/lib/state/*.svelte.ts`.
2. **TypeScript Strictness**:
   - Zero `any` policy. All API responses, state objects, and component props must have explicit TypeScript interfaces matching backend DTOs in `src/lib/types/`.
3. **Layered Frontend Structure**:
   - `src/lib/api/`: Centralized HTTP, SSE, and official SDK methods (`modelsApi`, `runtimesApi`, `chatsApi`, `hostApi`, `configApi`, `settingsApi`). Components must NOT execute ad-hoc `fetch()` calls or parse raw SSE streams directly.
   - `src/lib/state/`: Global application state (active model, chat tabs, confirm modal, host metrics) using reactive runes singletons.
   - `src/lib/components/ds/`: Atomic Mini-Design System (`atoms`, `molecules`, `organisms`) standardizing buttons, inputs, selects, cards, headings, modals, and alerts.
   - `src/routes/`: SvelteKit page routes responsible only for layout and view composition.
4. **OpenAI Official SDK Integration**:
   - All OpenAI-compatible chat completions (`/v1/chat/completions`) and image generation (`/v1/images/generations`) must be dispatched through the official `openai` JS/TS client configured in `src/lib/api/openai.ts`.
5. **Internationalization (i18n)**:
   - Mandatory use of `t('key.path')` via `src/lib/i18n`. Hardcoded UI strings in components are forbidden. All user-facing strings are stored in `src/lib/i18n/locales/en.json`.
6. **Styling & Layout Inviolability**:
   - Use Tailwind CSS v4 design tokens and CSS variables (`var(--bg-primary)`, `var(--text-primary)`, `var(--primary)`, etc.).
   - Do not use inline hardcoded HEX color values or blanket `text-white` on text elements; maintain theme consistency across all color schemes.
   - The layout shell (sidebar, floating header tabs/actions, floating glassmorphic chat input) is structurally fixed and must not be broken or stripped during feature development.

---

## 5. Desktop Guidelines (`desktop/`)

### 5.1 Strict Layered Desktop Architecture
The desktop application (`monolai-gui`) is organized into decoupled layers communicating via thread-safe channels:
1. **`core`**:
   - Pure domain models (`ServerStatus`), configuration serialization (`GuiConfig`, `config.yaml` synchronization), and thread-safe communication message types (`UiCommand`, `SupervisorEvent`, `TrayAction`).
   - Must NOT depend on GUI types (`egui`), HTTP network calls, or child process execution.
2. **`platform`**:
   - Cross-platform operating system integrations:
     - `single_instance`: Named Mutex (Windows) and runtime `flock` (Unix) single-instance enforcement.
     - `autostart`: System boot initialization via `auto-launch`.
     - `process_ext`: Process isolation guards (Windows Job Objects with `KILL_ON_JOB_CLOSE`, Linux `PR_SET_PDEATHSIG`, Windows `CREATE_NO_WINDOW`).
     - `window`: Cross-platform minimization and restoration across Wayland, X11, Windows, and macOS.
3. **`supervisor`**:
   - Asynchronous backend management.
   - `binary_locator`: Locates installed and dev backend binaries.
   - `health`: Port conflict verification, HTTP health check probes, and API notifications.
   - `process`: Spawn, log redirection, and graceful termination (`SIGTERM` / `SIGKILL`).
   - `worker`: Dedicated background worker thread processing `UiCommand`s and emitting `SupervisorEvent`s with `request_repaint()` triggers.
4. **`tray`**:
   - System tray management via `tray-icon`. Dispatches `TrayAction`s over channels to the UI loop without blocking or triggering uncoordinated process exits.
5. **`ui`**:
   - Pure visual presentation layer using `eframe`/`egui`.
   - `components`: Atomic GPU vector-rendered components (`draw_status_dot`, `singleline_input`, texture loaders).
   - `views`: Modular screens (`main_view`, `settings_view`) that send `UiCommand`s and render state without performing synchronous I/O.
   - `app`: `DesktopApp` implementing `eframe::App`, consuming `SupervisorEvent`s via `poll_event()`.

### 5.2 Concurrency & Thread Inviolability
1. **Zero I/O on GUI Thread**:
   - Never execute synchronous network requests (`ureq`), heavy process commands (`Command::output`), or file locks within the `eframe::App::update` rendering loop.
   - All server management actions, maintenance routines (`--clean-storage`, `--factory-reset`), and HTTP notifications must be delegated to the `SupervisorWorker` via `UiCommand`.
2. **Clean Lifecycle & Coordinated Shutdown**:
   - `SupervisorHandle` coordinates process termination upon application exit (`UiCommand::Shutdown`). No competing shutdown handlers or orphan child processes.

---

## 6. Testing & Quality Verification

- **Backend**: Every new service or business rule must include unit tests. Repositories must be tested against an in-memory SQLite database (`:memory:`). Run `cargo test` and `cargo check`.
- **Frontend**: Verify TypeScript validity and formatting (`npm run check`, `npm run lint`).
- **Desktop**: Ensure cross-platform compilation succeeds (`cargo check`).
