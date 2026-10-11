# Backend Architecture & Code Rules (`backend/`)

1. **Strict Layered Architecture**:
   - `core` / `domain`: Pure entities, types, enums, and centralized `AppError`. No Axum or SQL dependencies.
   - `infrastructure`: SQLx database repositories, OS child process management, hardware detectors.
   - `services`: Application business logic (`ModelService`, `ChatService`, `RuntimeService`, `HostService`, `ProxyService`).
   - `api`: Axum routes and thin handlers (DTO parsing -> service call -> DTO response).
2. **Concurrency**:
   - Never use `std::sync::Mutex` in async Tokio tasks. Use `tokio::sync::Mutex` or `tokio::sync::RwLock`.
3. **Standards**:
   - Use `uuid::Uuid::new_v4()` for IDs.
   - Return `AppResult<Json<T>>` from handlers with automatic HTTP status code mapping.
   - OpenAI endpoints (`/v1/*`) must return `{ "error": { "message": "...", "type": "...", "code": "..." } }`.
4. **Testing**:
   - Write unit tests for services and repository tests using in-memory SQLite (`sqlite::memory:`).
