<script lang="ts">
	import { Badge, Button, Card } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import type { ModelRecord } from '$lib/types/models';
	import { copyToClipboard } from '$lib/utils/clipboard';
	import { formatBytes } from '$lib/utils/format';
	import { Check, Copy, File, FileBox, Trash } from '@lucide/svelte';
	import Divider from '../ui/atoms/Divider.svelte';

	let {
		model,
		onDelete
	}: {
		model: ModelRecord;
		onDelete: (id: string, filename: string) => void;
	} = $props();

	let isCopied = $state(false);

	async function handleCopy() {
		const path = model.relative_path || model.filename;
		await copyToClipboard(path);
		isCopied = true;
		setTimeout(() => {
			isCopied = false;
		}, 2000);
	}

	let typeLabel = $derived(model.auxiliary_type ? model.auxiliary_type.toUpperCase() : 'AUXILIARY');
</script>

<Card variant="surface" padding="md" class="flex flex-col justify-between">
	<!-- Header / Identification -->
	<div class="flex flex-col gap-1.5 min-w-0 flex-1">
		<div class="flex justify-start items-center gap-1">
			<FileBox size={18} class="text-[var(--warning)] shrink-0" />

			<h4
				class="mr-auto text-sm font-semibold text-[var(--text-primary)] truncate"
				title={model.name || model.id}
			>
				{model.name || model.id}
			</h4>
		</div>

		<div class="flex items-center gap-1 text-[12px] text-[var(--text-secondary)]">
			<File size={12} />
			<span title={model.filename} class="truncate">{model.filename}</span>
		</div>

		<!-- Auxiliary Type & Format Badges -->
		<div class="flex items-center gap-1 shrink-0">
			<Badge variant="warning" rounded="rounded-md" size="xs">
				{typeLabel}
			</Badge>
			{#if model.format}
				<Badge variant="muted" rounded="rounded-md" size="xs">
					{model.format.toUpperCase()}
				</Badge>
			{/if}
		</div>

		<Divider />

		<!-- Metadata & Actions Footer -->
		<div class="flex items-center justify-between text-xs text-[var(--text-secondary)]">
			<div class="flex items-center gap-2">
				<span class="font-mono text-[11px] text-[var(--text-secondary)]">
					{formatBytes(model.size_bytes)}
				</span>
			</div>

			<div class="flex items-center gap-1">
				<!-- Copy Path Action -->
				<Button
					variant="secondary"
					size="icon-xs"
					onclick={handleCopy}
					title={isCopied ? t('models.auxiliary.pathCopied') : t('models.auxiliary.copyPath')}
					ariaLabel={t('models.auxiliary.copyPath')}
				>
					{#if isCopied}
						<Check size={12} class="text-[var(--success-text)]" />
					{:else}
						<Copy size={12} />
					{/if}
				</Button>

				<!-- Delete Action -->
				<Button
					variant="ghost"
					size="icon-xs"
					class="text-[var(--text-muted)] hover:text-[var(--danger-text)]"
					onclick={() => onDelete(model.id, model.filename || model.name)}
					title={t('common.delete')}
					ariaLabel={t('common.delete')}
				>
					<Trash size={12} />
				</Button>
			</div>
		</div>
	</div>
</Card>
