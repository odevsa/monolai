<script lang="ts">
	let {
		value = $bindable(''),
		variant = 'surface',
		size = 'sm',
		resize = 'none',
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
		size?: 'sm' | 'md' | 'lg';
		resize?: 'none' | 'y' | 'x' | 'both';
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

	const sizeClasses = {
		sm: 'text-xs p-2.5',
		md: 'text-sm p-3',
		lg: 'text-base p-3.5'
	};

	const resizeClasses = {
		none: 'resize-none',
		y: 'resize-y',
		x: 'resize-x',
		both: 'resize'
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
	class="w-full text-[var(--text-primary)] placeholder-[var(--text-muted)] rounded-lg border transition-all duration-150 outline-none leading-relaxed {variantClasses[
		variant
	]} {sizeClasses[size]} {resizeClasses[resize]} {error
		? 'border-[var(--danger)] focus:ring-2 focus:ring-[var(--danger-focus)]'
		: ''} {disabled ? 'opacity-50 cursor-not-allowed pointer-events-none' : ''} {className}"
></textarea>
