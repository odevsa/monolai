<script lang="ts">
	import { afterNavigate } from '$app/navigation';
	import { runtimesApi } from '$lib/api/runtimes';
	import { RuntimeCard } from '$lib/components/runtimes';
	import { Alert, Button, EmptyState, PageHeader, Spinner } from '$lib/components/ui';
	import { askConfirm } from '$lib/confirmStore';
	import { refreshFeatures } from '$lib/featuresStore';
	import { t } from '$lib/i18n';
	import { headerState } from '$lib/state/header.svelte';
	import type { Runtime } from '$lib/types/runtimes';
	import { Boxes } from '@lucide/svelte';
	import { onDestroy, onMount } from 'svelte';

	function isInstallingStatus(status?: string): boolean {
		if (!status) return false;
		const s = status.toLowerCase();
		return s.startsWith('download') || s.startsWith('extract');
	}

	let runtimes = $state<Runtime[]>([]);
	let selectedAccelerations = $state<Record<string, string>>({});
	let loading = $state(true);
	let error = $state<string | null>(null);

	// Map of active cancel callbacks keyed by runtime_id
	const activeStreams: Record<string, () => void> = {};

	async function fetchRuntimes() {
		try {
			loading = true;
			headerState.setRuntimesRefreshing(true);
			error = null;
			const data = await runtimesApi.list();
			runtimes = data || [];

			for (const r of runtimes) {
				if (!selectedAccelerations[r.id]) {
					const rec = r.available_accelerations?.find((a) => a.is_recommended);
					selectedAccelerations[r.id] =
						r.installed_acceleration ||
						rec?.id ||
						r.available_accelerations?.[0]?.id ||
						r.active_acceleration;
				}
			}

			// Reconnect SSE for any runtime currently downloading/extracting
			for (const runtime of runtimes) {
				if (isInstallingStatus(runtime.install_progress?.status) && !activeStreams[runtime.id]) {
					listenProgress(runtime.id);
				}
			}
		} catch (err: any) {
			error = err?.message || t('runtimes.errorLoading');
		} finally {
			loading = false;
			headerState.setRuntimesRefreshing(false);
		}
	}

	function listenProgress(runtimeId: string) {
		if (activeStreams[runtimeId]) {
			activeStreams[runtimeId]();
			delete activeStreams[runtimeId];
		}

		activeStreams[runtimeId] = runtimesApi.connectInstallStream(
			runtimeId,
			(progress) => {
				const idx = runtimes.findIndex((r) => r.id === runtimeId);
				if (idx !== -1) {
					runtimes[idx].install_progress = progress;
					if (progress.status === 'completed') {
						runtimes[idx].is_installed = true;
						if (activeStreams[runtimeId]) {
							activeStreams[runtimeId]();
							delete activeStreams[runtimeId];
						}
						fetchRuntimes();
						refreshFeatures(true);
					} else if (progress.status === 'error') {
						if (activeStreams[runtimeId]) {
							activeStreams[runtimeId]();
							delete activeStreams[runtimeId];
						}
					}
				}
			},
			() => {
				if (activeStreams[runtimeId]) {
					delete activeStreams[runtimeId];
				}
			}
		);
	}

	onMount(() => {
		fetchRuntimes();
		headerState.setRuntimesRefreshing(false, () => fetchRuntimes());

		const handleNavRuntimes = () => {
			fetchRuntimes();
		};

		window.addEventListener('monolai:nav-runtimes', handleNavRuntimes);

		return () => {
			window.removeEventListener('monolai:nav-runtimes', handleNavRuntimes);
			Object.values(activeStreams).forEach((cancel) => cancel());
		};
	});

	onDestroy(() => {
		headerState.setRuntimesRefreshing(false, undefined);
		Object.values(activeStreams).forEach((cancel) => cancel());
	});

	afterNavigate(() => {
		headerState.setRuntimesRefreshing(false, () => fetchRuntimes());
	});

	async function installRuntime(runtimeId: string) {
		try {
			const idx = runtimes.findIndex((r) => r.id === runtimeId);
			if (idx !== -1) {
				runtimes[idx].install_progress = {
					runtime_id: runtimeId,
					status: 'downloading',
					percent: 0,
					speed_mbps: 0,
					downloaded_bytes: 0,
					total_bytes: 0,
					error_message: null
				};
			}

			listenProgress(runtimeId);

			const chosenHardware = selectedAccelerations[runtimeId];
			console.log(
				`[Runtimes] Initiating install for '${runtimeId}' with chosen hardware:`,
				chosenHardware
			);
			await runtimesApi.install(runtimeId, chosenHardware);
		} catch (err: any) {
			console.error(`Failed to install runtime ${runtimeId}:`, err);
			const idx = runtimes.findIndex((r) => r.id === runtimeId);
			if (idx !== -1) {
				runtimes[idx].install_progress = {
					runtime_id: runtimeId,
					status: 'error',
					percent: 0,
					speed_mbps: 0,
					downloaded_bytes: 0,
					total_bytes: 0,
					error_message: err?.message || t('runtimes.installationError')
				};
			}
		}
	}

	async function uninstallRuntime(runtime: Runtime) {
		const confirmed = await askConfirm(
			t('runtimes.uninstallConfirmMsg', { name: runtime.name }),
			t('runtimes.uninstallConfirmTitle'),
			'danger',
			t('common.uninstall'),
			t('common.cancel')
		);

		if (!confirmed) return;

		try {
			await runtimesApi.uninstall(runtime.id);
			delete selectedAccelerations[runtime.id];
			await fetchRuntimes();
			refreshFeatures(true);
		} catch (err: any) {
			console.error(`Failed to uninstall runtime ${runtime.id}:`, err);
			error = err?.message || t('runtimes.uninstallError');
		}
	}

	async function confirmUpdateRuntime(runtime: Runtime) {
		const confirmed = await askConfirm(
			t('runtimes.updateConfirmMsg', {
				name: runtime.name,
				from: runtime.installed_version || 'unknown',
				to: runtime.version
			}),
			t('runtimes.updateConfirmTitle'),
			'primary',
			t('common.update'),
			t('common.cancel')
		);

		if (!confirmed) return;
		await installRuntime(runtime.id);
	}
</script>

<svelte:head>
	<title>{t('app.title')} - {t('runtimes.pageTitle')}</title>
</svelte:head>

<div class="w-full px-4 sm:px-8 py-5 md:py-8 pt-16 md:pt-16 max-w-7xl mx-auto box-border">
	<div class="flex flex-col gap-5 sm:gap-6">
		<PageHeader
			title={t('runtimes.pageTitle')}
			subtitle={t('runtimes.pageSubtitle')}
			icon={Boxes}
		/>

		{#if error}
			<Alert variant="error" title={t('common.connectionFailure')}>
				<div class="flex items-center justify-between gap-3">
					<span>{error}</span>
					<Button variant="secondary" size="sm" onclick={() => fetchRuntimes()}>
						{t('common.refresh')}
					</Button>
				</div>
			</Alert>
		{/if}

		{#if loading && runtimes.length === 0}
			<div
				class="flex flex-col items-center justify-center gap-3 p-16 text-[var(--text-muted)] text-sm"
			>
				<Spinner size={32} />
				<p class="m-0 text-xs">{t('common.loading')}</p>
			</div>
		{:else if runtimes.length === 0}
			<EmptyState
				title={t('runtimes.noRuntimesFound')}
				description={t('runtimes.noRuntimesDesc')}
				icon={Boxes}
			/>
		{:else}
			<div class="grid grid-cols-1 lg:grid-cols-2 gap-5 items-stretch">
				{#each runtimes as runtime (runtime.id)}
					<RuntimeCard
						{runtime}
						selectedAcceleration={selectedAccelerations[runtime.id]}
						onAccelerationChange={(id, val) => {
							selectedAccelerations[id] = val;
						}}
						onInstall={installRuntime}
						onUninstall={uninstallRuntime}
						onUpdate={confirmUpdateRuntime}
					/>
				{/each}
			</div>
		{/if}
	</div>
</div>
