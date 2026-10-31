<script lang="ts">
	import { t } from '$lib/i18n';
	import { portal } from '$lib/utils/portal';
	import { Check, ChevronDown, Search } from '@lucide/svelte';
	import { onMount } from 'svelte';

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
		writable = false,
		searchPlaceholder = 'Search...',
		align = 'left',
		class: className = '',
		id = '',
		ariaLabel = '',
		disabled = false,
		onchange
	}: {
		value?: string;
		options: SelectOption[];
		placeholder?: string;
		searchable?: boolean;
		writable?: boolean;
		searchPlaceholder?: string;
		align?: 'left' | 'right';
		class?: string;
		id?: string;
		ariaLabel?: string;
		disabled?: boolean;
		onchange?: (val: string) => void;
	} = $props();

	let open = $state(false);
	let searchQuery = $state('');
	let containerRef = $state<HTMLDivElement | null>(null);
	let dropdownRef = $state<HTMLDivElement | null>(null);
	let coords = $state({ top: 0, left: 0, width: 0 });
	let transformOrigin = $state<'top' | 'bottom'>('top');
	let highlightedIndex = $state(-1);

	const listboxId = `select-listbox-${Math.random().toString(36).substring(2, 9)}`;

	let selectedOption = $derived(options.find((o) => o.value === value));

	let filteredOptions = $derived.by(() => {
		const q = (searchable ? searchQuery : writable ? value : '').toLowerCase().trim();
		if (!q) return options;
		return options.filter(
			(o) =>
				o.value.toLowerCase().includes(q) ||
				o.label.toLowerCase().includes(q) ||
				(o.sublabel && o.sublabel.toLowerCase().includes(q))
		);
	});

	function scrollHighlightedIntoView(index: number) {
		if (!dropdownRef || index < 0) return;
		const optionsElements = dropdownRef.querySelectorAll<HTMLButtonElement>('[role="option"]');
		const targetEl = optionsElements[index];
		if (targetEl) {
			targetEl.scrollIntoView({ block: 'nearest' });
		}
	}

	function updateCoords() {
		if (!containerRef) return;
		const rect = containerRef.getBoundingClientRect();
		const dropdownHeight = 260;
		const spaceBelow = window.innerHeight - rect.bottom;
		const spaceAbove = rect.top;

		let top = rect.bottom + 4;
		let origin: 'top' | 'bottom' = 'top';
		if (spaceBelow < dropdownHeight && spaceAbove > spaceBelow) {
			top = Math.max(8, rect.top - dropdownHeight - 4);
			origin = 'bottom';
		}

		transformOrigin = origin;
		coords = {
			top,
			left: align === 'right' ? rect.right - Math.max(rect.width, 240) : rect.left,
			width: Math.max(rect.width, 240)
		};
	}

	function toggleOpen() {
		if (disabled) return;
		if (!open) {
			openDropdown();
		} else {
			open = false;
		}
	}

	function openDropdown() {
		if (disabled || open) return;
		searchQuery = '';
		updateCoords();
		const currentIndex = filteredOptions.findIndex((o) => o.value === value);
		highlightedIndex = currentIndex >= 0 ? currentIndex : 0;
		open = true;
		setTimeout(() => scrollHighlightedIntoView(highlightedIndex), 0);
	}

	$effect(() => {
		if (open) {
			updateCoords();
			const handleScroll = (e: Event) => {
				if (dropdownRef && dropdownRef.contains(e.target as Node)) {
					return;
				}
				updateCoords();
			};
			window.addEventListener('scroll', handleScroll, true);
			window.addEventListener('resize', updateCoords);
			return () => {
				window.removeEventListener('scroll', handleScroll, true);
				window.removeEventListener('resize', updateCoords);
			};
		}
	});

	$effect(() => {
		if (open) {
			if (filteredOptions.length === 0) {
				highlightedIndex = -1;
			} else if (highlightedIndex >= filteredOptions.length || highlightedIndex < 0) {
				highlightedIndex = 0;
			}
		}
	});

	function selectOption(val: string) {
		value = val;
		open = false;
		if (onchange) {
			onchange(val);
		}
	}

	function handleKeyNavigation(e: KeyboardEvent) {
		if (disabled) return;

		if (!open) {
			if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
				e.preventDefault();
				openDropdown();
				const idx = filteredOptions.findIndex((o) => o.value === value);
				highlightedIndex =
					idx >= 0 ? idx : e.key === 'ArrowDown' ? 0 : Math.max(0, filteredOptions.length - 1);
				setTimeout(() => scrollHighlightedIntoView(highlightedIndex), 0);
				return;
			}
			if ((e.key === 'Enter' || e.key === ' ') && !writable) {
				e.preventDefault();
				toggleOpen();
				return;
			}
			return;
		}

		if (e.key === 'ArrowDown') {
			e.preventDefault();
			if (filteredOptions.length === 0) return;
			highlightedIndex = (highlightedIndex + 1) % filteredOptions.length;
			scrollHighlightedIntoView(highlightedIndex);
		} else if (e.key === 'ArrowUp') {
			e.preventDefault();
			if (filteredOptions.length === 0) return;
			highlightedIndex = (highlightedIndex - 1 + filteredOptions.length) % filteredOptions.length;
			scrollHighlightedIntoView(highlightedIndex);
		} else if (e.key === 'Enter') {
			if (highlightedIndex >= 0 && highlightedIndex < filteredOptions.length) {
				const opt = filteredOptions[highlightedIndex];
				if (!opt.disabled) {
					e.preventDefault();
					selectOption(opt.value);
				}
			}
		} else if (e.key === 'Escape') {
			e.preventDefault();
			e.stopPropagation();
			open = false;
		} else if (e.key === 'Tab') {
			open = false;
		} else if (e.key === 'Home' && !writable) {
			e.preventDefault();
			highlightedIndex = 0;
			scrollHighlightedIntoView(highlightedIndex);
		} else if (e.key === 'End' && !writable) {
			e.preventDefault();
			highlightedIndex = Math.max(0, filteredOptions.length - 1);
			scrollHighlightedIntoView(highlightedIndex);
		}
	}

	function handleClickOutside(event: MouseEvent) {
		const target = event.target as Node;
		if (
			containerRef &&
			!containerRef.contains(target) &&
			dropdownRef &&
			!dropdownRef.contains(target)
		) {
			open = false;
		}
	}

	onMount(() => {
		document.addEventListener('click', handleClickOutside, true);
		return () => {
			document.removeEventListener('click', handleClickOutside, true);
		};
	});
</script>

<div class="relative {open ? 'z-50' : 'z-10'} {className}" bind:this={containerRef}>
	{#if writable}
		<div
			class="flex items-center justify-between w-full h-[34px] px-2.5 py-1.5 text-xs font-medium rounded-lg transition-colors duration-150 box-border bg-[var(--bg-primary)] text-[var(--text-primary)] border border-[var(--border-color)] focus-within:border-[var(--primary)] {disabled
				? 'opacity-50 cursor-not-allowed pointer-events-none'
				: ''}"
		>
			<input
				{id}
				type="text"
				role="combobox"
				aria-autocomplete="list"
				aria-haspopup="listbox"
				aria-expanded={open}
				aria-controls={open ? listboxId : undefined}
				aria-activedescendant={open && highlightedIndex >= 0
					? `${listboxId}-opt-${highlightedIndex}`
					: undefined}
				class="w-full bg-transparent border-0 outline-none text-xs text-[var(--text-primary)] placeholder-[var(--text-muted)] font-mono min-w-0 truncate"
				{value}
				onfocus={openDropdown}
				onkeydown={handleKeyNavigation}
				oninput={(e) => {
					const val = (e.target as HTMLInputElement).value;
					value = val;
					if (!open) {
						updateCoords();
						open = true;
					}
					if (onchange) onchange(val);
				}}
				{placeholder}
				{disabled}
				aria-label={ariaLabel || placeholder}
			/>
			<button
				type="button"
				class="p-0.5 ml-1 bg-transparent border-0 cursor-pointer text-[var(--text-muted)] hover:text-[var(--text-primary)] transition-colors shrink-0 flex items-center"
				onclick={toggleOpen}
				tabindex="-1"
				aria-label="Toggle options"
			>
				<ChevronDown
					size={13}
					class="transition-transform duration-150 {open ? 'rotate-180' : ''}"
				/>
			</button>
		</div>
	{:else}
		<button
			{id}
			type="button"
			role="combobox"
			aria-haspopup="listbox"
			aria-expanded={open}
			aria-controls={open ? listboxId : undefined}
			aria-activedescendant={open && highlightedIndex >= 0
				? `${listboxId}-opt-${highlightedIndex}`
				: undefined}
			class="flex items-center justify-between w-full h-[34px] px-3 py-2 text-xs font-medium rounded-lg cursor-pointer transition-colors duration-150 box-border bg-[var(--bg-primary)] text-[var(--text-primary)] hover:bg-[var(--bg-surface-hover)] border border-[var(--border-color)] select-none {disabled
				? 'opacity-50 cursor-not-allowed pointer-events-none'
				: ''}"
			onclick={toggleOpen}
			onkeydown={handleKeyNavigation}
			{disabled}
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
	{/if}

	{#if open && coords.width > 0}
		<div
			use:portal
			bind:this={dropdownRef}
			id={listboxId}
			role="listbox"
			aria-label={ariaLabel || placeholder}
			style="position: fixed; top: {coords.top}px; left: {coords.left}px; width: {coords.width}px; z-index: 9999; transform-origin: {transformOrigin};"
			class="p-1.5 rounded-xl shadow-2xl bg-[var(--bg-surface)] border border-[var(--border-color)]/80 max-h-[260px] overflow-y-auto overflow-x-hidden space-y-0.5 backdrop-blur-xl animate-in fade-in zoom-in-95 duration-100"
		>
			{#if searchable}
				<div class="relative px-2 py-1 mb-1 border-b border-[var(--border-color)]">
					<Search size={12} class="absolute left-3.5 top-2.5 text-[var(--text-muted)]" />
					<input
						type="text"
						bind:value={searchQuery}
						onkeydown={handleKeyNavigation}
						placeholder={searchPlaceholder}
						class="w-full pl-6 pr-2 py-1 text-xs bg-transparent border-0 outline-none text-[var(--text-primary)] placeholder-[var(--text-muted)]"
					/>
				</div>
			{/if}

			{#if filteredOptions.length === 0}
				<div class="px-3 py-2 text-xs text-[var(--text-muted)] text-center">
					{t('common.noOptionsFound')}
				</div>
			{:else}
				{#each filteredOptions as opt, index (opt.value)}
					{@const isSelected = value === opt.value}
					{@const isHighlighted = index === highlightedIndex}
					<button
						id={`${listboxId}-opt-${index}`}
						role="option"
						aria-selected={isSelected}
						type="button"
						disabled={opt.disabled}
						class="flex items-center justify-between w-full px-2.5 py-1.5 text-xs font-medium rounded-lg cursor-pointer transition-colors duration-100 text-left border-0 disabled:opacity-40 disabled:cursor-not-allowed {isSelected
							? 'text-[var(--primary)] bg-[var(--primary-light)] font-semibold'
							: isHighlighted
								? 'text-[var(--text-primary)] bg-[var(--bg-hover)]'
								: 'text-[var(--text-secondary)] bg-transparent hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)]'}"
						onmouseenter={() => {
							highlightedIndex = index;
						}}
						onclick={() => selectOption(opt.value)}
					>
						<div class="truncate min-w-0 pr-2">
							<div class="truncate">{opt.label}</div>
							{#if opt.sublabel}
								<div class="text-[10px] text-[var(--text-muted)] truncate">{opt.sublabel}</div>
							{/if}
						</div>
						{#if isSelected}
							<Check size={13} class="shrink-0 text-[var(--primary)]" />
						{/if}
					</button>
				{/each}
			{/if}
		</div>
	{/if}
</div>
