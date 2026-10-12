<script lang="ts">
	import type { Snippet } from 'svelte';

	export interface TabItem {
		id: string;
		label: string;
		icon?: any;
		badge?: string | number;
	}

	let {
		value = $bindable(''),
		items = [],
		variant = 'card',
		surface = 'light',
		title,
		subtitle,
		header,
		action,
		tabsAction,
		children,
		class: className = '',
		cardClass = '',
		onchange
	}: {
		value?: string;
		items: TabItem[];
		variant?: 'card' | 'pills';
		surface?: 'light' | 'dark' | 'primary' | 'secondary';
		title?: string;
		subtitle?: string;
		header?: Snippet;
		action?: Snippet;
		tabsAction?: Snippet;
		children?: Snippet;
		class?: string;
		cardClass?: string;
		onchange?: (id: string) => void;
	} = $props();

	// Auto-select first item if value is empty and items are provided
	$effect(() => {
		if (!value && items.length > 0) {
			value = items[0].id;
		}
	});

	let activeIndex = $derived(items.findIndex((item) => item.id === value));

	function selectTab(id: string) {
		value = id;
		if (onchange) onchange(id);
	}

	let bgClass = $derived.by(() => {
		switch (surface) {
			case 'dark':
				return 'bg-[var(--color-black)]/30';
			case 'primary':
				return 'bg-[var(--color-primary)]/10';
			case 'secondary':
				return 'bg-[var(--color-secondary)]/10';
			default:
				return 'bg-[var(--color-white)]/5';
		}
	});
</script>

{#if variant === 'card'}
	<div class="w-full flex flex-col {className}">
		<!-- Title / Header above tabs outside the card -->
		{#if header}
			{@render header()}
		{:else if title || subtitle || action}
			<div class="flex items-center justify-between gap-4 pb-3.5 sm:pb-4">
				<div class="flex flex-col gap-0.5 min-w-0">
					{#if title}
						<h3 class="m-0 text-base sm:text-lg font-bold text-[var(--text-primary)] truncate">
							{title}
						</h3>
					{/if}
					{#if subtitle}
						<p class="m-0 text-xs text-[var(--text-muted)]">
							{subtitle}
						</p>
					{/if}
				</div>
				{#if action}
					<div class="shrink-0">
						{@render action()}
					</div>
				{/if}
			</div>
		{/if}

		<!-- Tabs bar sitting on top of the card -->
		{#if items.length > 0}
			<div class="flex items-end justify-between relative z-10">
				<div class="flex items-end gap-1.5" role="tablist">
					{#each items as item, idx (item.id)}
						{@const Icon = item.icon}
						{@const isActive = value === item.id}
						<button
							type="button"
							role="tab"
							aria-selected={isActive}
							class="flex items-center gap-2 px-4 sm:px-5 py-2.5 text-xs sm:text-sm font-semibold cursor-pointer transition-all {isActive
								? `${bgClass} text-[var(--text-primary)] rounded-t-xl relative z-10`
								: 'bg-transparent text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)]/30 rounded-t-lg'}"
							onclick={() => selectTab(item.id)}
						>
							{#if Icon}
								<Icon size={15} class="shrink-0" />
							{/if}
							<span>{item.label}</span>
							{#if item.badge !== undefined}
								<span
									class="px-1.5 py-0.2 rounded-full text-[10px] font-bold bg-[var(--primary-light)] text-[var(--primary)]"
								>
									{item.badge}
								</span>
							{/if}
						</button>
					{/each}
				</div>

				{#if tabsAction}
					<div class="pb-1.5 shrink-0">
						{@render tabsAction()}
					</div>
				{/if}
			</div>
		{/if}

		<!-- Connected content card -->
		{#if children}
			<div
				class="{bgClass} rounded-b-2xl {items.length === 0
					? 'rounded-t-2xl'
					: activeIndex === 0
						? 'rounded-tr-2xl rounded-tl-none'
						: 'rounded-t-2xl'} shadow-sm {cardClass || 'p-5 sm:p-6'}"
			>
				{@render children()}
			</div>
		{/if}
	</div>
{:else}
	<!-- Pills Variant -->
	<div class="flex flex-col gap-2.5 {className}">
		<div
			class="inline-flex items-center p-1 rounded-xl bg-[var(--bg-primary)] gap-1 select-none w-fit"
			role="tablist"
		>
			{#each items as item (item.id)}
				{@const Icon = item.icon}
				<button
					type="button"
					role="tab"
					aria-selected={value === item.id}
					class="flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-semibold cursor-pointer transition-all duration-150 {value ===
					item.id
						? 'bg-[var(--bg-surface)] text-[var(--text-primary)] shadow-sm'
						: 'bg-transparent text-[var(--text-muted)] hover:text-[var(--text-secondary)] hover:bg-[var(--bg-hover)]'}"
					onclick={() => selectTab(item.id)}
				>
					{#if Icon}
						<Icon size={14} class="shrink-0" />
					{/if}
					<span>{item.label}</span>
					{#if item.badge !== undefined}
						<span
							class="px-1.5 py-0.2 rounded-full text-[10px] font-bold bg-[var(--primary-light)] text-[var(--primary)]"
						>
							{item.badge}
						</span>
					{/if}
				</button>
			{/each}
		</div>

		{#if children}
			<div class={cardClass}>
				{@render children()}
			</div>
		{/if}
	</div>
{/if}
