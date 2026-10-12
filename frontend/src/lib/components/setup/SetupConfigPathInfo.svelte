<script lang="ts">
	import { Button } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import { copyToClipboard } from '$lib/utils/clipboard';
	import { Check, Copy, FileCode } from '@lucide/svelte';

	let {
		expectedPath
	}: {
		expectedPath: string;
	} = $props();

	let copied = $state(false);

	async function handleCopy() {
		const ok = await copyToClipboard(expectedPath);
		if (ok) {
			copied = true;
			setTimeout(() => (copied = false), 2000);
		}
	}
</script>

<div
	class="flex items-center justify-between p-3 rounded-xl border border-[var(--border-color)] bg-[var(--bg-surface-hover)]/30 text-xs"
>
	<div class="flex items-center gap-2 truncate text-[var(--text-muted)] min-w-0">
		<FileCode size={14} class="text-[var(--primary)] shrink-0" />
		<span class="text-[var(--text-secondary)] font-medium shrink-0">
			{t('setup.configWillBeWrittenTo')}
		</span>
		<span class="text-[var(--text-primary)] font-mono truncate">
			{expectedPath}
		</span>
	</div>

	<Button
		variant="secondary"
		size="sm"
		onclick={handleCopy}
		class="shrink-0 ml-2 text-[11px]"
		title={t('setup.copy')}
	>
		{#if copied}
			<Check size={12} class="text-[var(--success-text)]" />
			<span>{t('setup.copied')}</span>
		{:else}
			<Copy size={12} />
			<span>{t('setup.copy')}</span>
		{/if}
	</Button>
</div>
