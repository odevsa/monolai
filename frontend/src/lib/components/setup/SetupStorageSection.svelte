<script lang="ts">
	import { Badge, Input } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import type { ConfigStatus } from '$lib/types/config';
	import { CircleCheck, Folder, HardDrive } from '@lucide/svelte';

	let {
		configStatus,
		isStorageConfigured,
		modelsDir = $bindable(''),
		runtimesDir = $bindable(''),
		onResetModels,
		onResetRuntimes
	}: {
		configStatus: ConfigStatus;
		isStorageConfigured: boolean;
		modelsDir: string;
		runtimesDir: string;
		onResetModels: () => void;
		onResetRuntimes: () => void;
	} = $props();
</script>

{#if !isStorageConfigured}
	<!-- Step 1: Models Directory -->
	<div
		class="flex flex-col gap-2 p-4 rounded-xl border border-[var(--border-color)] bg-[var(--bg-surface-hover)]/30"
	>
		<div class="flex items-center justify-between">
			<label
				for="models-dir-input"
				class="flex items-center gap-2 text-xs font-bold uppercase tracking-wider text-[var(--text-secondary)]"
			>
				<HardDrive size={15} class="text-[var(--primary)]" />
				<span>{t('setup.stepModels')}</span>
			</label>
			<button
				type="button"
				onclick={onResetModels}
				class="text-[11px] text-[var(--text-muted)] hover:text-[var(--primary)] transition cursor-pointer bg-transparent border-0 p-0"
			>
				{t('common.resetToDefault')}
			</button>
		</div>
		<Input
			id="models-dir-input"
			bind:value={modelsDir}
			placeholder={t('setup.modelsDirPlaceholder')}
			class="font-mono text-xs"
		/>
	</div>

	<!-- Step 2: Runtimes Directory -->
	<div
		class="flex flex-col gap-2 p-4 rounded-xl border border-[var(--border-color)] bg-[var(--bg-surface-hover)]/30"
	>
		<div class="flex items-center justify-between">
			<label
				for="runtimes-dir-input"
				class="flex items-center gap-2 text-xs font-bold uppercase tracking-wider text-[var(--text-secondary)]"
			>
				<Folder size={15} class="text-[var(--primary)]" />
				<span>{t('setup.stepRuntimes')}</span>
			</label>
			<button
				type="button"
				onclick={onResetRuntimes}
				class="text-[11px] text-[var(--text-muted)] hover:text-[var(--primary)] transition cursor-pointer bg-transparent border-0 p-0"
			>
				{t('common.resetToDefault')}
			</button>
		</div>
		<Input
			id="runtimes-dir-input"
			bind:value={runtimesDir}
			placeholder={t('setup.runtimesDirPlaceholder')}
			class="font-mono text-xs"
		/>
	</div>
{:else}
	<!-- Storage Volumes Detected Card -->
	<div
		class="flex items-center gap-3 p-3.5 rounded-xl border border-[var(--border-color)] bg-[var(--bg-surface-hover)]/30 text-xs"
	>
		<div
			class="w-8 h-8 rounded-lg bg-[var(--primary-light)] text-[var(--primary)] border border-[var(--primary-focus)] flex items-center justify-center shrink-0"
		>
			<CircleCheck size={16} />
		</div>
		<div class="flex flex-col gap-0.5 min-w-0">
			<div class="flex items-center gap-2">
				<span class="font-semibold text-[var(--text-primary)]">
					{configStatus.is_docker ? t('setup.dockerVolumes') : t('setup.storageVolumesConfigured')}
				</span>
				{#if configStatus.is_docker}
					<Badge variant="info" rounded="rounded-sm">
						{t('setup.fixedMounts')}
					</Badge>
				{/if}
			</div>
			<ul class="text-[11px] text-[var(--text-muted)] font-mono truncate list-none p-0 m-0">
				<li>{t('setup.modelsPrefix')} {configStatus.models_dir || '/app/models'}</li>
				<li>{t('setup.runtimesPrefix')} {configStatus.runtimes_dir || '/app/runtimes'}</li>
			</ul>
		</div>
	</div>
{/if}
