# Frontend Architecture & Code Rules (`frontend/`)

1. **Modern Svelte 5**:
   - Use Svelte 5 Runes: `$state`, `$derived`, `$props`, `$effect`.
   - Never use legacy Svelte 3/4 reactivity (`let` for reactive variables, `$:` statements).
   - Singleton global state lives in `src/lib/state/*.svelte.ts`.
2. **TypeScript Strictness**:
   - Strict typing with zero `any`.
   - Keep centralized types in `src/lib/types/` synchronized with backend DTOs.
3. **Layered Client Architecture**:
   - `src/lib/api/`: Centralized typed API clients (`modelsApi`, `runtimesApi`, `chatsApi`, `hostApi`, `configApi`, `settingsApi`).
   - Components must never call raw `fetch()` or parse manual SSE chunks directly.
4. **OpenAI SDK Integration**:
   - All OpenAI-compatible chat completions (`/v1/chat/completions`) and image generation (`/v1/images/generations`) must use the official `openai` JS/TS client via `src/lib/api/openai.ts`.
5. **Internationalization (i18n)**:
   - All user-facing UI text must use the reactive helper `t('key.path')` from `src/lib/i18n`.
   - Hardcoded UI text strings are strictly forbidden. All entries belong in `src/lib/i18n/locales/en.json`.
6. **Design System & Layout Inviolability**:
   - Reusable UI elements must use the Atomic Design System in `src/lib/components/ds/` (`atoms`, `molecules`, `organisms`).
   - Use Tailwind CSS v4 design tokens and CSS variables (`var(--bg-primary)`, `var(--text-primary)`, etc.). Never hardcode inline HEX colors or generic `text-white` on themed elements.
   - The layout shell (sidebar navigation, header action bar, floating glassmorphic chat input) is structurally canonical and must remain intact.
