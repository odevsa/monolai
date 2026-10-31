<script lang="ts">
	import { Button, CodeBlock } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import type { ConfigStatus, HardwareReport } from '$lib/types/config';
	import { copyToClipboard } from '$lib/utils/clipboard';
	import { Check, Copy, RefreshCw, Terminal } from '@lucide/svelte';
	import { getHardwareName } from './types';

	let {
		configStatus,
		hardwareReport = null,
		rechecking = false,
		onRecheck
	}: {
		configStatus: ConfigStatus;
		hardwareReport?: HardwareReport | null;
		rechecking?: boolean;
		onRecheck: () => void;
	} = $props();

	let copiedPath = $state(false);
	let copiedCli = $state(false);

	async function handleCopy(text: string, kind: 'path' | 'cli') {
		const ok = await copyToClipboard(text);
		if (ok) {
			if (kind === 'path') {
				copiedPath = true;
				setTimeout(() => (copiedPath = false), 2000);
			} else {
				copiedCli = true;
				setTimeout(() => (copiedCli = false), 2000);
			}
		}
	}
</script>

<div class="flex flex-col gap-4 text-xs text-[var(--text-secondary)]">
	<!-- 1. Default File Path -->
	<div
		class="p-4 rounded-xl border border-[var(--border-color)] bg-[var(--bg-surface-hover)]/30 flex flex-col gap-2"
	>
		<span class="font-semibold text-[var(--text-primary)]">
			{t('setup.defaultConfigLocation')}
		</span>
		<div
			class="flex items-center justify-between p-2.5 rounded-lg bg-[var(--bg-primary)] border border-[var(--border-color)] font-mono text-xs"
		>
			<span class="text-[var(--primary)] break-all">{configStatus.expected_path}</span>
			<Button
				variant="secondary"
				size="sm"
				onclick={() => handleCopy(configStatus.expected_path, 'path')}
				class="shrink-0 ml-2"
				title={t('common.copy')}
			>
				{#if copiedPath}
					<Check size={12} class="text-[var(--success-text)]" />
					<span>{t('setup.copied')}</span>
				{:else}
					<Copy size={12} />
					<span>{t('common.copy')}</span>
				{/if}
			</Button>
		</div>
	</div>

	<!-- 2. CLI Custom Path Argument -->
	{#if configStatus.cli_command_example}
		<div
			class="p-4 rounded-xl border border-[var(--border-color)] bg-[var(--bg-surface-hover)]/30 flex flex-col gap-2"
		>
			<div class="flex items-center gap-2 font-semibold text-[var(--text-primary)]">
				<Terminal size={14} class="text-[var(--primary)]" />
				<span>{t('setup.customConfigCli')}</span>
			</div>
			<div
				class="flex items-center justify-between p-2.5 rounded-lg bg-[var(--bg-primary)] border border-[var(--border-color)] font-mono text-xs"
			>
				<span class="text-[var(--text-primary)] break-all">
					{configStatus.cli_command_example}
				</span>
				<Button
					variant="secondary"
					size="sm"
					onclick={() => handleCopy(configStatus.cli_command_example || '', 'cli')}
					class="shrink-0 ml-2"
					title={t('common.copy')}
				>
					{#if copiedCli}
						<Check size={12} class="text-[var(--success-text)]" />
						<span>{t('setup.copied')}</span>
					{:else}
						<Copy size={12} />
						<span>{t('common.copy')}</span>
					{/if}
				</Button>
			</div>
		</div>
	{/if}

	<!-- 3. Example YAML Content -->
	<div
		class="p-4 rounded-xl border border-[var(--border-color)] bg-[var(--bg-surface-hover)]/30 flex flex-col gap-2.5"
	>
		<div class="flex items-center justify-between">
			<span class="font-semibold text-[var(--text-primary)]">
				{t('setup.validYamlExample')}
			</span>
			{#if hardwareReport}
				<span class="text-[11px] text-[var(--text-muted)] flex items-center gap-1.5">
					<span>{t('setup.detectedHostLabel')}</span>
					<strong class="text-[var(--success-text)] font-semibold">
						{getHardwareName(hardwareReport.recommended_acceleration)}
					</strong>
				</span>
			{/if}
		</div>
		<CodeBlock code={configStatus.example_yaml} lang="yaml" />
	</div>

	<!-- Action: Recheck -->
	<div class="pt-1 flex justify-end">
		<Button
			variant="primary"
			size="md"
			loading={rechecking}
			disabled={rechecking}
			onclick={onRecheck}
			class="shadow-sm"
		>
			<RefreshCw size={13} />
			<span>{t('setup.recheckConfig')}</span>
		</Button>
	</div>
</div>
