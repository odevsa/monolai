<script lang="ts">
	let {
		checked = $bindable(false),
		disabled = false,
		id = undefined,
		ariaLabel = undefined,
		size = 'md',
		class: className = '',
		onchange
	}: {
		checked?: boolean;
		disabled?: boolean;
		id?: string;
		ariaLabel?: string;
		size?: 'sm' | 'md';
		class?: string;
		onchange?: (checked: boolean) => void;
	} = $props();

	function handleToggle() {
		if (disabled) return;
		checked = !checked;
		onchange?.(checked);
	}

	function handleKeyDown(e: KeyboardEvent) {
		if (disabled) return;
		if (e.key === ' ' || e.key === 'Enter') {
			e.preventDefault();
			handleToggle();
		}
	}
</script>

<button
	type="button"
	role="switch"
	aria-checked={checked}
	aria-label={ariaLabel}
	{id}
	{disabled}
	onclick={handleToggle}
	onkeydown={handleKeyDown}
	class="relative inline-flex items-center shrink-0 cursor-pointer select-none rounded-full transition-colors duration-200 ease-in-out outline-none focus-visible:ring-2 focus-visible:ring-[var(--primary)]/50 focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--bg-surface)] disabled:opacity-50 disabled:cursor-not-allowed {size ===
	'sm'
		? 'w-9 h-5'
		: 'w-11 h-6'} {checked
		? 'bg-[var(--primary)]'
		: 'bg-[var(--border-color)] hover:bg-[var(--border-hover)]'} {className}"
>
	<span
		class="inline-block rounded-full bg-[var(--btn-text)] shadow-xs transform transition-transform duration-200 ease-in-out pointer-events-none {size ===
		'sm'
			? 'w-4 h-4'
			: 'w-5 h-5'} {checked
			? size === 'sm'
				? 'translate-x-[18px]'
				: 'translate-x-[22px]'
			: 'translate-x-[2px]'}"
	></span>
</button>
