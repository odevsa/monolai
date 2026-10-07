<script lang="ts">
	export interface TabItem {
		id: string;
		label: string;
		icon?: any;
		badge?: string | number;
	}

	let {
		value = $bindable(''),
		items = [],
		class: className = '',
		onchange
	}: {
		value?: string;
		items: TabItem[];
		class?: string;
		onchange?: (id: string) => void;
	} = $props();

	function selectTab(id: string) {
		value = id;
		if (onchange) onchange(id);
	}
</script>

<div
	class="inline-flex items-center p-1 rounded-xl bg-[var(--bg-primary)] border border-[var(--border-color)] gap-1 select-none {className}"
	role="tablist"
>
	{#each items as item (item.id)}
		{@const Icon = item.icon}
		<button
			type="button"
			role="tab"
			aria-selected={value === item.id}
			class="flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-semibold cursor-pointer transition-all duration-150 border-0 {value ===
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
