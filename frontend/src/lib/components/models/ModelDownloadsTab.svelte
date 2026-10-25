<script lang="ts">
	import { Badge, Button, EmptyState } from '$lib/components/ui';
	import { askConfirm } from '$lib/confirmStore';
	import { t } from '$lib/i18n';
	import { downloadStore } from '$lib/state/downloads.svelte';
	import type { DownloadRecord } from '$lib/types/models';
	import { formatBytes, formatSpeed } from '$lib/utils/format';
	import { CircleAlert, ArrowDownToLine, CircleCheck, Pause, Play, Trash, X } from '@lucide/svelte';

	let downloads = $derived(downloadStore.downloads);

	function getDisplayName(download: DownloadRecord): string {
		if (download.filename) return download.filename;
		if (download.file_path) {
			const parts = download.file_path.split('/');
			return parts[parts.length - 1] || download.file_path;
		}
		return download.model_name || download.id;
	}

	function calculateProgress(download: DownloadRecord): number {
		if (!download.total_bytes || download.total_bytes <= 0) return 0;
		return Math.min(100, Math.round((download.downloaded_bytes / download.total_bytes) * 100));
	}

	async function handleDelete(d: DownloadRecord) {
		const name = getDisplayName(d);
		const confirmed = await askConfirm(
			t('models.downloads.deleteConfirm', { name }),
			t('common.delete'),
			'danger',
			t('common.delete'),
			t('common.cancel')
		);
		if (!confirmed) return;

		try {
			await downloadStore.delete(d.id);
		} catch (err: unknown) {
			console.error('Failed to delete download record:', err);
		}
	}
</script>

<div class="space-y-4">
	<!-- Summary header when downloads active -->
	{#if downloadStore.hasActive}
		<div
			class="bg-[var(--primary-light)] border border-[var(--primary)]/30 rounded-xl p-3.5 flex items-center justify-between gap-3 text-xs"
		>
			<div class="flex items-center gap-2">
				<ArrowDownToLine size={16} class="text-[var(--primary)] animate-bounce" />
				<span class="font-semibold text-[var(--primary)]">
					{t('models.downloads.activeDownloadsCount', { count: downloadStore.activeCount })}
				</span>
			</div>

			<span class="font-mono font-bold text-[var(--primary)]">
				{formatSpeed(downloadStore.totalSpeedBytesSec)}
			</span>
		</div>
	{/if}

	{#if downloads.length === 0}
		<EmptyState
			icon={ArrowDownToLine}
			title={t('models.downloads.emptyTitle')}
			description={t('models.downloads.emptyDesc')}
		/>
	{:else}
		<div class="space-y-3">
			{#each downloads as d (d.id)}
				{@const progress = calculateProgress(d)}
				{@const displayName = getDisplayName(d)}
				<div
					class="bg-[var(--bg-surface)] rounded-xl p-4 space-y-3 shadow-xs transition-all hover:border-[var(--border-hover)]"
				>
					<!-- Top Row: File Name, Model ID, Status Badge, Action Controls -->
					<div class="flex items-start justify-between gap-3 flex-wrap">
						<div class="min-w-0 flex-1">
							<div class="flex items-center gap-2 flex-wrap">
								<h4 class="text-sm font-semibold text-[var(--text-primary)] m-0 font-mono truncate">
									{displayName}
								</h4>
								{#if d.status === 'downloading'}
									<Badge variant="primary" pulse>
										{t('common.downloading')}
									</Badge>
								{:else if d.status === 'paused'}
									<Badge variant="warning">
										{t('models.downloads.paused')}
									</Badge>
								{:else if d.status === 'completed'}
									<Badge variant="success">
										{t('models.downloads.completed')}
									</Badge>
								{:else if d.status === 'failed'}
									<Badge variant="danger">
										{t('models.downloads.failed')}
									</Badge>
								{:else}
									<Badge variant="muted">
										{d.status}
									</Badge>
								{/if}
							</div>
							<span class="text-[11px] text-[var(--text-muted)] font-mono block mt-0.5 truncate">
								{d.model_name}
							</span>
						</div>

						<!-- Action Controls -->
						<div class="flex items-center gap-1.5 shrink-0">
							{#if d.status === 'downloading'}
								<Button
									variant="ghost"
									size="icon-sm"
									onclick={() => downloadStore.pause(d.id)}
									title={t('models.downloads.pause')}
									ariaLabel={t('models.downloads.pause')}
								>
									<Pause size={14} />
								</Button>
								<Button
									variant="ghost"
									size="icon-sm"
									onclick={() => downloadStore.cancel(d.id)}
									title={t('common.cancel')}
									ariaLabel={t('common.cancel')}
								>
									<X size={14} class="text-[var(--danger-text)]" />
								</Button>
							{:else if d.status === 'paused'}
								<Button
									variant="ghost"
									size="icon-sm"
									onclick={() => downloadStore.resume(d.id)}
									title={t('models.downloads.resume')}
									ariaLabel={t('models.downloads.resume')}
								>
									<Play size={14} class="text-[var(--success-text)]" />
								</Button>
								<Button
									variant="ghost"
									size="icon-sm"
									onclick={() => downloadStore.cancel(d.id)}
									title={t('common.cancel')}
									ariaLabel={t('common.cancel')}
								>
									<X size={14} class="text-[var(--danger-text)]" />
								</Button>
							{:else}
								<Button
									variant="ghost"
									size="icon-sm"
									onclick={() => handleDelete(d)}
									title={t('common.delete')}
									ariaLabel={t('common.delete')}
								>
									<Trash size={14} class="text-[var(--danger-text)]" />
								</Button>
							{/if}
						</div>
					</div>

					<!-- Progress Bar and Numbers -->
					{#if d.status === 'downloading' || d.status === 'paused'}
						<div class="space-y-1.5">
							<div
								class="w-full bg-[var(--bg-primary)] rounded-full h-2 overflow-hidden border border-[var(--border-color)]/60"
							>
								<div
									class="bg-[var(--primary)] h-full transition-all duration-300 rounded-full"
									style="width: {progress}%"
								></div>
							</div>

							<div
								class="flex items-center justify-between text-[11px] text-[var(--text-muted)] font-mono"
							>
								<span>
									{formatBytes(d.downloaded_bytes)} / {formatBytes(d.total_bytes)} ({progress}%)
								</span>
								{#if d.status === 'downloading' && (d.speed_bytes_sec || 0) > 0}
									<span class="text-[var(--primary)] font-semibold">
										{formatSpeed(d.speed_bytes_sec || 0)}
									</span>
								{/if}
							</div>
						</div>
					{:else if d.status === 'completed'}
						<div class="flex items-center gap-2 text-xs text-[var(--success-text)]">
							<CircleCheck size={13} />
							<span
								>{t('models.downloads.installedReady', { size: formatBytes(d.total_bytes) })}</span
							>
						</div>
					{:else if d.status === 'failed'}
						<div class="flex items-start gap-2 text-xs text-[var(--danger-text)]">
							<CircleAlert size={14} class="shrink-0 mt-0.5" />
							<span>{d.error_message || t('common.error')}</span>
						</div>
					{/if}
				</div>
			{/each}
		</div>
	{/if}
</div>
