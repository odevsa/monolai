import Prism from 'prismjs';

// Prism components (prism-*.js) are legacy scripts that expect a global `Prism` variable.
// In Vite/Rollup production builds, Prism is bundled into a module scope and not exposed
// to globalThis/window by default. Assigning it here ensures that subsequent grammar imports
// can access Prism on the global scope without ReferenceError.
if (typeof window !== 'undefined') {
	(window as unknown as { Prism: typeof Prism }).Prism = Prism;
}
if (typeof globalThis !== 'undefined') {
	(globalThis as unknown as { Prism: typeof Prism }).Prism = Prism;
}

export default Prism;
