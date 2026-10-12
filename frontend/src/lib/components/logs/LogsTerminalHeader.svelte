<script lang="ts">
	import { Button } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import { ArrowDown, Box, Download, RefreshCw, Trash } from '@lucide/svelte';

	interface LogsTerminalHeaderProps {
		filteredCount: number;
		showModelTag: boolean;
		autoScroll: boolean;
		isLoading: boolean;
		onToggleModelTag: () => void;
		onToggleAutoScroll: () => void;
		onRefresh: () => void;
		onDownload: () => void;
		onClear: () => void;
	}

	let {
		filteredCount,
		showModelTag,
		autoScroll,
		isLoading,
		onToggleModelTag,
		onToggleAutoScroll,
		onRefresh,
		onDownload,
		onClear
	}: LogsTerminalHeaderProps = $props();
</script>

<div
	class="flex items-center justify-between px-3.5 py-2.5 bg-transparent border-0 select-none shrink-0"
>
	<!-- Left: Title & Line Count -->
	<div class="flex items-center gap-2">
		<span
			class="text-xs font-semibold tracking-wider uppercase text-[var(--text-secondary)] font-mono"
		>
			{t('logs.terminalTitle')}
		</span>
		<span class="text-[11px] text-[var(--text-muted)] font-mono">
			({t('logs.linesCount', { count: filteredCount })})
		</span>
	</div>

	<!-- Right: Action Icons Toolbar -->
	<div class="flex items-center gap-1 text-[var(--text-muted)]">
		<!-- Model Tag Visibility Toggle -->
		<Button
			variant="ghost"
			size="icon"
			onclick={onToggleModelTag}
			title={showModelTag ? t('logs.hideModelTag') : t('logs.showModelTag')}
			ariaLabel={showModelTag ? t('logs.hideModelTag') : t('logs.showModelTag')}
			class="w-7 h-7 {showModelTag ? 'text-[var(--primary)]' : 'text-[var(--text-muted)]'}"
		>
			<Box size={15} class={showModelTag ? 'text-[var(--primary)]' : 'opacity-60'} />
		</Button>

		<!-- AutoScroll Toggle -->
		<Button
			variant="ghost"
			size="icon"
			onclick={onToggleAutoScroll}
			title={autoScroll ? t('logs.autoScrollEnabled') : t('logs.autoScrollDisabled')}
			ariaLabel={t('logs.autoScroll')}
			class="w-7 h-7 {autoScroll ? 'text-[var(--primary)]' : 'text-[var(--text-muted)]'}"
		>
			<ArrowDown size={16} class={autoScroll ? 'text-[var(--primary)]' : 'opacity-60'} />
		</Button>

		<!-- Refresh Logs -->
		<Button
			variant="ghost"
			size="icon"
			onclick={onRefresh}
			disabled={isLoading}
			title={t('common.refresh')}
			ariaLabel={t('common.refresh')}
			class="w-7 h-7 text-[var(--text-muted)]"
		>
			<RefreshCw size={15} class={isLoading ? 'animate-spin text-[var(--primary)]' : ''} />
		</Button>

		<!-- Download Log File -->
		<Button
			variant="ghost"
			size="icon"
			onclick={onDownload}
			title={t('common.download')}
			ariaLabel={t('common.download')}
			class="w-7 h-7 text-[var(--text-muted)]"
		>
			<Download size={15} />
		</Button>

		<!-- Clear Log View -->
		<Button
			variant="ghost"
			size="icon"
			onclick={onClear}
			title={t('common.clear')}
			ariaLabel={t('common.clear')}
			class="w-7 h-7 text-[var(--text-muted)] hover:text-[var(--danger-text)] hover:bg-[var(--danger-light)]"
		>
			<Trash size={15} />
		</Button>
	</div>
</div>
