<script lang="ts">
	import { Badge, Button } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import type { ModelFileItem, ModelRecord, RunningModelStatus } from '$lib/types/models';
	import { getMainModelFilePath, parseModelDetails } from '$lib/utils/model';
	import { Box, Check, Copy, Cpu, Pencil, Trash, TriangleAlert } from '@lucide/svelte';
	import ModelFileInfo from './ModelFileInfo.svelte';

	let {
		model,
		availableFiles = [],
		runningInfo,
		copiedId = null,
		onCopyId,
		onEdit,
		onDelete
	}: {
		model: ModelRecord;
		availableFiles?: ModelFileItem[];
		runningInfo?: RunningModelStatus;
		copiedId?: string | null;
		onCopyId?: (id: string) => void;
		onEdit: (model: ModelRecord) => void;
		onDelete: (id: string) => void;
	} = $props();

	let mainFile = $derived(getMainModelFilePath(model.flags));
	let isMissing = $derived(model.file_exists === false);
	let isRunning = $derived(!!runningInfo);
	let isCopied = $derived(copiedId === model.id);
	let parsed = $derived(parseModelDetails(model.id));
	let displayName = $derived(parsed.name || model.id);
	let isIdDifferent = $derived(model.id !== displayName);
</script>

<div
	class="bg-[var(--bg-surface)] rounded-xl p-4 flex flex-col justify-between gap-3.5 shadow-xs transition-all"
>
	<!-- Header: Avatar + Title + Tags + Real-time Status -->
	<div class="flex items-start justify-between gap-3 min-w-0">
		<div class="flex items-start gap-2.5 min-w-0 flex-1">
			<Box size={22} class="text-[var(--text-muted)]" />

			<div class="min-w-0 flex-1">
				<div class="flex items-center gap-1.5 flex-wrap">
					<h4
						class="m-0 text-sm font-semibold text-[var(--text-primary)] truncate"
						title={displayName}
					>
						{displayName}
					</h4>

					{#if !isIdDifferent}
						<!-- Copy button directly beside title when ID equals displayName -->
						<button
							type="button"
							class="inline-flex items-center justify-center p-1 rounded-sm text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-primary)] transition-colors cursor-pointer bg-transparent border-0 shrink-0"
							onclick={() => onCopyId?.(model.id)}
							title={isCopied ? t('models.copyIdSuccess') : t('models.copyId')}
						>
							{#if isCopied}
								<Check size={12} class="text-[var(--success)]" />
							{:else}
								<Copy size={12} />
							{/if}
						</button>
					{/if}

					{#if parsed.tags.length > 0}
						<div class="flex items-center gap-1 flex-wrap">
							{#each parsed.tags as tag}
								<span
									class="px-1.5 py-0.2 rounded text-[0.62rem] font-medium bg-[var(--primary-light)] text-[var(--primary)] border border-[var(--primary)]/20"
								>
									{tag}
								</span>
							{/each}
						</div>
					{/if}
				</div>

				{#if isIdDifferent}
					<!-- Subtitle with full model ID and copy action when different from displayName -->
					<button
						type="button"
						class="inline-flex items-center gap-1.5 text-[0.72rem] font-mono text-[var(--text-muted)] hover:text-[var(--text-primary)] transition-colors mt-1 max-w-full truncate text-left cursor-pointer group bg-transparent border-0 p-0"
						onclick={() => onCopyId?.(model.id)}
						title={isCopied ? t('models.copyIdSuccess') : t('models.copyId')}
					>
						<span class="truncate">{model.id}</span>
						{#if isCopied}
							<Check size={11} class="text-[var(--success)] shrink-0" />
						{:else}
							<Copy
								size={11}
								class="opacity-0 group-hover:opacity-100 transition-opacity shrink-0 text-[var(--text-muted)]"
							/>
						{/if}
					</button>
				{/if}
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

	<!-- File information -->
	<ModelFileInfo filePath={mainFile} {availableFiles} {isMissing} />

	<!-- Footer: Runtime Backend on Left, Actions on Right -->
	<div
		class="flex items-center justify-between gap-2 pt-2 border-t border-[var(--border-color)]/50"
	>
		<div class="flex items-center gap-1.5 text-xs text-[var(--text-muted)]">
			<Cpu size={13} class="text-[var(--text-muted)] shrink-0" />
			<span class="font-mono text-[0.72rem] text-[var(--text-secondary)] font-medium">
				{model.runtime}
			</span>
		</div>

		<div class="flex items-center gap-2">
			<Button variant="secondary" size="sm" onclick={() => onEdit(model)} title={t('common.edit')}>
				<Pencil size={13} />
				<span>{t('common.edit')}</span>
			</Button>
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
	</div>
</div>
