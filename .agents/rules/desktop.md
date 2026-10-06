# Desktop Application Rules (`desktop/`)

1. **Process Supervision**:
   - The desktop application supervises the backend process. Spawning, port discovery, health checking, and shutdown must be resilient.
2. **GUI Loop Decoupling**:
   - Never perform synchronous I/O or HTTP calls inside the `eframe::App::update` loop.
   - Use background worker threads or async tasks communicating with the GUI thread via thread-safe channels (`crossbeam-channel` or `mpsc`).
3. **Cross-Platform Safety**:
   - Properly guard OS-specific logic with `#[cfg(windows)]`, `#[cfg(target_os = "linux")]`, and `#[cfg(target_os = "macos")]`.
   - Windows console windows must be hidden (`CREATE_NO_WINDOW`) for background backend child processes.
4. **Lifecycle & Clean Shutdown**:
   - Ensure the spawned backend child process is gracefully terminated when the desktop app exits.
