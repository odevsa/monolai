<script lang="ts">
	import { page } from '$app/state';
	import { FileQuestionMark, Home } from '@lucide/svelte';

	let status = $derived(page.status || 404);
	let message = $derived(page.error?.message || 'Page Not Found');
</script>

<div class="flex items-center justify-center w-full h-screen bg-[var(--bg-primary)] p-6 box-border">
	<div
		class="flex flex-col items-center text-center max-w-[420px] w-full bg-[var(--bg-surface)] border border-[var(--border-color)] rounded-2xl p-10 shadow-2xl"
	>
		<div
			class="w-18 h-18 rounded-full bg-[var(--primary-light)] border border-[var(--primary-focus)] flex items-center justify-center mb-5"
		>
			<FileQuestionMark size={40} class="text-[var(--primary)]" />
		</div>

		<div class="text-4xl font-extrabold text-[var(--primary)] tracking-tight mb-1 leading-none">
			{status}
		</div>

		<h1 class="text-xl font-bold text-[var(--text-primary)] m-0 mb-2">
			{status === 404 ? 'Page Not Found' : 'An Error Occurred'}
		</h1>

		<p class="text-xs text-[var(--text-muted)] m-0 mb-7 leading-relaxed">
			{status === 404 ? 'The page you are looking for does not exist or has been moved.' : message}
		</p>

		<a
			href="/chat"
			class="inline-flex items-center gap-2 bg-[var(--primary)] text-white no-underline px-5 py-2.5 rounded-xl text-xs font-semibold transition-all duration-150 hover:bg-[var(--primary-hover)] hover:-translate-y-0.5"
		>
			<Home size={16} />
			<span>Return to Home</span>
		</a>
	</div>
</div>
