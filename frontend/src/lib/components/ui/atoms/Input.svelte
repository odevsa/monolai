<script lang="ts">
	import { t } from '$lib/i18n';
	import { X } from '@lucide/svelte';
	import type { Snippet } from 'svelte';
	import type { HTMLInputAttributes } from 'svelte/elements';

	let {
		value = $bindable(''),
		type = 'text',
		size = 'md',
		variant = 'default',
		placeholder = '',
		disabled = false,
		error = false,
		clearable = false,
		class: className = '',
		inputClass = '',
		id = '',
		name = '',
		autocomplete = 'off',
		min = undefined,
		max = undefined,
		step = undefined,
		tabindex = undefined,
		spellcheck = undefined,
		oninput,
		onchange,
		onkeydown,
		prefix,
		suffix,
		...restProps
	}: {
		value?: string | number;
		type?: string;
		size?: 'sm' | 'md' | 'lg';
		variant?: 'default' | 'surface' | 'ghost';
		placeholder?: string;
		disabled?: boolean;
		error?: boolean;
		clearable?: boolean;
		class?: string;
		inputClass?: string;
		id?: string;
		name?: string;
		autocomplete?: HTMLInputAttributes['autocomplete'];
		spellcheck?: boolean;
		min?: number | string;
		max?: number | string;
		step?: number | string;
		tabindex?: number | undefined | null;
		oninput?: (e: Event) => void;
		onchange?: (e: Event) => void;
		onkeydown?: (e: KeyboardEvent) => void;
		prefix?: Snippet;
		suffix?: Snippet;
		[key: string]: any;
	} = $props();

	const variantClasses = {
		default: 'bg-[var(--bg-primary)] border-[var(--border-color)]',
		surface: 'bg-[var(--bg-surface)] border-[var(--border-color)]',
		ghost: 'bg-transparent border-transparent'
	};

	const sizeClasses = {
		sm: 'py-1.5 px-2.5 text-xs',
		md: 'py-2 px-3 text-xs',
		lg: 'py-2.5 px-3.5 text-sm'
	};

	const prefixPadClasses = {
		sm: 'pl-2.5',
		md: 'pl-3',
		lg: 'pl-3.5'
	};

	function handleClear() {
		value = '';
	}

	function handleKeyDown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			(e.currentTarget as HTMLInputElement)?.blur();
		}
		if (onkeydown) onkeydown(e);
	}
</script>

<div
	title={restProps.title}
	class="relative flex items-center w-full rounded-lg border transition-all duration-150 {variantClasses[
		variant
	]} {error
		? 'border-[var(--danger)] focus-within:ring-2 focus-within:ring-[var(--danger-focus)]'
		: 'focus-within:border-[var(--primary)] focus-within:ring-1 focus-within:ring-[var(--primary)]/40'} {disabled
		? 'opacity-50 cursor-not-allowed pointer-events-none'
		: ''} {className}"
>
	{#if prefix}
		<div
			class="flex items-center {prefixPadClasses[
				size
			]} text-[var(--text-muted)] pointer-events-none shrink-0"
		>
			{@render prefix()}
		</div>
	{/if}

	<input
		{id}
		{name}
		{type}
		{min}
		{max}
		{step}
		bind:value
		{placeholder}
		{disabled}
		{autocomplete}
		{spellcheck}
		{oninput}
		{onchange}
		onkeydown={handleKeyDown}
		enterkeyhint={restProps.enterkeyhint || (type === 'search' ? 'search' : undefined)}
		{tabindex}
		{...restProps}
		class="w-full bg-transparent text-[var(--text-primary)] placeholder-[var(--text-muted)] font-normal border-0 outline-none select-text min-w-0 {prefix
			? 'pl-1.5'
			: ''} {sizeClasses[size]} {inputClass}"
	/>

	{#if clearable && value && !disabled}
		<button
			type="button"
			class="pr-2 text-[var(--text-muted)] hover:text-[var(--text-primary)] transition-colors p-1"
			onclick={handleClear}
			aria-label={t('common.clear')}
		>
			<X size={14} />
		</button>
	{/if}

	{#if suffix}
		<div class="flex items-center pr-3 text-[var(--text-muted)] shrink-0">
			{@render suffix()}
		</div>
	{/if}
</div>
