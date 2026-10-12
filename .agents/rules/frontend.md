# Frontend Architecture & Code Rules (`frontend/`)

1. **Modern Svelte 5**:
   - Use Svelte 5 Runes: `$state`, `$derived`, `$props`, `$effect`.
   - Never use legacy Svelte 3/4 reactivity (`let` for reactive variables, `$:` statements).
   - Singleton global state lives in `src/lib/state/*.svelte.ts`.
2. **TypeScript Strictness**:
   - Strict typing with zero `any`.
   - Centralized interfaces in `src/lib/types/` matching backend DTOs.
3. **Layered Client Architecture**:
   - `src/lib/api/`: Centralized typed API clients (`modelsApi`, `runtimesApi`, `chatsApi`, `hostApi`, `configApi`, `settingsApi`).
   - Components must never call raw `fetch()` or parse manual SSE chunks directly.
4. **OpenAI SDK Integration**:
   - All OpenAI-compatible chat completions (`/v1/chat/completions`) and image generation (`/v1/images/generations`) must use the official `openai` JS/TS client via `src/lib/api/openai.ts`.
5. **Internationalization (i18n)**:
   - All user-facing UI text must use the reactive helper `t('key.path')` from `src/lib/i18n`.
   - Hardcoded UI text strings are strictly forbidden. All entries belong in `src/lib/i18n/locales/en.json`.
   - **DRY & Semantic Optimization Policy**: Always reuse generic actions and controls from `common.*` (`common.save`, `common.cancel`, `common.delete`, `common.new`, `common.add`, `common.edit`, `common.close`, `common.copy`, `common.clear`, `common.refresh`, `common.download`, `common.unload`, etc.) for buttons and tooltips instead of creating redundant domain-specific variations (e.g. avoid `models.saveModel`, `image.deleteImage`). Reserve domain namespaces exclusively for domain-specific descriptions, dialog messages, and unique labels. Run `just i18n-check` to verify zero missing and zero unused keys.
6. **Styling, Theme Tokens & Zero Hardcoded Colors Policy**:
   - Centralized Theme Tokens: All theme colors, semantic status tokens, and syntax highlighting variables MUST reside exclusively in `src/themes.css` (imported by `src/app.css`) under `@theme` and `[data-theme='...']`. Components must NEVER define local theme overrides or arbitrary color values.
   - Mandatory Semantic CSS Variables: Always use theme variables (`var(--bg-primary)`, `var(--bg-surface)`, `var(--border-color)`, `var(--text-primary)`, `var(--primary)`, `var(--success)`, `var(--warning)`, `var(--danger)`, `var(--info)`, etc.).
   - Zero Hardcoded Colors: Absolutely NO inline hardcoded HEX values (`#...`), raw `rgb()`/`rgba()`, or arbitrary Tailwind utility colors (e.g. `text-red-500`, `bg-emerald-500`, `text-white`, `bg-black`). Use the corresponding theme token (`text-[var(--danger-text)]`, `bg-[var(--success-light)]`, etc.) to guarantee seamless compatibility across all 11 themes.
   - Layout Shell Inviolability: The layout shell (sidebar navigation, header action bar, floating glassmorphic chat input) is structurally canonical and must remain intact.
7. **Componentization & Subcomponentization (DRY Principle)**:
   - Avoid monolithic components that lump together views, event listeners, and business logic into a single file. Break down large pages into modular components.
   - Deconstruct repetitive markup patterns (iterated items in lists, metric bars, repeated action rows, status indicators) into reusable subcomponents rather than duplicating raw HTML and Tailwind utility chains.
   - Maintain high cohesion and clear encapsulation: components must accept strongly-typed `$props()` and manage only state pertinent to their isolated scope.
   - Zero local `<style>` blocks in components: use 100% Tailwind CSS classes with semantic theme variables.
8. **Mandatory Global UI Design System Usage (`src/lib/components/ui/`)**:
   - Always favor canonical UI components from `src/lib/components/ui/` (`Button`, `Badge`, `Input`, `Select`, `Card`, `Modal`, `Alert`, `EmptyState`) instead of writing ad-hoc raw HTML controls (`<button>`, `<input>`, `<select>`) with manual utility chains.
   - If a reusable UI pattern or control variant is needed across multiple views (e.g. specialized pills, action buttons, filter inputs) and does not yet exist, create it as a global atom/molecule in `src/lib/components/ui/` rather than duplicating local ad-hoc markup in page views.
