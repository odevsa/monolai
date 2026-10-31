<script lang="ts">
	import { Alert, Badge, Button, Card, DiagonalLabel } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import type { Runtime } from '$lib/types/runtimes';
	import { Download, RefreshCw, Trash } from '@lucide/svelte';
	import RuntimeCardHeader from './RuntimeCardHeader.svelte';
	import RuntimeHardwareSelect from './RuntimeHardwareSelect.svelte';
	import RuntimeInstallProgress from './RuntimeInstallProgress.svelte';
	import RuntimePathHint from './RuntimePathHint.svelte';

	interface RuntimeCardProps {
		runtime: Runtime;
		selectedAcceleration: string;
		onAccelerationChange: (runtimeId: string, val: string) => void;
		onInstall: (runtimeId: string) => void;
		onUninstall: (runtime: Runtime) => void;
		onUpdate: (runtime: Runtime) => void;
	}

	let {
		runtime,
		selectedAcceleration,
		onAccelerationChange,
		onInstall,
		onUninstall,
		onUpdate
	}: RuntimeCardProps = $props();

	let progress = $derived(runtime.install_progress);
	let isInstalling = $derived.by(() => {
		if (!progress?.status) return false;
		const s = progress.status.toLowerCase();
		return s.startsWith('download') || s.startsWith('extract');
	});
</script>

<Card class="relative overflow-hidden">
	{#if runtime.is_installed}
		<DiagonalLabel text={runtime.has_update ? t('common.update') : t('common.installed')} />
	{/if}

	<div class="flex flex-col gap-4 justify-between h-full">
		<!-- Card Top Details -->
		<div class="flex flex-col gap-4 justify-between h-full">
			<!-- Header -->
			<RuntimeCardHeader {runtime} />

			<!-- Description & Feature Badges -->
			<div class="flex flex-col gap-2 mb-auto">
				<p class="m-0 text-xs leading-relaxed text-[var(--text-secondary)]">
					{t(`runtimes.description.${runtime.id}`)}
				</p>

				{#if runtime.features && runtime.features.length > 0}
					<div class="flex items-center gap-1.5 flex-wrap">
						{#each runtime.features as feat}
							<Badge variant="muted" rounded="rounded-md">{feat}</Badge>
						{/each}
					</div>
				{/if}
			</div>

			<!-- Installed Binary Path Hint -->
			{#if runtime.is_installed && runtime.installed_path}
				<RuntimePathHint
					installedPath={runtime.installed_path}
					acceleration={runtime.installed_acceleration}
				/>
			{/if}

			<!-- Hardware Acceleration Selector -->
			{#if !runtime.is_installed && !isInstalling && runtime.available_accelerations && runtime.available_accelerations.length > 0}
				<RuntimeHardwareSelect
					runtimeId={runtime.id}
					accelerations={runtime.available_accelerations}
					{selectedAcceleration}
					onSelect={(val) => onAccelerationChange(runtime.id, val)}
				/>
			{/if}

			<!-- Installation Error Notice if any -->
			{#if progress && progress.status === 'error'}
				<Alert variant="error" dismissible>
					{progress.error_message || t('runtimes.installationError')}
				</Alert>
			{/if}
		</div>

		<!-- Card Bottom Actions -->
		<div
			class="pt-4 border-t border-[var(--border-color)] flex items-center justify-end gap-2.5 flex-wrap"
		>
			{#if isInstalling && progress}
				<RuntimeInstallProgress {progress} />
			{:else if !runtime.is_installed}
				<Button variant="primary" size="md" class="w-full" onclick={() => onInstall(runtime.id)}>
					<Download size={14} />
					<span>{t('common.install')}</span>
				</Button>
			{:else}
				<Button
					variant="danger"
					size="sm"
					onclick={() => onUninstall(runtime)}
					title={t('runtimes.uninstallTooltip')}
					ariaLabel={t('runtimes.uninstallTooltip')}
				>
					<Trash size={13} />
					<span>{t('common.uninstall')}</span>
				</Button>

				{#if runtime.has_update}
					<Button
						variant="primary"
						size="sm"
						onclick={() => onUpdate(runtime)}
						title={t('runtimes.updateTooltip')}
						ariaLabel={t('runtimes.updateTooltip')}
					>
						<Download size={13} />
						<span>{t('common.update')}</span>
					</Button>
				{:else}
					<Button
						variant="secondary"
						size="sm"
						onclick={() => onInstall(runtime.id)}
						title={t('runtimes.reinstallTooltip')}
						ariaLabel={t('runtimes.reinstallTooltip')}
					>
						<RefreshCw size={13} />
						<span>{t('common.reinstall')}</span>
					</Button>
				{/if}
			{/if}
		</div>
	</div>
</Card>
