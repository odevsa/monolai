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
4. **Ever use SSoT principle**:
   - **Single Source of Truth**: Every piece of information should have a single, canonical source.
   - **Avoid Duplication**: Don't store the same information in multiple places.
   - **Traceability**: You should be able to trace any piece of information back to its source.

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
   - `src/lib/components/ui/`: Atomic Mini-Design System (`atoms`, `molecules`, `organisms`) standardizing buttons, inputs, selects, cards, headings, modals, and alerts.
   - `src/routes/`: SvelteKit page routes responsible only for layout and view composition.
4. **OpenAI Official SDK Integration**:
   - All OpenAI-compatible chat completions (`/v1/chat/completions`) and image generation (`/v1/images/generations`) must be dispatched through the official `openai` JS/TS client configured in `src/lib/api/openai.ts`.
5. **Internationalization (i18n)**:
   - Mandatory use of `t('key.path')` via `src/lib/i18n`. Hardcoded UI strings in components are forbidden.
   - **Semantic Optimization & DRY Keys Policy**: Always prioritize reusing generic action verbs and global controls from `common.*` (`common.save`, `common.cancel`, `common.delete`, `common.new`, `common.add`, `common.edit`, `common.close`, `common.copy`, `common.download`, `common.clear`, `common.refresh`, `common.unload`, `common.scrollToBottom`, etc.) instead of creating redundant domain-specific variants (e.g. do NOT create `models.saveModel`, `settings.saveGeneral`, `image.deleteImage`, `chat.deleteChat`). Reserve domain namespaces (`chat.*`, `models.*`, `image.*`, `settings.*`) strictly for domain concepts, unique descriptions, contextual confirmation modals (e.g., `chat.deleteChatConfirmMsg`), and specialized error feedback.
   - **Feature Development Workflow**: When implementing new features or components, agents MUST only introduce the `t('context.key')` calls in the views and add the canonical English definition to `src/lib/i18n/locales/en.json`. Do NOT populate or translate all other locale `.json` files (`pt-BR.json`, `de.json`, `fr.json`, etc.) during feature development, as bulk translations are performed on demand. Run `just i18n-check` to verify that no `t()` call is missing from `en.json` and no dead keys are left behind.
6. **Styling, Theme Tokens & Zero Hardcoded Colors Policy**:
   - **Centralized Theme Tokens**: All theme colors, semantic status tokens, and syntax highlighting variables MUST reside exclusively in `src/themes.css` (imported by `src/app.css`) under `@theme` and `[data-theme='...']`. Components must NEVER define disconnected theme blocks, local theme overrides, or arbitrary color values.
   - **Mandatory Semantic CSS Variables**: Always use semantic theme tokens:
     - Core surfaces & text: `var(--bg-primary)`, `var(--bg-surface)`, `var(--bg-surface-hover)`, `var(--border-color)`, `var(--border-hover)`, `var(--text-primary)`, `var(--text-secondary)`, `var(--text-muted)`.
     - Brand accent: `var(--primary)`, `var(--primary-hover)`, `var(--primary-light)`, `var(--primary-focus)`.
     - Status & feedback: `var(--success)`, `var(--warning)`, `var(--danger)`, `var(--info)` along with their semantic sub-variants (`-light`, `-border`, `-text`, `-hover`, `-focus`).
   - **Zero Hardcoded Colors**: Absolutely NO inline hardcoded HEX values (`#...`), raw `rgb()`/`rgba()`, or arbitrary Tailwind utility colors (e.g., `text-red-500`, `bg-emerald-500`, `text-amber-400`, `text-white`, `bg-black`). Use the corresponding theme token (`text-[var(--danger-text)]`, `bg-[var(--success-light)]`, etc.) to guarantee seamless compatibility across all 11 themes.
   - **Layout Shell Inviolability**: The layout shell (sidebar, floating header tabs/actions, floating glassmorphic chat input) is structurally fixed and must not be broken or stripped during feature development.
7. **Componentization & Subcomponentization (DRY Principle)**:
   - Avoid monolithic components that lump together views, event listeners, and business logic for multiple unrelated features or routes into a single file.
   - Extract route-specific or contextual content (e.g., chat history, image galleries, system metrics in sidebars) into isolated components that only instantiate and run effects when their corresponding view is active.
   - Deconstruct repetitive markup patterns (e.g., hardware metric progress bars, status items, list rows) into reusable subcomponents (`atoms`/`molecules`) rather than duplicating raw HTML and Tailwind utility chains.
   - Maintain high cohesion and clear encapsulation: components must accept strongly-typed `$props()` and manage only state pertinent to their isolated scope.
   - Zero local `<style>` blocks in components: rely exclusively on Tailwind CSS utility classes and semantic theme variables.
8. **Mandatory Global UI Design System Usage (`src/lib/components/ui/`)**:
   - Always favor canonical UI components from `src/lib/components/ui/` (`Button`, `Badge`, `Input`, `Select`, `Card`, `Modal`, `Alert`, `EmptyState`) instead of writing ad-hoc raw HTML controls (`<button>`, `<input>`, `<select>`) with manual utility chains.
   - If a reusable UI pattern or control variant is needed across multiple views (e.g. specialized pills, action buttons, filter inputs) and does not yet exist, create it as a global atom/molecule in `src/lib/components/ui/` rather than duplicating local ad-hoc markup in page views.

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
