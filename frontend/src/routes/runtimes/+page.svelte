<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { afterNavigate } from '$app/navigation';
	import {
		Boxes,
		Download,
		ExternalLink,
		RefreshCw,
		Trash2,
		Cpu,
		FolderCheck
	} from '@lucide/svelte';
	import { askConfirm } from '$lib/confirmStore';
	import { headerState } from '$lib/state/header.svelte';
	import { runtimesApi } from '$lib/api/runtimes';
	import type { Runtime, InstallProgress, AccelerationOption } from '$lib/types/runtimes';
	import { t } from '$lib/i18n';
	import { Button, Badge, Card, Select, PageHeader, Alert, EmptyState, Label, DiagonalLabel } from '$lib/components/ds';
	import { refreshFeatures } from '$lib/featuresStore';
	import { formatBytes } from '$lib/utils/format';
	import llamaIcon from '$lib/assets/runtimes/llama-cpp.svg';
	import sdIcon from '$lib/assets/runtimes/sd-cpp.svg';

	const localIcons: Record<string, string> = {
		'llama-cpp': llamaIcon,
		'sd-cpp': sdIcon
	};

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
			error = err?.message || 'Error loading runtimes';
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
					error_message: err?.message || 'Installation initiation failed'
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
			await fetchRuntimes();
			refreshFeatures(true);
		} catch (err: any) {
			console.error(`Failed to uninstall runtime ${runtime.id}:`, err);
			error = err?.message || 'Failed to uninstall runtime';
		}
	}
</script>

<div class="w-full px-4 sm:px-8 py-5 md:py-8 pt-16 md:pt-16 max-w-7xl mx-auto box-border">
	<div class="flex flex-col gap-5 sm:gap-6">
		<PageHeader
			title={t('runtimes.pageTitle')}
			subtitle={t('runtimes.pageSubtitle')}
			icon={Boxes}
		/>

		{#if error}
			<Alert variant="error" title="Backend Connection Failure:">
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
				<div
					class="w-8 h-8 rounded-full border-2 border-[var(--border-color)] border-t-[var(--primary)] animate-spin"
				></div>
				<p class="m-0 text-xs">{t('common.loading')}</p>
			</div>
		{:else if runtimes.length === 0}
			<EmptyState
				title={t('runtimes.noRuntimesFound')}
				description="No executable runtime backends configured on the server."
				icon={Boxes}
			/>
		{:else}
			<div class="grid grid-cols-1 lg:grid-cols-2 gap-5 items-stretch">
				{#each runtimes as runtime}
					{@const progress = runtime.install_progress}
					{@const isInstalling = isInstallingStatus(progress?.status)}

					<Card class="relative">
						{#if runtime.is_installed}
							<DiagonalLabel text={t('runtimes.installed')} />
						{/if}

						<div class="flex flex-col gap-4 justify-between h-full">
							<!-- Card Top Details -->
							<div class="flex flex-col gap-4 justify-between h-full">
								<div class="flex items-start justify-between gap-3">
									<div class="flex items-start gap-3.5 min-w-0 flex-1">
										<div
											class="w-14 h-14 rounded-2xl bg-[var(--bg-surface-hover)] border border-[var(--border-color)] flex items-center justify-center p-2.5 shrink-0 overflow-hidden shadow-xs"
										>
											{#if localIcons[runtime.id] || runtime.icon}
												<img
													src={localIcons[runtime.id] || runtime.icon}
													alt={runtime.name}
													class="w-full h-full object-contain"
												/>
											{:else}
												<Cpu size={26} class="text-[var(--primary)]" />
											{/if}
										</div>

										<div class="min-w-0 flex-1">
											<div class="flex flex-col gap-0">
												<h3
													class="m-0 text-base font-bold text-[var(--text-primary)] truncate"
													title={runtime.name}
												>
													{runtime.name}
												</h3>
												{#if runtime.website}
													<a
														href={runtime.website}
														target="_blank"
														rel="noopener noreferrer"
														class="text-[var(--primary)] hover:text-[var(--primary-hover)] text-xs transition-colors truncate"
														title="Visit official repository"
													>
														{runtime.website}
													</a>
												{/if}
												<span class="text-[var(--text-muted)] text-xs">Version: {runtime.version}</span>
											</div>
										</div>
									</div>
								</div>

								<div class="flex flex-col gap-2 mb-auto">
									<!-- Description -->
									<p class="m-0 text-xs leading-relaxed text-[var(--text-secondary)]">
										{t(`runtimes.description.${runtime.id}`)}
									</p>

									<!-- Features Badges -->
									{#if runtime.features && runtime.features.length > 0}
										<div class="flex items-center gap-1.5 flex-wrap">
											{#each runtime.features as feat}
												<Badge variant="pill">{feat}</Badge>
											{/each}
										</div>
									{/if}
								</div>

								<!-- Installed Binary Path Hint -->
								{#if runtime.is_installed && runtime.installed_path}
									<div
										class="flex items-center gap-2 p-2.5 rounded-xl bg-[var(--bg-primary)] border border-[var(--border-color)] text-[11px] font-mono text-[var(--text-muted)]"
									>
										<FolderCheck size={13} class="text-emerald-400 shrink-0" />
										<span class="truncate" title={runtime.installed_path}
											>{runtime.installed_path}</span
										>
										<Badge variant="success">{runtime.installed_acceleration}</Badge>
									</div>
								{/if}

								<!-- Acceleration Selector -->
								{#if !runtime.is_installed && !isInstalling && runtime.available_accelerations && runtime.available_accelerations.length > 0}
									<div
										class="flex flex-col gap-2 p-2.5 rounded-xl bg-[var(--bg-primary)] border border-[var(--border-color)]"
									>
										<div class="flex items-center justify-between gap-2">
											<span
												class="text-xs font-semibold text-[var(--text-primary)] whitespace-nowrap shrink-0"
												>{t('runtimes.accelerationLabel')}</span
											>
											<span class="text-[11px] text-[var(--text-muted)] truncate text-right"
												>Choose GPU backend or CUDA version</span
											>
										</div>
										<div class="w-full">
											<Select
												bind:value={selectedAccelerations[runtime.id]}
												options={runtime.available_accelerations.map((opt) => ({
													value: opt.id,
													label:
														opt.label +
														(opt.is_recommended ? ` (${t('runtimes.recommendedBadge')})` : '')
												}))}
												placeholder="Select hardware target..."
												ariaLabel={t('runtimes.accelerationLabel')}
											/>
										</div>
									</div>
								{/if}

								<!-- Installation Error Notice if any -->
								{#if progress && progress.status === 'error'}
									<Alert variant="error" dismissible>
										{progress.error_message || 'Installation error occurred'}
									</Alert>
								{/if}
							</div>

							<!-- Card Bottom Actions with Dedicated Progress Bar -->
							<div
								class="pt-4 border-t border-[var(--border-color)] flex items-center justify-end gap-2.5 flex-wrap"
							>
								{#if isInstalling && progress}
									<div
										class="w-full flex flex-col gap-2 p-3 rounded-xl bg-[var(--bg-primary)] border border-[var(--border-color)]"
									>
										<div class="flex items-center justify-between gap-3 text-xs">
											<div class="flex items-center gap-2 text-[var(--text-primary)] min-w-0">
												<RefreshCw size={13} class="animate-spin text-[var(--primary)] shrink-0" />
												<span class="font-medium truncate">
													{#if progress.message}
														{progress.message}
													{:else if progress.status === 'extracting'}
														{t('runtimes.extracting')}...
													{:else}
														{t('runtimes.downloading')}...
													{/if}
												</span>
											</div>
											<span class="font-mono text-xs font-bold text-[var(--primary)] shrink-0">
												{progress.percent.toFixed(0)}%
											</span>
										</div>

										<div class="w-full h-2 rounded-full bg-[var(--bg-surface)] overflow-hidden">
											<div
												class="h-full bg-[var(--primary)] transition-all duration-200 rounded-full"
												style="width: {Math.max(2, Math.min(100, progress.percent))}%"
											></div>
										</div>

										{#if progress.total_bytes > 0 || progress.speed_mbps > 0}
											<div
												class="flex items-center justify-between text-[11px] font-mono text-[var(--text-muted)] pt-0.5"
											>
												<span>
													{#if progress.total_bytes > 0}
														{formatBytes(progress.downloaded_bytes)} of {formatBytes(
															progress.total_bytes
														)}
													{/if}
												</span>
												<span>
													{#if progress.speed_mbps > 0}
														{progress.speed_mbps.toFixed(1)} MB/s
													{/if}
												</span>
											</div>
										{/if}
									</div>
								{:else if !runtime.is_installed}
									<Button
										variant="primary"
										size="md"
										class="w-full"
										onclick={() => installRuntime(runtime.id)}
									>
										<Download size={14} />
										<span>{t('runtimes.installBtn')}</span>
									</Button>
								{:else}
									<Button
										variant="danger"
										size="sm"
										onclick={() => uninstallRuntime(runtime)}
										title="Uninstall this runtime"
									>
										<Trash2 size={13} />
										<span>{t('runtimes.uninstallBtn')}</span>
									</Button>

									<Button
										variant="secondary"
										size="sm"
										onclick={() => installRuntime(runtime.id)}
										title="Reinstall or update runtime to latest package"
									>
										<RefreshCw size={13} />
										<span>Reinstall</span>
									</Button>
								{/if}
							</div>
						</div>
					</Card>
				{/each}
			</div>
		{/if}
	</div>
</div>
