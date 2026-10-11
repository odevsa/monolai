<script lang="ts">
	let {
		value = 0,
		max = 100,
		size = 'md',
		variant = 'primary',
		class: className = ''
	}: {
		value: number;
		max?: number;
		size?: 'sm' | 'md' | 'lg';
		variant?: 'primary' | 'success' | 'warning' | 'danger';
		class?: string;
	} = $props();

	let percent = $derived(max > 0 ? Math.max(0, Math.min(100, (value / max) * 100)) : 0);

	const sizeClasses = {
		sm: 'h-1.5',
		md: 'h-2',
		lg: 'h-3'
	};

	const variantClasses = {
		primary: 'bg-[var(--primary)]',
		success: 'bg-[var(--success)]',
		warning: 'bg-[var(--warning)]',
		danger: 'bg-[var(--danger)]'
	};
</script>

<div
	class="w-full rounded-full bg-[var(--bg-surface-hover)] overflow-hidden {sizeClasses[
		size
	]} {className}"
	role="progressbar"
	aria-valuenow={value}
	aria-valuemin={0}
	aria-valuemax={max}
>
	<div
		class="h-full rounded-full transition-all duration-200 {variantClasses[variant]}"
		style="width: {percent}%"
	></div>
</div>
