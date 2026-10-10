# Desktop Application Rules (`desktop/`)

### 1. Strict Layered Architecture
All code in `desktop/` must follow the designated module layers:
- `core/`: Pure domain types (`ServerStatus`), message enums (`UiCommand`, `SupervisorEvent`, `TrayAction`), and config serialization (`GuiConfig`). Zero dependencies on `egui` or process calls.
- `platform/`: OS abstractions (`single_instance`, `autostart`, `process_ext`, `window`). Must be cleanly isolated with target `cfg` attributes.
- `supervisor/`: Asynchronous backend supervision (`binary_locator`, `health`, `process`, `worker`). Manages child process lifecycles on a dedicated worker thread.
- `tray/`: System tray management via `tray-icon`. Forwards `TrayAction` to the UI thread via channels.
- `ui/`: `eframe`/`egui` presentation layer (`theme`, `components`, `views`, `app`).

### 2. Zero I/O on GUI Rendering Loop
- Never perform synchronous network requests (`ureq`), child process execution (`Command::output`), or file blocking inside `eframe::App::update`.
- All operations (start, stop, restart, maintenance flags, and config updates) must be dispatched as `UiCommand` to `SupervisorHandle`.
- The GUI thread consumes results reactively via `SupervisorEvent`.

### 3. Cross-Platform Safety
- Guard OS-specific logic (`#[cfg(windows)]`, `#[cfg(target_os = "linux")]`, `#[cfg(target_os = "macos")]`).
- Windows backend child processes must hide console windows (`CREATE_NO_WINDOW`) and link to Job Objects (`KILL_ON_JOB_CLOSE`).
- Linux backend processes must set parent death signals (`PR_SET_PDEATHSIG` -> `SIGTERM`).

### 4. Lifecycle & Clean Shutdown
- Child processes are tracked exclusively by `SupervisorWorker`.
- Upon application close or tray quit, send `UiCommand::Shutdown` to guarantee clean termination without orphan processes or race conditions.
