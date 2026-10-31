<script lang="ts">
	import { ProgressBar, Spinner } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import type { InstallProgress } from '$lib/types/runtimes';
	import { formatBytes } from '$lib/utils/format';

	interface RuntimeInstallProgressProps {
		progress: InstallProgress;
	}

	let { progress }: RuntimeInstallProgressProps = $props();

	let statusMessage = $derived(
		progress.message
			? progress.message
			: progress.status === 'extracting'
				? `${t('runtimes.extracting')}...`
				: `${t('runtimes.downloading')}...`
	);
</script>

<div
	class="w-full flex flex-col gap-2 p-3 rounded-xl bg-[var(--bg-primary)] border border-[var(--border-color)]"
>
	<div class="flex items-center justify-between gap-3 text-xs">
		<div class="flex items-center gap-2 text-[var(--text-primary)] min-w-0">
			<Spinner size={13} class="text-[var(--primary)] shrink-0" />
			<span class="font-medium truncate">{statusMessage}</span>
		</div>
		<span class="font-mono text-xs font-bold text-[var(--primary)] shrink-0">
			{progress.percent.toFixed(0)}%
		</span>
	</div>

	<ProgressBar value={progress.percent} size="md" variant="primary" />

	{#if progress.total_bytes > 0 || progress.speed_mbps > 0}
		<div
			class="flex items-center justify-between text-[11px] font-mono text-[var(--text-muted)] pt-0.5"
		>
			<span>
				{#if progress.total_bytes > 0}
					{t('sysinfo.usedOf', {
						used: formatBytes(progress.downloaded_bytes),
						total: formatBytes(progress.total_bytes)
					})}
				{/if}
			</span>
			<span>
				{#if progress.speed_mbps > 0}
					{progress.speed_mbps.toFixed(1)} MB/s
				{/if}
			</span>
		</div>
	{/if}
</div>
