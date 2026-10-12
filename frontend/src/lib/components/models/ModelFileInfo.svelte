<script lang="ts">
	import { Badge } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import type { ModelFileItem } from '$lib/types/models';
	import { formatBytes } from '$lib/utils/format';
	import { FileText } from '@lucide/svelte';

	let {
		filePath = '',
		availableFiles = [],
		isMissing = false
	}: {
		filePath?: string;
		availableFiles?: ModelFileItem[];
		isMissing?: boolean;
	} = $props();

	let fileDetails = $derived.by(() => {
		if (!filePath) {
			return {
				fileName: t('models.noFileConfigured'),
				sizeFormatted: null,
				format: 'UNKNOWN'
			};
		}

		const fileName = filePath.split('/').pop() || filePath;
		const found = availableFiles.find(
			(f) =>
				(f.name && f.name === fileName) ||
				(f.path && f.path.endsWith(fileName)) ||
				(f.relative_path && f.relative_path.endsWith(fileName))
		);

		const size = found?.size_bytes ?? found?.size;
		const sizeFormatted = size ? formatBytes(size) : null;

		let format = 'MODEL';
		const lower = fileName.toLowerCase();
		if (lower.endsWith('.gguf')) format = 'GGUF';
		else if (lower.endsWith('.safetensors')) format = 'SAFETENSORS';
		else if (lower.endsWith('.bin')) format = 'BIN';

		return { fileName, sizeFormatted, format };
	});
</script>

<div
	class="flex items-center justify-between gap-2 text-xs py-2 px-2.5 rounded-lg bg-[var(--bg-hover)]"
>
	<div
		class="flex items-center gap-1.5 min-w-0 font-mono {isMissing
			? 'text-[var(--warning-text)] opacity-80 line-through'
			: 'text-[var(--text-secondary)]'}"
	>
		<FileText size={13} class="shrink-0 text-[var(--text-muted)]" />
		<span class="truncate text-[0.72rem]" title={filePath || fileDetails.fileName}>
			{fileDetails.fileName}
		</span>
	</div>

	<div class="flex items-center gap-1.5 shrink-0">
		{#if fileDetails.sizeFormatted}
			<span class="text-[0.68rem] text-[var(--text-muted)] font-mono">
				{fileDetails.sizeFormatted}
			</span>
		{/if}
		<Badge variant="muted" rounded="rounded-sm" class="text-[0.65rem] px-1.5 py-0.5 font-semibold">
			{fileDetails.format}
		</Badge>
	</div>
</div>
