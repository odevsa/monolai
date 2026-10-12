<script lang="ts">
	import { Button, EmptyState } from '$lib/components/ui';
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
		onUserInteraction?: () => void;
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
		onUserInteraction,
		onScrollToBottom
	}: LogsTerminalViewerProps = $props();
</script>

<div class="relative flex-1 min-h-[340px] flex flex-col overflow-hidden">
	<!-- Terminal Scrollable Body -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		bind:this={containerElement}
		onscroll={onScroll}
		onwheel={() => onUserInteraction?.()}
		ontouchmove={() => onUserInteraction?.()}
		onpointermove={(e) => {
			if (e.buttons > 0) onUserInteraction?.();
		}}
		onkeydown={(e) => {
			if (['ArrowUp', 'ArrowDown', 'PageUp', 'PageDown', 'Home', 'End', ' '].includes(e.key)) {
				onUserInteraction?.();
			}
		}}
		class="flex-1 min-h-[340px] overflow-y-auto overflow-x-auto p-3.5 pt-1 font-mono text-[11px] leading-relaxed select-text space-y-0.5 bg-transparent"
	>
		{#if isLoading && entries.length === 0}
			<div
				class="h-full min-h-[200px] flex items-center justify-center text-[var(--text-muted)] gap-2"
			>
				<RefreshCw size={16} class="animate-spin text-[var(--primary)]" />
				<span>{t('common.loading')}</span>
			</div>
		{:else if entries.length === 0}
			<EmptyState
				icon={ScrollText}
				title={t('logs.noLogs')}
				class="border-0 bg-transparent min-h-[200px]"
			/>
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
			<Button
				variant="glass"
				size="icon"
				class="rounded-full shadow-md hover:scale-105 active:scale-95"
				onclick={onScrollToBottom}
				title={t('logs.scrollToBottom')}
				ariaLabel={t('logs.scrollToBottom')}
			>
				<ChevronDown size={18} />
			</Button>
		</div>
	{/if}
</div>
