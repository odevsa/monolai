<script lang="ts">
	import { Badge, Button, ModelBadge } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import type { ModelFileItem, ModelRecord, RunningModelStatus } from '$lib/types/models';
	import { Check, Copy, Pencil, Trash, TriangleAlert } from '@lucide/svelte';
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

	function getMainFile(flagsStr: string): string {
		try {
			const flags = JSON.parse(flagsStr);
			return flags.m || flags['--model'] || flags.model || '';
		} catch {
			return '';
		}
	}

	let mainFile = $derived(getMainFile(model.flags));
	let isMissing = $derived(model.file_exists === false);
	let isRunning = $derived(!!runningInfo);
	let isCopied = $derived(copiedId === model.id);
</script>

<div
	class="bg-[var(--bg-surface)] rounded-xl p-4 flex flex-col justify-between gap-3 shadow-xs transition-opacity {isMissing
		? 'opacity-65 hover:opacity-85'
		: ''}"
>
	<!-- Header: Model ID + Badges + Status -->
	<div class="flex flex-col gap-2 min-w-0">
		<ModelBadge model={model.id} variant="badge-full" />

		<div class="flex items-center justify-between gap-1.5 min-w-0">
			<!-- Model ID with copy shortcut -->
			<button
				type="button"
				class="flex items-center gap-1.5 text-xs text-[var(--text-muted)] hover:text-[var(--text-primary)] transition-colors p-1 truncate text-left rounded-sm group cursor-pointer bg-transparent border-0 min-w-0"
				onclick={() => onCopyId?.(model.id)}
				title={isCopied ? t('models.copyIdSuccess') : t('models.copyId')}
			>
				<span class="truncate">{model.id}</span>
				{#if isCopied}
					<Check size={12} class="text-[var(--success)] shrink-0" />
				{:else}
					<Copy
						size={11}
						class="opacity-0 group-hover:opacity-100 transition-opacity shrink-0 text-[var(--text-muted)]"
					/>
				{/if}
			</button>

			<div class="flex items-center gap-1.5 shrink-0">
				<Badge variant="muted" rounded="rounded-md">{model.runtime}</Badge>

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
	</div>

	<!-- File information -->
	<ModelFileInfo filePath={mainFile} {availableFiles} {isMissing} />

	<!-- Action Buttons -->
	<div class="flex items-center justify-end gap-2 pt-1">
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
