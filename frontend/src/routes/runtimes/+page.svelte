<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
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
	import { runtimesRefreshing, runtimesRefreshFn } from '$lib/headerStore';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import Badge from '$lib/components/Badge.svelte';
	import Alert from '$lib/components/Alert.svelte';
	import Card from '$lib/components/Card.svelte';
	import llamaIcon from '$lib/assets/runtimes/llama-cpp.svg';
	import sdIcon from '$lib/assets/runtimes/sd-cpp.svg';

	const localIcons: Record<string, string> = {
		'llama-cpp': llamaIcon,
		'sd-cpp': sdIcon
	};

	interface InstallProgress {
		runtime_id: string;
		status: 'idle' | 'downloading' | 'extracting' | 'completed' | 'error';
		percent: number;
		speed_mbps: number;
		downloaded_bytes: number;
		total_bytes: number;
		error_message: string | null;
	}

	interface AccelerationOption {
		id: string;
		label: string;
		is_recommended: boolean;
	}

	interface Runtime {
		id: string;
		name: string;
		version: string;
		icon: string | null;
		website: string | null;
		description: string;
		features: string[];
		is_installed: boolean;
		installed_path: string | null;
		active_acceleration: string;
		installed_acceleration: string | null;
		available_accelerations: AccelerationOption[];
		install_progress: InstallProgress | null;
	}

	let runtimes = $state<Runtime[]>([]);
	let selectedAccelerations = $state<Record<string, string>>({});
	let loading = $state(true);
	let error = $state<string | null>(null);

	// Map of active event sources keyed by runtime_id
	const activeEventSources: Record<string, EventSource> = {};

	async function fetchRuntimes() {
		try {
			loading = true;
			runtimesRefreshing.set(true);
			error = null;
			const res = await fetch('/api/runtimes');
			if (!res.ok) {
				throw new Error('Failed to fetch runtimes');
			}
			const data: Runtime[] = await res.json();
			runtimes = data;

			for (const r of data) {
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
				const status = runtime.install_progress?.status;
				if (
					(status === 'downloading' || status === 'extracting') &&
					!activeEventSources[runtime.id]
				) {
					listenProgress(runtime.id);
				}
			}
		} catch (err: any) {
			error = err?.message || 'Error loading runtimes';
		} finally {
			loading = false;
			runtimesRefreshing.set(false);
		}
	}

	function listenProgress(runtimeId: string) {
		if (activeEventSources[runtimeId]) {
			activeEventSources[runtimeId].close();
		}

		const sse = new EventSource(`/api/runtimes/${encodeURIComponent(runtimeId)}/install/stream`);
		activeEventSources[runtimeId] = sse;

		sse.onmessage = (event) => {
			try {
				const progress: InstallProgress = JSON.parse(event.data);
				const idx = runtimes.findIndex((r) => r.id === runtimeId);
				if (idx !== -1) {
					runtimes[idx].install_progress = progress;
					if (progress.status === 'completed') {
						runtimes[idx].is_installed = true;
						sse.close();
						delete activeEventSources[runtimeId];
						// Refresh full state
						fetchRuntimes();
					} else if (progress.status === 'error') {
						sse.close();
						delete activeEventSources[runtimeId];
					}
				}
			} catch (e) {
				console.error('Error parsing SSE progress:', e);
			}
		};

		sse.onerror = () => {
			sse.close();
			delete activeEventSources[runtimeId];
		};
	}

	async function installRuntime(runtimeId: string) {
		try {
			// Optimistically set status
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
			const res = await fetch(`/api/runtimes/${encodeURIComponent(runtimeId)}/install`, {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ hardware: chosenHardware })
			});

			if (!res.ok) {
				const msg = await res.text();
				throw new Error(msg || 'Failed to start installation');
			}
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
			`Are you sure you want to uninstall '${runtime.name}'? Models requiring this runtime will not be able to run until reinstalled.`,
			`Uninstall ${runtime.name}`,
			'danger',
			'Uninstall',
			'Cancel'
		);

		if (!confirmed) return;

		try {
			const res = await fetch(`/api/runtimes/${encodeURIComponent(runtime.id)}`, {
				method: 'DELETE'
			});

			if (!res.ok) {
				const msg = await res.text();
				throw new Error(msg || 'Failed to uninstall runtime');
			}

			await fetchRuntimes();
		} catch (err: any) {
			alert(`Error uninstalling runtime: ${err?.message || err}`);
		}
	}

	function formatBytes(bytes: number): string {
		if (bytes === 0) return '0 B';
		const k = 1024;
		const sizes = ['B', 'KB', 'MB', 'GB'];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`;
	}

	onMount(() => {
		runtimesRefreshFn.set(fetchRuntimes);
		fetchRuntimes();
	});

	onDestroy(() => {
		runtimesRefreshFn.set(null);
		runtimesRefreshing.set(false);
		// Clean up all active SSE connections
		for (const key in activeEventSources) {
			activeEventSources[key].close();
		}
	});
</script>

<svelte:head>
	<title>Monolai - Runtimes</title>
</svelte:head>

<div class="page-container">
	<div class="page-inner gap-8">
		<!-- Standard Page Header matching /sysinfo -->
		<PageHeader
			title="Runtimes"
			subtitle="1-click installation of upstream AI inference engines"
			icon={Boxes}
		/>

		{#if error}
			<Alert variant="error" title="Backend Connection Failure:" message={error}>
				{#snippet action()}
					<button
						type="button"
						class="app-btn app-btn-secondary app-btn-sm"
						onclick={fetchRuntimes}
					>
						Retry Connection
					</button>
				{/snippet}
			</Alert>
		{/if}

		{#if loading && runtimes.length === 0}
			<div
				class="flex flex-col items-center justify-center gap-3 p-12 text-[var(--text-muted)] text-sm"
			>
				<div
					class="w-8 h-8 rounded-full border-2 border-[var(--border-color)] border-t-[var(--primary)] animate-spin"
				></div>
				<p class="m-0">Loading available runtimes...</p>
			</div>
		{:else}
			<div class="grid grid-cols-1 lg:grid-cols-2 gap-5 items-stretch">
				{#each runtimes as runtime}
					{@const progress = runtime.install_progress}
					{@const isInstalling =
						progress && (progress.status === 'downloading' || progress.status === 'extracting')}

					<Card class="justify-between">
						<!-- Card Top Details -->
						<div class="flex flex-col gap-4">
							<div class="flex items-start justify-between gap-3">
								<div class="flex items-start gap-3.5 min-w-0 flex-1">
									<div
										class="w-13 h-13 rounded-2xl bg-[var(--bg-surface-hover)] border-0 flex items-center justify-center p-2.5 shrink-0 overflow-hidden shadow-xs"
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
										<div class="flex items-center gap-2 flex-wrap">
											<h3 class="m-0 text-base font-bold text-[var(--text-primary)] truncate">
												{runtime.name}
											</h3>
											{#if runtime.website}
												<a
													href={runtime.website}
													target="_blank"
													rel="noopener noreferrer"
													class="text-[var(--text-muted)] hover:text-[var(--primary)] transition-colors shrink-0"
													title="Visit official repository"
												>
													<ExternalLink size={13} />
												</a>
											{/if}
										</div>

										<div class="flex items-center gap-1.5 mt-1.5 flex-wrap">
											<Badge variant="pill">v{runtime.version}</Badge>
											<Badge variant="pill" class="text-[var(--primary)] font-semibold uppercase">
												{runtime.installed_acceleration || runtime.active_acceleration}
											</Badge>
										</div>
									</div>
								</div>

								<!-- Status Badge -->
								<div class="shrink-0 pt-0.5">
									{#if runtime.is_installed}
										<Badge variant="success" dot>Installed</Badge>
									{:else if isInstalling}
										<Badge variant="warning">
											<RefreshCw size={11} class="animate-spin" />
											Installing
										</Badge>
									{:else}
										<Badge variant="muted">Not Installed</Badge>
									{/if}
								</div>
							</div>

							<!-- Description -->
							<p class="m-0 text-xs leading-relaxed text-[var(--text-secondary)]">
								{runtime.description}
							</p>

							<!-- Features Badges -->
							{#if runtime.features && runtime.features.length > 0}
								<div class="flex items-center gap-1.5 flex-wrap">
									{#each runtime.features as feat}
										<Badge variant="pill">{feat}</Badge>
									{/each}
								</div>
							{/if}

							<!-- Installed Binary Path Hint -->
							{#if runtime.is_installed && runtime.installed_path}
								<div
									class="flex items-center gap-2 p-2.5 rounded-xl bg-[var(--bg-surface-hover)] border-0 text-[11px] font-mono text-[var(--text-muted)]"
								>
									<FolderCheck size={13} class="text-emerald-400 shrink-0" />
									<span class="truncate" title={runtime.installed_path}
										>{runtime.installed_path}</span
									>
								</div>
							{/if}

							<!-- Acceleration Selector -->
							{#if runtime.available_accelerations && runtime.available_accelerations.length > 0}
								<div
									class="flex items-center justify-between gap-3 p-2.5 rounded-xl bg-white/[0.02] border border-[var(--border-color)]"
								>
									<div class="flex flex-col min-w-0">
										<span class="text-[11px] font-semibold text-[var(--text-secondary)]">Hardware Target</span>
										<span class="text-[10px] text-[var(--text-muted)] truncate">Choose GPU backend or CUDA version</span>
									</div>
									<select
										class="px-2.5 py-1 text-xs font-semibold rounded-lg bg-[var(--bg-surface-hover)] border border-[var(--border-color)] text-[var(--text-primary)] cursor-pointer focus:outline-none focus:border-[var(--primary)] transition disabled:opacity-60 disabled:cursor-not-allowed max-w-[210px]"
										bind:value={selectedAccelerations[runtime.id]}
										disabled={isInstalling}
									>
										{#each runtime.available_accelerations as opt}
											<option value={opt.id}>
												{opt.label}{opt.is_recommended ? ' (Recommended)' : ''}
											</option>
										{/each}
									</select>
								</div>
							{/if}

							<!-- Installation Error Notice if any -->
							{#if progress && progress.status === 'error'}
								<Alert
									variant="error"
									message={progress.error_message || 'Installation error occurred'}
									class="py-2 px-3 text-xs"
								/>
							{/if}
						</div>

						<!-- Card Bottom Actions with Embedded Progress -->
						<div class="pt-2 border-0 flex items-center justify-end gap-2.5 flex-wrap">
							{#if !runtime.is_installed}
								{#if isInstalling && progress}
									<!-- In-Button Progress State -->
									<button
										type="button"
										disabled={true}
										class="app-btn app-btn-primary app-btn-md w-full relative overflow-hidden select-none cursor-wait text-white"
										title="Installation in progress"
									>
										<!-- Progress Fill Bar -->
										<div
											class="absolute inset-0 bg-white/25 transition-all duration-200 pointer-events-none"
											style="width: {Math.max(2, Math.min(100, progress.percent))}%"
										></div>

										<!-- Label and Percent Overlay -->
										<div
											class="relative z-10 w-full flex items-center justify-between gap-2 px-1 text-xs font-semibold"
										>
											<div class="flex items-center gap-2 min-w-0">
												<RefreshCw size={13} class="animate-spin shrink-0" />
												<span class="truncate">
													{progress.status === 'extracting'
														? 'Extracting files...'
														: 'Downloading engine...'}
												</span>
											</div>
											<div class="flex items-center gap-1.5 shrink-0 font-mono text-[11px]">
												{#if progress.speed_mbps > 0}
													<span class="opacity-80 font-normal hidden sm:inline"
														>{progress.speed_mbps.toFixed(1)} MB/s •</span
													>
												{/if}
												{#if progress.total_bytes > 0}
													<span class="opacity-80 font-normal hidden md:inline"
														>{formatBytes(progress.downloaded_bytes)} / {formatBytes(
															progress.total_bytes
														)} •</span
													>
												{/if}
												<span class="font-bold">{progress.percent.toFixed(0)}%</span>
											</div>
										</div>
									</button>
								{:else}
									<button
										onclick={() => installRuntime(runtime.id)}
										class="app-btn app-btn-primary app-btn-md w-full"
									>
										<Download size={14} />
										<span>Install Runtime</span>
									</button>
								{/if}
							{:else}
								<button
									onclick={() => uninstallRuntime(runtime)}
									disabled={isInstalling}
									class="app-btn app-btn-danger app-btn-sm"
									title="Uninstall this runtime"
								>
									<Trash2 size={13} />
									<span>Uninstall</span>
								</button>

								{#if isInstalling && progress}
									<!-- In-Button Progress for Reinstall -->
									<button
										type="button"
										disabled={true}
										class="app-btn app-btn-secondary app-btn-sm relative overflow-hidden select-none cursor-wait min-w-[200px]"
										title="Reinstallation in progress"
									>
										<div
											class="absolute inset-0 bg-[var(--primary)]/20 transition-all duration-200 pointer-events-none"
											style="width: {Math.max(2, Math.min(100, progress.percent))}%"
										></div>
										<div
											class="relative z-10 w-full flex items-center justify-between gap-2 text-xs font-medium"
										>
											<div class="flex items-center gap-1.5">
												<RefreshCw size={12} class="animate-spin text-[var(--primary)] shrink-0" />
												<span
													>{progress.status === 'extracting'
														? 'Extracting...'
														: 'Downloading...'}</span
												>
											</div>
											<span class="font-mono font-bold text-[11px] text-[var(--primary)]"
												>{progress.percent.toFixed(0)}%</span
											>
										</div>
									</button>
								{:else}
									<button
										onclick={() => installRuntime(runtime.id)}
										class="app-btn app-btn-secondary app-btn-sm"
										title="Reinstall or update runtime to latest package"
									>
										<RefreshCw size={13} />
										<span>Reinstall</span>
									</button>
								{/if}
							{/if}
						</div>
					</Card>
				{/each}
			</div>
		{/if}
	</div>
</div>
