<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLInputAttributes } from 'svelte/elements';
	import { X } from '@lucide/svelte';

	let {
		value = $bindable(''),
		type = 'text',
		placeholder = '',
		disabled = false,
		error = false,
		clearable = false,
		class: className = '',
		id = '',
		name = '',
		autocomplete = 'off',
		oninput,
		onchange,
		onkeydown,
		prefix,
		suffix
	}: {
		value?: string | number;
		type?: string;
		placeholder?: string;
		disabled?: boolean;
		error?: boolean;
		clearable?: boolean;
		class?: string;
		id?: string;
		name?: string;
		autocomplete?: HTMLInputAttributes['autocomplete'];
		oninput?: (e: Event) => void;
		onchange?: (e: Event) => void;
		onkeydown?: (e: KeyboardEvent) => void;
		prefix?: Snippet;
		suffix?: Snippet;
	} = $props();

	function handleClear() {
		value = '';
	}
</script>

<div
	class="relative flex items-center w-full rounded-lg bg-[var(--bg-primary)] border transition-all duration-150 {error
		? 'border-red-500/50 focus-within:ring-2 focus-within:ring-red-500/20'
		: 'border-[var(--border-color)] focus-within:border-[var(--primary)] focus-within:ring-1 focus-within:ring-[var(--primary)]/40'} {disabled
		? 'opacity-50 cursor-not-allowed pointer-events-none'
		: ''} {className}"
>
	{#if prefix}
		<div class="flex items-center pl-3 text-[var(--text-muted)] pointer-events-none shrink-0">
			{@render prefix()}
		</div>
	{/if}

	<input
		{id}
		{name}
		{type}
		bind:value
		{placeholder}
		{disabled}
		{autocomplete}
		{oninput}
		{onchange}
		{onkeydown}
		class="w-full bg-transparent text-[var(--text-primary)] placeholder-[var(--text-muted)] text-xs font-normal py-2 px-3 border-0 outline-none select-text min-w-0"
	/>

	{#if clearable && value && !disabled}
		<button
			type="button"
			class="pr-2 text-[var(--text-muted)] hover:text-[var(--text-primary)] transition-colors p-1"
			onclick={handleClear}
			aria-label="Clear input"
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
