<script lang="ts">
	import { onMount } from 'svelte';
	import { Sun, Moon, Monitor, ChevronDown, Check, Palette } from '@lucide/svelte';
	import { THEMES, type ThemeId } from '$lib/themes';

	let {
		themeMode = $bindable<ThemeId>('system')
	}: {
		themeMode: ThemeId;
	} = $props();

	let open = $state(false);
	let containerRef = $state<HTMLDivElement | null>(null);

	let currentTheme = $derived(THEMES.find((t) => t.id === themeMode) || THEMES[0]);

	function toggleOpen() {
		open = !open;
	}

	function selectTheme(val: ThemeId) {
		themeMode = val;
		open = false;
	}

	function handleClickOutside(event: MouseEvent) {
		if (containerRef && !containerRef.contains(event.target as Node)) {
			open = false;
		}
	}

	onMount(() => {
		document.addEventListener('click', handleClickOutside);
		return () => document.removeEventListener('click', handleClickOutside);
	});
</script>

<div class="relative w-full" bind:this={containerRef}>
	<button
		type="button"
		class="flex items-center justify-between w-full px-3 py-2 text-xs font-medium border rounded-lg cursor-pointer transition-all duration-150 box-border border-[var(--border-color)] bg-[var(--bg-surface)] text-[var(--text-primary)] hover:border-[var(--border-hover)] hover:bg-[var(--bg-surface-hover)]"
		onclick={toggleOpen}
		aria-expanded={open}
		aria-label="Select Theme"
	>
		<div class="flex items-center gap-2">
			{#key currentTheme.id}
				{#if currentTheme.id === 'system'}
					<Monitor size={14} class="text-[var(--text-secondary)]" />
				{:else if currentTheme.id === 'dark'}
					<Moon size={14} class="text-[var(--primary)]" />
				{:else if currentTheme.id === 'light'}
					<Sun size={14} class="text-amber-500" />
				{:else}
					<Palette size={14} class="text-[var(--primary)]" />
				{/if}
			{/key}
			<span class="truncate max-w-[110px]">{currentTheme.name}</span>
		</div>
		<ChevronDown
			size={14}
			class="transition-transform duration-150 text-[var(--text-muted)] {open ? 'rotate-180' : ''}"
		/>
	</button>

	{#if open}
		<div
			class="absolute bottom-[calc(100%+0.35rem)] left-0 right-0 z-50 p-1 border rounded-lg shadow-xl bg-[var(--bg-surface)] border-[var(--border-hover)] max-h-[220px] overflow-y-auto space-y-0.5 animate-in fade-in slide-in-from-bottom-1 duration-150"
		>
			{#each THEMES as theme (theme.id)}
				<button
					type="button"
					class="flex items-center justify-between w-full px-2.5 py-1.5 text-xs font-medium rounded-md cursor-pointer transition-colors duration-120 text-left border-0 {themeMode ===
					theme.id
						? 'text-[var(--primary)] bg-[var(--primary-light)]'
						: 'text-[var(--text-secondary)] bg-transparent hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)]'}"
					onclick={() => selectTheme(theme.id)}
				>
					<div class="flex items-center gap-2 min-w-0">
						<span
							class="w-3 h-3 rounded-full border border-black/20 shrink-0 inline-block shadow-xs"
							style="background-color: {theme.preview.primary}"
						></span>
						<span class="truncate">{theme.name}</span>
					</div>
					{#if themeMode === theme.id}
						<Check size={13} class="shrink-0 text-[var(--primary)]" />
					{/if}
				</button>
			{/each}
		</div>
	{/if}
</div>
