<script lang="ts">
	import { t } from '$lib/i18n';
	import type { LogEntry } from '$lib/types/logs';
	import { ChevronDown, RefreshCw, ScrollText } from '@lucide/svelte';
	import LogsEntryRow from './LogsEntryRow.svelte';

	interface LogsTerminalViewerProps {
		entries: LogEntry[];
		isLoading: boolean;
		showModelTag: boolean;
		autoScroll: boolean;
		isScrolledUp: boolean;
		containerElement?: HTMLDivElement | null;
		onScroll: () => void;
		onScrollToBottom: () => void;
	}

	let {
		entries,
		isLoading,
		showModelTag,
		autoScroll,
		isScrolledUp,
		containerElement = $bindable(null),
		onScroll,
		onScrollToBottom
	}: LogsTerminalViewerProps = $props();
</script>

<div class="relative flex-1 min-h-[340px] flex flex-col overflow-hidden">
	<!-- Terminal Scrollable Body -->
	<div
		bind:this={containerElement}
		onscroll={onScroll}
		class="flex-1 min-h-[340px] overflow-y-auto overflow-x-auto p-3.5 pt-1 font-mono text-[11px] leading-relaxed select-text space-y-0.5 scroll-smooth bg-transparent"
	>
		{#if isLoading && entries.length === 0}
			<div
				class="h-full min-h-[200px] flex items-center justify-center text-[var(--text-muted)] gap-2"
			>
				<RefreshCw size={16} class="animate-spin text-[var(--primary)]" />
				<span>{t('common.loading')}</span>
			</div>
		{:else if entries.length === 0}
			<div
				class="h-full min-h-[200px] flex flex-col items-center justify-center text-[var(--text-muted)] gap-2"
			>
				<ScrollText size={28} class="opacity-30" />
				<p class="text-xs">{t('logs.noLogs')}</p>
			</div>
		{:else}
			{#each entries as entry (entry.id)}
				<LogsEntryRow {entry} {showModelTag} />
			{/each}
		{/if}
	</div>

	<!-- Floating Center-Bottom Resume Scroll Button -->
	{#if !autoScroll && isScrolledUp && entries.length > 0}
		<div
			class="absolute bottom-4 left-1/2 -translate-x-1/2 z-20 animate-in fade-in zoom-in-95 duration-150 pointer-events-auto"
		>
			<button
				type="button"
				class="w-9 h-9 rounded-full bg-[var(--bg-surface)]/85 hover:bg-[var(--bg-hover)] backdrop-blur-md border border-[var(--border-color)]/80 text-[var(--text-primary)] flex items-center justify-center cursor-pointer shadow-md hover:scale-105 active:scale-95 transition-all"
				onclick={onScrollToBottom}
				title={t('logs.scrollToBottom')}
				aria-label={t('logs.scrollToBottom')}
			>
				<ChevronDown size={18} />
			</button>
		</div>
	{/if}
</div>
