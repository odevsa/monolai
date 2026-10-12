<script lang="ts">
	import { goto } from '$app/navigation';
	import { configApi, hostApi } from '$lib/api';
	import { SetupManualConfig, SetupWizard } from '$lib/components/setup';
	import { Alert, Button, PageHeader, Spinner, Tabs } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import type { ConfigStatus, HardwareReport } from '$lib/types/config';
	import { FileCode, RefreshCw, Sparkles, Wand } from '@lucide/svelte';
	import { getContext, onMount } from 'svelte';

	let activeTab = $state<string>('wizard');
	let configStatus = $state<ConfigStatus | null>(null);
	let hardwareReport = $state<HardwareReport | null>(null);
	let loading = $state(true);
	let saving = $state(false);
	let saveError = $state<string | null>(null);
	let rechecking = $state(false);

	// Wizard Form State
	let modelsDir = $state('');
	let runtimesDir = $state('');
	let host = $state('0.0.0.0');
	let port = $state(8080);

	let isStorageConfigured = $derived(
		!!configStatus?.is_docker || (!!configStatus?.has_models && !!configStatus?.has_runtimes)
	);

	const checkConfigFromLayout = getContext<() => Promise<void>>('checkConfigStatus');

	async function fetchConfigData() {
		try {
			rechecking = true;
			const [status, hw] = await Promise.all([
				configApi.getStatus().catch(() => null),
				hostApi.detectHardware().catch(() => null)
			]);

			if (status) {
				configStatus = status;

				if (status.is_valid) {
					goto('/chat');
					return;
				}

				if (!modelsDir && status.models_dir) {
					modelsDir = status.models_dir;
				}
				if (!runtimesDir && status.runtimes_dir) {
					runtimesDir = status.runtimes_dir;
				}
				if (status.host) {
					host = status.host;
				}
				if (status.port) {
					port = status.port;
				}
			}

			if (hw) {
				hardwareReport = hw;
			}
		} catch (err) {
			console.error('Failed to load setup configuration:', err);
		} finally {
			loading = false;
			rechecking = false;
		}
	}

	async function saveSetup() {
		const targetModels = configStatus?.is_docker
			? configStatus.models_dir || '/app/models'
			: modelsDir.trim() || configStatus?.models_dir || '';
		const targetRuntimes = configStatus?.is_docker
			? configStatus.runtimes_dir || '/app/runtimes'
			: runtimesDir.trim() || configStatus?.runtimes_dir || '';
		const targetHost = configStatus?.is_docker ? '0.0.0.0' : host.trim();
		const parsedPort = configStatus?.is_docker ? configStatus.port || 8080 : Number(port);

		if (!targetModels) {
			saveError = t('setup.errModelsDir');
			return;
		}
		if (!targetRuntimes) {
			saveError = t('setup.errRuntimesDir');
			return;
		}
		if (!targetHost) {
			saveError = t('setup.errHost');
			return;
		}
		if (isNaN(parsedPort) || parsedPort < 1 || parsedPort > 65535) {
			saveError = t('setup.errPort');
			return;
		}

		try {
			saving = true;
			saveError = null;

			await configApi.setup({
				models: targetModels,
				runtimes: targetRuntimes,
				host: targetHost,
				port: parsedPort
			});

			if (checkConfigFromLayout) {
				await checkConfigFromLayout();
			}

			goto('/chat');
		} catch (err: any) {
			saveError = err?.message || t('setup.errGeneric');
		} finally {
			saving = false;
		}
	}

	onMount(() => {
		fetchConfigData();
	});
</script>

<svelte:head>
	<title>Monolai - Setup</title>
</svelte:head>

<!-- Full page wrapper with natural page scroll -->
<div
	class="min-h-screen w-full flex items-center justify-center p-4 sm:p-6 md:p-8 bg-[var(--bg-primary)]"
>
	{#if loading && !configStatus}
		<div
			class="flex flex-col items-center justify-center gap-3 p-8 text-[var(--text-muted)] text-sm"
		>
			<Spinner size={32} class="text-[var(--primary)]" />
			<p class="m-0 text-xs">{t('setup.checkingConfig')}</p>
		</div>
	{:else if configStatus}
		<div class="max-w-[700px] w-full flex flex-col my-auto">
			<Tabs
				bind:value={activeTab}
				items={!isStorageConfigured
					? [
							{ id: 'wizard', label: t('setup.interactiveSetup'), icon: Wand },
							{ id: 'manual', label: t('setup.manualConfiguration'), icon: FileCode }
						]
					: []}
				cardClass="p-6 flex flex-col gap-5 shadow-2xl"
			>
				{#snippet header()}
					<div class="flex items-center justify-between gap-4">
						<PageHeader
							title={t('setup.monolaiSetup')}
							subtitle={t(
								isStorageConfigured ? 'setup.setupDescHardware' : 'setup.setupDescStorage'
							)}
							icon={Sparkles}
						/>

						<Button
							variant="secondary"
							size="sm"
							onclick={fetchConfigData}
							disabled={rechecking}
							loading={rechecking}
							title={t('common.refresh')}
						>
							<RefreshCw size={12} class={rechecking ? 'animate-spin' : ''} />
							<span>{t('common.refresh')}</span>
						</Button>
					</div>
				{/snippet}

				{#if saveError}
					<Alert
						variant="error"
						dismissible
						ondismiss={() => (saveError = null)}
						message={saveError}
					/>
				{/if}

				{#if activeTab === 'wizard'}
					<SetupWizard
						{configStatus}
						{isStorageConfigured}
						{saving}
						bind:modelsDir
						bind:runtimesDir
						bind:host
						bind:port
						onsubmit={saveSetup}
						onResetModels={() => {
							if (configStatus?.models_dir) modelsDir = configStatus.models_dir;
						}}
						onResetRuntimes={() => {
							if (configStatus?.runtimes_dir) runtimesDir = configStatus.runtimes_dir;
						}}
					/>
				{:else}
					<SetupManualConfig
						{configStatus}
						{hardwareReport}
						{rechecking}
						onRecheck={fetchConfigData}
					/>
				{/if}
			</Tabs>
		</div>
	{/if}
</div>
