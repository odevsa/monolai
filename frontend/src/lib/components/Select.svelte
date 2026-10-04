<script lang="ts">
	import { onMount } from 'svelte';
	import { ChevronDown, Check } from '@lucide/svelte';

	interface Option {
		value: string;
		label: string;
	}

	let {
		value = $bindable(''),
		options = [],
		placeholder = 'Select an option...',
		class: className = '',
		ariaLabel = ''
	}: {
		value: string;
		options: Option[];
		placeholder?: string;
		class?: string;
		ariaLabel?: string;
	} = $props();

	let open = $state(false);
	let containerRef = $state<HTMLDivElement | null>(null);

	let selectedOption = $derived(options.find((o) => o.value === value));

	function toggleOpen() {
		open = !open;
	}

	function selectOption(val: string) {
		value = val;
		open = false;
	}

	function handleClickOutside(event: MouseEvent) {
		if (containerRef && !containerRef.contains(event.target as Node)) {
			open = false;
		}
	}

	function handleKeyDown(event: KeyboardEvent) {
		if (event.key === 'Escape' && open) {
			open = false;
		}
	}

	onMount(() => {
		document.addEventListener('click', handleClickOutside);
		document.addEventListener('keydown', handleKeyDown);
		return () => {
			document.removeEventListener('click', handleClickOutside);
			document.removeEventListener('keydown', handleKeyDown);
		};
	});
</script>

<div class="relative {className}" bind:this={containerRef}>
	<button
		type="button"
		class="flex items-center justify-between w-full px-3 py-2 text-xs font-medium rounded-lg cursor-pointer transition-colors duration-150 box-border bg-[var(--bg-primary)] text-[var(--text-primary)] hover:bg-[var(--bg-surface-hover)] border-0 select-none"
		onclick={toggleOpen}
		aria-expanded={open}
		aria-label={ariaLabel || placeholder}
	>
		<span class="truncate text-left mr-2">
			{selectedOption ? selectedOption.label : placeholder}
		</span>
		<ChevronDown
			size={13}
			class="transition-transform duration-150 text-[var(--text-muted)] shrink-0 {open ? 'rotate-180' : ''}"
		/>
	</button>

	{#if open}
		<div
			class="absolute top-[calc(100%+0.35rem)] left-0 right-0 z-50 p-1 rounded-xl shadow-xl bg-[var(--bg-surface)] border border-[var(--border-color)]/70 max-h-[240px] overflow-y-auto space-y-0.5 animate-in fade-in duration-150"
		>
			{#each options as opt (opt.value)}
				<button
					type="button"
					class="flex items-center justify-between w-full px-2.5 py-1.5 text-xs font-medium rounded-lg cursor-pointer transition-colors duration-120 text-left border-0 {value === opt.value
						? 'text-[var(--primary)] bg-[var(--primary-light)] font-semibold'
						: 'text-[var(--text-secondary)] bg-transparent hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)]'}"
					onclick={() => selectOption(opt.value)}
				>
					<span class="truncate">{opt.label}</span>
					{#if value === opt.value}
						<Check size={13} class="shrink-0 text-[var(--primary)]" />
					{/if}
				</button>
			{/each}
		</div>
	{/if}
</div>
