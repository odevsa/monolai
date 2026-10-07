<script lang="ts">
	import { onMount } from 'svelte';
	import { ChevronDown, Check, Search } from '@lucide/svelte';

	export interface SelectOption {
		value: string;
		label: string;
		sublabel?: string;
		disabled?: boolean;
	}

	let {
		value = $bindable(''),
		options = [],
		placeholder = 'Select an option...',
		searchable = false,
		searchPlaceholder = 'Search...',
		class: className = '',
		ariaLabel = '',
		disabled = false,
		onchange
	}: {
		value?: string;
		options: SelectOption[];
		placeholder?: string;
		searchable?: boolean;
		searchPlaceholder?: string;
		class?: string;
		ariaLabel?: string;
		disabled?: boolean;
		onchange?: (val: string) => void;
	} = $props();

	let open = $state(false);
	let searchQuery = $state('');
	let containerRef = $state<HTMLDivElement | null>(null);

	let selectedOption = $derived(options.find((o) => o.value === value));

	let filteredOptions = $derived.by(() => {
		if (!searchable || !searchQuery.trim()) return options;
		const q = searchQuery.toLowerCase().trim();
		return options.filter(
			(o) =>
				o.label.toLowerCase().includes(q) || (o.sublabel && o.sublabel.toLowerCase().includes(q))
		);
	});

	function toggleOpen() {
		if (disabled) return;
		open = !open;
		if (open) searchQuery = '';
	}

	function selectOption(val: string) {
		value = val;
		open = false;
		if (onchange) {
			onchange(val);
		}
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

<div class="relative {open ? 'z-40' : 'z-10'} {className}" bind:this={containerRef}>
	<button
		type="button"
		class="flex items-center justify-between w-full px-3 py-2 text-xs font-medium rounded-lg cursor-pointer transition-colors duration-150 box-border bg-[var(--bg-primary)] text-[var(--text-primary)] hover:bg-[var(--bg-surface-hover)] border border-[var(--border-color)] select-none {disabled
			? 'opacity-50 cursor-not-allowed pointer-events-none'
			: ''}"
		onclick={toggleOpen}
		{disabled}
		aria-expanded={open}
		aria-label={ariaLabel || placeholder}
	>
		<span class="truncate text-left mr-2">
			{selectedOption ? selectedOption.label : placeholder}
		</span>
		<ChevronDown
			size={13}
			class="transition-transform duration-150 text-[var(--text-muted)] shrink-0 {open
				? 'rotate-180'
				: ''}"
		/>
	</button>

	{#if open}
		<div
			class="absolute top-[calc(100%+0.35rem)] left-0 right-0 z-50 p-1.5 rounded-xl shadow-2xl bg-[var(--bg-surface)] border border-[var(--border-color)]/80 max-h-[260px] overflow-y-auto overflow-x-hidden space-y-0.5 backdrop-blur-xl animate-in fade-in zoom-in-95 duration-150"
		>
			{#if searchable}
				<div class="relative px-2 py-1 mb-1 border-b border-[var(--border-color)]">
					<Search size={12} class="absolute left-3.5 top-2.5 text-[var(--text-muted)]" />
					<input
						type="text"
						bind:value={searchQuery}
						placeholder={searchPlaceholder}
						class="w-full pl-6 pr-2 py-1 text-xs bg-transparent border-0 outline-none text-[var(--text-primary)] placeholder-[var(--text-muted)]"
					/>
				</div>
			{/if}

			{#if filteredOptions.length === 0}
				<div class="px-3 py-2 text-xs text-[var(--text-muted)] text-center">No options found</div>
			{:else}
				{#each filteredOptions as opt (opt.value)}
					<button
						type="button"
						disabled={opt.disabled}
						class="flex items-center justify-between w-full px-2.5 py-1.5 text-xs font-medium rounded-lg cursor-pointer transition-colors duration-100 text-left border-0 disabled:opacity-40 disabled:cursor-not-allowed {value ===
						opt.value
							? 'text-[var(--primary)] bg-[var(--primary-light)] font-semibold'
							: 'text-[var(--text-secondary)] bg-transparent hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)]'}"
						onclick={() => selectOption(opt.value)}
					>
						<div class="truncate min-w-0 pr-2">
							<div class="truncate">{opt.label}</div>
							{#if opt.sublabel}
								<div class="text-[10px] text-[var(--text-muted)] truncate">{opt.sublabel}</div>
							{/if}
						</div>
						{#if value === opt.value}
							<Check size={13} class="shrink-0 text-[var(--primary)]" />
						{/if}
					</button>
				{/each}
			{/if}
		</div>
	{/if}
</div>
