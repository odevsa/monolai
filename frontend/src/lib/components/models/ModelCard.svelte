<script lang="ts">
	import { modelsApi } from '$lib/api/models';
	import { Badge, Button, Card } from '$lib/components/ui';
	import { askConfirm } from '$lib/confirmStore';
	import { t } from '$lib/i18n';
	import type { ModelPreset, ModelRecord, RunningModelStatus } from '$lib/types/models';
	import { formatBytes } from '$lib/utils/format';
	import {
		Box,
		Check,
		Copy,
		Layers,
		Pencil,
		Plus,
		Star,
		Trash,
		TriangleAlert
	} from '@lucide/svelte';
	import ModelPresetModal from './ModelPresetModal.svelte';

	let {
		model,
		runningInfo,
		copiedId = null,
		onCopyId,
		onModelUpdated,
		onDelete
	}: {
		model: ModelRecord;
		runningInfo?: RunningModelStatus;
		copiedId?: string | null;
		onCopyId?: (id: string) => void;
		onModelUpdated: () => void;
		onDelete: (id: string, deleteFile?: boolean) => void;
	} = $props();

	let isMissing = $derived(model.file_exists === false);
	let isRunning = $derived(!!runningInfo);
	let isCopied = $derived(copiedId === model.id);
	let displayPath = $derived(model.relative_path || model.filename || model.id);

	// Presets management modal
	let showPresetModal = $state(false);
	let editingPreset = $state<ModelPreset | null>(null);

	function openAddPreset() {
		editingPreset = null;
		showPresetModal = true;
	}

	function openEditPreset(p: ModelPreset) {
		editingPreset = p;
		showPresetModal = true;
	}

	async function deletePreset(preset: ModelPreset) {
		const confirmed = await askConfirm(
			t('models.presets.deleteConfirm', { name: preset.name }),
			t('common.delete'),
			'danger',
			t('common.delete'),
			t('common.cancel')
		);
		if (!confirmed) return;

		try {
			await modelsApi.deletePreset(preset.id);
			onModelUpdated();
		} catch (err: unknown) {
			console.error('Failed to delete preset:', err);
		}
	}

	function getModalityVariant(
		modality: string
	): 'primary' | 'info' | 'success' | 'warning' | 'muted' {
		switch (modality) {
			case 'text-to-text':
				return 'primary';
			case 'text-to-image':
				return 'info';
			case 'text-to-speech':
			case 'speech-to-text':
				return 'warning';
			default:
				return 'muted';
		}
	}
</script>

<Card class="flex flex-col justify-between gap-3.5 h-full" padding="md">
	<!-- Top: Avatar + Title + Badges + Real-time Status -->
	<div class="flex items-start justify-between gap-3 min-w-0">
		<div class="flex items-start gap-2.5 min-w-0 flex-1">
			<Box size={22} class="text-[var(--primary)] shrink-0 mt-0.5" />

			<div class="min-w-0 flex-1">
				<div class="flex items-center gap-1.5 flex-wrap">
					<h4
						class="m-0 text-sm font-semibold text-[var(--text-primary)] truncate"
						title={model.name || model.id}
					>
						{model.name || model.id}
					</h4>

					<Button
						variant="ghost"
						size="icon-xs"
						onclick={() => onCopyId?.(model.id)}
						title={isCopied ? t('models.copyIdSuccess') : t('models.copyId')}
						ariaLabel={isCopied ? t('models.copyIdSuccess') : t('models.copyId')}
					>
						{#if isCopied}
							<Check size={12} class="text-[var(--success-text)]" />
						{:else}
							<Copy size={12} />
						{/if}
					</Button>

					<!-- Modality Badge -->
					<Badge variant={getModalityVariant(model.modality)}>
						{model.modality}
					</Badge>

					<!-- Format & Architecture -->
					{#if model.format}
						<Badge variant="muted" class="uppercase font-mono text-[10px]">
							{model.format}
						</Badge>
					{/if}
					{#if model.architecture}
						<Badge variant="muted" class="font-mono text-[10px]">
							{model.architecture}
						</Badge>
					{/if}
				</div>

				<div class="flex items-center gap-2 mt-1 text-[11px] text-[var(--text-muted)]">
					<span class="font-mono truncate">{displayPath}</span>
					{#if model.size_bytes > 0}
						<span class="shrink-0">• {formatBytes(model.size_bytes)}</span>
					{/if}
				</div>
			</div>
		</div>

		<!-- Status Badge -->
		<div class="flex items-center gap-1.5 shrink-0">
			{#if isRunning}
				<Badge variant="success" pulse={true}>
					{t('models.runningPort', { port: runningInfo?.port ?? 0 })}
				</Badge>
			{:else if isMissing}
				<Badge variant="warning">
					<TriangleAlert size={11} class="shrink-0" />
					<span>{t('models.fileMissing')}</span>
				</Badge>
			{/if}
		</div>
	</div>

	<!-- Presets Section -->
	<div
		class="bg-[var(--bg-primary)] border border-[var(--border-color)]/70 rounded-lg p-3 space-y-2"
	>
		<div class="flex items-center justify-between">
			<div class="flex items-center gap-1.5 text-xs font-semibold text-[var(--text-secondary)]">
				<Layers size={13} class="text-[var(--primary)]" />
				<span>{t('models.presets.title')} ({model.presets?.length || 0})</span>
			</div>

			<Button variant="ghost" size="xs" onclick={openAddPreset}>
				<Plus size={12} />
				<span>{t('models.presets.addPreset')}</span>
			</Button>
		</div>

		{#if !model.presets || model.presets.length === 0}
			<p class="text-[11px] text-[var(--text-muted)] italic m-0">
				{t('models.presets.noPresets')}
			</p>
		{:else}
			<div class="space-y-1.5">
				{#each model.presets as preset (preset.id)}
					<div
						class="flex items-center justify-between gap-2 p-1.5 bg-[var(--bg-surface)] border border-[var(--border-color)]/50 rounded-md text-xs"
					>
						<div class="flex items-center gap-1.5 min-w-0">
							{#if preset.is_default}
								<Star size={11} class="text-[var(--warning)] shrink-0 fill-[var(--warning)]" />
							{/if}
							<span class="font-medium text-[var(--text-primary)] truncate">
								{preset.name}
							</span>
							<Badge variant="muted" class="font-mono text-[10px]">
								{preset.runtime_id}
							</Badge>
						</div>

						<div class="flex items-center gap-1 shrink-0">
							<Button
								variant="ghost"
								size="icon-xs"
								onclick={() => openEditPreset(preset)}
								title={t('common.edit')}
								ariaLabel={t('common.edit')}
							>
								<Pencil size={11} />
							</Button>
							{#if model.presets.length > 1}
								<Button
									variant="ghost"
									size="icon-xs"
									onclick={() => deletePreset(preset)}
									title={t('common.delete')}
									ariaLabel={t('common.delete')}
								>
									<Trash size={11} class="text-[var(--danger-text)]" />
								</Button>
							{/if}
						</div>
					</div>
				{/each}
			</div>
		{/if}
	</div>

	<!-- Card Footer -->
	<div class="flex items-center justify-end gap-2 pt-2 border-t border-[var(--border-color)]/50">
		<Button
			variant="danger"
			size="sm"
			onclick={() => onDelete(model.id)}
			title={t('common.delete')}
		>
			<Trash size={13} />
			<span>{t('common.delete')}</span>
		</Button>
	</div>
</Card>

{#if showPresetModal}
	<ModelPresetModal
		open={showPresetModal}
		{model}
		preset={editingPreset}
		onclose={() => (showPresetModal = false)}
		onSaved={() => {
			showPresetModal = false;
			onModelUpdated();
		}}
	/>
{/if}
