<script lang="ts">
	let {
		value = $bindable(''),
		variant = 'surface',
		rows = 3,
		placeholder = '',
		disabled = false,
		error = false,
		class: className = '',
		id = '',
		oninput,
		onkeydown
	}: {
		value?: string;
		variant?: 'surface' | 'glass';
		rows?: number;
		placeholder?: string;
		disabled?: boolean;
		error?: boolean;
		class?: string;
		id?: string;
		oninput?: (e: Event) => void;
		onkeydown?: (e: KeyboardEvent) => void;
	} = $props();

	const variantClasses = {
		surface:
			'bg-[var(--bg-primary)] border-[var(--border-color)] focus:border-[var(--primary)] focus:ring-1 focus:ring-[var(--primary)]/40',
		glass:
			'bg-[var(--bg-surface)]/40 backdrop-blur-md border-[var(--border-color)]/70 focus:border-[var(--border-hover)] focus:bg-[var(--bg-surface)]/70'
	};
</script>

<textarea
	{id}
	bind:value
	{rows}
	{placeholder}
	{disabled}
	{oninput}
	{onkeydown}
	class="w-full text-xs text-[var(--text-primary)] placeholder-[var(--text-muted)] rounded-lg p-2.5 border transition-all duration-150 outline-none resize-none leading-relaxed {variantClasses[
		variant
	]} {error ? 'border-red-500/50 focus:ring-red-500/20' : ''} {disabled
		? 'opacity-50 cursor-not-allowed pointer-events-none'
		: ''} {className}"></textarea>
