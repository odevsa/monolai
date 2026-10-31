<script lang="ts">
	import { hubApi } from '$lib/api/hub';
	import { modelsApi } from '$lib/api/models';
	import {
		Alert,
		Badge,
		Button,
		Card,
		EmptyState,
		MarkdownRenderer,
		Modal,
		Spinner,
		Tabs
	} from '$lib/components/ui';
	import { askConfirm } from '$lib/confirmStore';
	import { t } from '$lib/i18n';
	import { downloadStore } from '$lib/state/downloads.svelte';
	import { hostState } from '$lib/state/host.svelte';
	import type { HubModelSibling, ModelRecord } from '$lib/types/models';
	import { formatBytes } from '$lib/utils/format';
	import { resolveMarkdownRelativeUrls, stripMarkdownFrontmatter } from '$lib/utils/markdown';
	import {
		ArrowDownToLine,
		Box,
		Download,
		ExternalLink,
		FileText,
		Layers,
		Trash
	} from '@lucide/svelte';
	import { untrack } from 'svelte';

	let {
		open = $bindable(false),
		modelId,
		title,
		repoId = 'huggingface',
		installedModels = [],
		onclose,
		onModelUpdated
	}: {
		open: boolean;
		modelId: string;
		title?: string;
		repoId?: string;
		installedModels?: ModelRecord[];
		onclose?: () => void;
		onModelUpdated?: () => void;
	} = $props();

	type ModalTabType = 'readme' | 'files';
	let activeModalTab = $state<ModalTabType>('readme');
	let readmeContent = $state<string | null>(null);
	let isLoadingReadme = $state(false);
	let readmeError = $state<string | null>(null);

	let modelFiles = $state<HubModelSibling[]>([]);
	let isLoadingDetails = $state(false);
	let detailsError = $state<string | null>(null);
	let enqueuedFiles = $state<Record<string, boolean>>({});
	let deletingModelId = $state<string | null>(null);

	// Normalize model ID: remove colon preset if present (e.g., "TheBloke/Llama-2:default" -> "TheBloke/Llama-2")
	let cleanRepoId = $derived.by(() => {
		if (!modelId) return '';
		return modelId.includes(':') ? modelId.split(':')[0] : modelId;
	});

	let displayTitle = $derived(title || cleanRepoId || modelId);

	let modalTabs = $derived.by(() => {
		const tabs: Array<{ id: ModalTabType; label: string; icon?: any; badge?: number | string }> = [
			{ id: 'readme', label: t('models.discover.tabReadme'), icon: FileText }
		];
		if (modelFiles.length > 0) {
			tabs.push({
				id: 'files',
				label: t('models.discover.tabFiles'),
				icon: Layers,
				badge: modelFiles.length
			});
		}
		return tabs;
	});

	// If active tab is files and no files available, stay on or fallback to readme
	$effect(() => {
		if (activeModalTab === 'files' && modelFiles.length === 0 && !isLoadingDetails) {
			activeModalTab = 'readme';
		}
	});

	let loadedKey = $state<string | null>(null);

	// Trigger loading when modal opens or modelId changes
	$effect(() => {
		if (open && cleanRepoId) {
			const key = `${repoId}:${cleanRepoId}`;
			if (loadedKey !== key) {
				loadedKey = key;
				untrack(() => {
					activeModalTab = 'readme';
					readmeContent = null;
					readmeError = null;
					modelFiles = [];
					detailsError = null;
					loadReadme();
					loadDetails();
				});
			}
		} else if (!open) {
			loadedKey = null;
		}
	});

	function cleanMarkdown(content: string): string {
		const stripped = stripMarkdownFrontmatter(content);
		const rawBaseUrl = `https://huggingface.co/${cleanRepoId}/resolve/main`;
		return resolveMarkdownRelativeUrls(stripped, rawBaseUrl);
	}

	async function loadReadme() {
		if (!cleanRepoId || isLoadingReadme) return;
		isLoadingReadme = true;
		readmeError = null;
		try {
			const res = await hubApi.getModelReadme(repoId, cleanRepoId);
			readmeContent = cleanMarkdown(res.content);
		} catch (err: unknown) {
			const msg = err instanceof Error ? err.message : String(err);
			readmeError = msg.includes('404') ? t('models.discover.noReadmeDesc') : msg;
		} finally {
			isLoadingReadme = false;
		}
	}

	async function loadDetails() {
		if (!cleanRepoId || isLoadingDetails) return;
		isLoadingDetails = true;
		detailsError = null;
		try {
			const details = await hubApi.getModelDetails(repoId, cleanRepoId);
			modelFiles = details.siblings || [];
		} catch (err: unknown) {
			const msg = err instanceof Error ? err.message : String(err);
			detailsError = msg;
		} finally {
			isLoadingDetails = false;
		}
	}

	function handleModalTabChange(tabId: string) {
		activeModalTab = tabId as ModalTabType;
		if (tabId === 'readme' && readmeContent === null && !isLoadingReadme) {
			loadReadme();
		} else if (tabId === 'files' && modelFiles.length === 0 && !isLoadingDetails) {
			loadDetails();
		}
	}

	function getPerformanceFit(fileSizeBytes?: number) {
		if (!fileSizeBytes || fileSizeBytes <= 0) {
			return {
				color: 'bg-[var(--text-muted)]',
				title: t('models.discover.fitUnknown')
			};
		}

		const info = hostState.sysinfo;
		if (!info) {
			return {
				color: 'bg-[var(--text-muted)]',
				title: t('models.discover.fitUnknown')
			};
		}

		const requiredBytes = fileSizeBytes * 1.15;
		const vramTotal =
			info.vram?.total_bytes ??
			(info.gpus?.reduce((acc, g) => acc + (g.memory_total_bytes ?? 0), 0) ||
				info.gpu?.memory_total_bytes ||
				0);
		const ramTotal = info.ram?.total_bytes ?? 0;

		if (vramTotal > 0 && requiredBytes <= vramTotal) {
			return {
				color: 'bg-[var(--success)]',
				title: t('models.discover.fitOptimal')
			};
		}

		if (ramTotal > 0 && requiredBytes <= ramTotal * 0.85) {
			return {
				color: 'bg-[var(--warning)]',
				title: vramTotal > 0 ? t('models.discover.fitPartialGpu') : t('models.discover.fitModerate')
			};
		}

		return {
			color: 'bg-[var(--danger)]',
			title: t('models.discover.fitLow')
		};
	}

	function findDownloadedModelForFile(file: HubModelSibling): ModelRecord | undefined {
		const filename = file.rfilename.split('/').pop() || file.rfilename;
		return installedModels.find((m) => {
			if (m.filename === filename) return true;
			if (m.relative_path && m.relative_path.endsWith(filename)) return true;
			return false;
		});
	}

	async function handleDownload(file: HubModelSibling) {
		const filename = file.rfilename.split('/').pop() || file.rfilename;
		enqueuedFiles[file.rfilename] = true;

		const resolvedUrl =
			file.download_url || `https://huggingface.co/${cleanRepoId}/resolve/main/${file.rfilename}`;

		try {
			await downloadStore.enqueue({
				url: resolvedUrl,
				download_url: resolvedUrl,
				model_name: cleanRepoId,
				filename,
				repo_id: repoId,
				modality: 'text-to-text',
				size_bytes: file.size || 0
			});
		} catch (err: unknown) {
			console.error('Failed to enqueue download:', err);
			enqueuedFiles[file.rfilename] = false;
		}
	}

	async function handleDeleteDownloaded(record: ModelRecord) {
		const confirmed = await askConfirm(
			t('models.deleteConfirmMsg', { id: record.id }),
			t('models.deleteConfirmTitle'),
			'danger',
			t('common.delete'),
			t('common.cancel')
		);
		if (!confirmed) return;

		deletingModelId = record.id;
		try {
			await modelsApi.delete(record.id, true);
			onModelUpdated?.();
		} catch (err: unknown) {
			console.error('Failed to delete downloaded model:', err);
		} finally {
			deletingModelId = null;
		}
	}

	function handleClose() {
		open = false;
		loadedKey = null;
		readmeContent = null;
		readmeError = null;
		modelFiles = [];
		detailsError = null;
		onclose?.();
	}
</script>

{#if open}
	<Modal icon={Box} bind:open title={displayTitle} maxWidth="2xl" onclose={handleClose}>
		{#snippet actions()}
			{#if cleanRepoId && cleanRepoId.includes('/')}
				<a
					href={`https://huggingface.co/${cleanRepoId}`}
					target="_blank"
					rel="noopener noreferrer"
					class="inline-flex items-center gap-1.5 px-2.5 py-1 text-xs font-medium text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)] rounded-lg transition-colors border border-[var(--border-color)]/60"
					title={t('models.discover.openInHuggingFace')}
				>
					<ExternalLink size={13} />
					<span class="hidden sm:inline">{t('models.discover.openInHuggingFace')}</span>
				</a>
			{/if}
		{/snippet}

		<Tabs
			variant="card"
			items={modalTabs}
			bind:value={activeModalTab}
			onchange={handleModalTabChange}
			cardClass="p-4 border border-[var(--border-color)]/50 bg-[var(--bg-surface-hover)]/20 rounded-b-2xl"
		>
			{#if activeModalTab === 'readme'}
				{#if isLoadingReadme}
					<div
						class="flex flex-col items-center justify-center p-12 text-[var(--text-muted)] gap-2 min-h-[220px]"
					>
						<Spinner size={24} />
						<span class="text-xs">{t('common.loading')}</span>
					</div>
				{:else if !readmeContent || !readmeContent.trim() || readmeError}
					<div class="py-8">
						<EmptyState
							icon={FileText}
							size="md"
							title={t('models.discover.noReadmeTitle')}
							description={t('models.discover.noReadmeDesc')}
						/>
					</div>
				{:else}
					<div class="max-h-[60vh] overflow-y-auto pr-3 pl-1 py-1">
						<MarkdownRenderer content={readmeContent} />
					</div>
				{/if}
			{:else if activeModalTab === 'files'}
				<div class="space-y-4">
					<div class="text-xs text-[var(--text-muted)]">
						{t('models.discover.selectQuantPrompt')}
					</div>

					{#if isLoadingDetails}
						<div
							class="flex flex-col items-center justify-center p-10 text-[var(--text-muted)] gap-2 min-h-[200px]"
						>
							<Spinner size={24} />
							<span class="text-xs">{t('common.loading')}</span>
						</div>
					{:else if modelFiles.length === 0}
						<div class="py-8">
							<EmptyState icon={Box} size="sm" title={t('models.discover.noFilesAvailable')} />
						</div>
					{:else}
						<div class="space-y-2 max-h-[60vh] overflow-y-auto pr-1">
							{#each modelFiles as file}
								{@const fileSize = file.size || 0}
								{@const fit = getPerformanceFit(fileSize)}
								{@const downloadedModel = findDownloadedModelForFile(file)}
								{@const isDownloading = enqueuedFiles[file.rfilename]}
								{@const isDeleting = downloadedModel && deletingModelId === downloadedModel.id}

								<Card
									padding="sm"
									class="transition-all {downloadedModel
										? 'border border-[var(--primary)] bg-[var(--primary-light)] shadow-xs'
										: 'border border-[var(--border-color)]/70'}"
								>
									<div class="flex items-center justify-between gap-3 text-xs">
										<div class="min-w-0 flex-1">
											<div class="flex items-center gap-2 flex-wrap">
												<span
													class="w-2.5 h-2.5 rounded-full shrink-0 shadow-xs {fit.color}"
													title={fit.title}
												></span>
												<span
													class="font-mono text-[var(--text-primary)] font-medium truncate block"
												>
													{file.rfilename.split('/').pop()}
												</span>
												{#if file.quant}
													<Badge variant="info" class="font-mono text-[10px]">
														{file.quant}
													</Badge>
												{/if}
											</div>
											<div class="flex items-center gap-2 mt-1">
												{#if fileSize > 0}
													<span class="text-[11px] text-[var(--text-muted)] font-mono">
														{formatBytes(fileSize)}
													</span>
													<span class="text-[10px] text-[var(--border-color)]">•</span>
												{/if}
												<span
													class="text-[11px] text-[var(--text-muted)] cursor-help truncate"
													title={fit.title}
												>
													{fit.title}
												</span>
											</div>
										</div>

										<div class="shrink-0 flex items-center gap-2">
											{#if isDownloading}
												<Button variant="success" size="sm" disabled>
													<ArrowDownToLine
														size={13}
														class="animate-pulse text-[var(--success-text)]"
													/>
													<span>{t('common.downloading')}</span>
												</Button>
											{:else if downloadedModel}
												<Button
													variant="danger"
													size="sm"
													loading={isDeleting}
													onclick={() => handleDeleteDownloaded(downloadedModel)}
													title={t('common.delete')}
												>
													<Trash size={13} />
													<span>{t('common.delete')}</span>
												</Button>
											{:else}
												<Button
													variant="primary"
													size="sm"
													onclick={() => handleDownload(file)}
													title={t('common.download')}
												>
													<Download size={13} />
													<span>{t('common.download')}</span>
												</Button>
											{/if}
										</div>
									</div>
								</Card>
							{/each}
						</div>
					{/if}
				</div>
			{/if}
		</Tabs>
	</Modal>
{/if}
