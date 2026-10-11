# Monolai General Development Rules

These rules apply to all projects in the Monolai monorepo (`backend`, `frontend`, `desktop`):

1. **Language & Documentation**:
   - All code, comments, documentation, and commit messages MUST be written in **English**.
2. **Project Structure**:
   - `backend/`: Rust Axum REST API and LLM proxy.
   - `frontend/`: Svelte 5 + SvelteKit web application.
   - `desktop/`: Rust eframe/egui desktop application with system tray.
3. **Paths and Portability**:
   - Do NOT hardcode user-specific paths or system directories. Always use relative paths or environment configurations.
4. **Clean Code**:
   - Maintain documentation integrity and preserve explanatory comments.
   - Run compilation and tests before completing tasks.
