# Frontend Architecture & Code Rules (`frontend/`)

1. **Modern Svelte 5**:
   - Use Svelte 5 Runes: `$state`, `$derived`, `$props`, `$effect`.
   - Never use legacy Svelte 3/4 reactivity (`let` for reactive variables, `$:` statements).
2. **TypeScript Strictness**:
   - Strict typing with zero `any`.
   - Keep API types and DTOs synchronized with backend schemas.
3. **Layered Client Architecture**:
   - `src/lib/api/` or `src/lib/services/`: Centralized API clients. Components must never call raw `fetch()` directly.
   - `src/lib/stores/` or `src/lib/state/`: Centralized reactive state using runes or stores.
   - `src/lib/components/`: Reusable, atomic components decoupled from router pages.
4. **Styling & UI**:
   - Use Tailwind CSS v4 design tokens and CSS variables.
   - Ensure responsive, polished design for both web browsers and embedded webviews.
