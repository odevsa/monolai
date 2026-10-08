<script lang="ts">
	import { MAX_CHART_POINTS } from '$lib';
	import { hostApi } from '$lib/api/host';
	import { Alert, AreaChart, PageHeader } from '$lib/components/ds';
	import { t } from '$lib/i18n';
	import {
		getStatusType,
		runningModelsState,
		startRunningStatePolling,
		unloadAllModels,
		unloadModel
	} from '$lib/runningModelsStore';
	import { headerState } from '$lib/state/header.svelte';
	import type { HostMetricsTick, SysInfo } from '$lib/types/host';
	import { formatBytes, formatUptime } from '$lib/utils/format';
	import { parseModelDetails } from '$lib/utils/model';
	import { Activity, Box, Cpu, Gpu, MemoryStick, Monitor, Power, RefreshCw } from '@lucide/svelte';
	import { onDestroy, onMount } from 'svelte';

	let runningModelsList = $derived(runningModelsState.runningModels);
	let unloadingIds = $derived(runningModelsState.unloadingModelIds);

	let sysinfo = $state<SysInfo | null>(null);
	let loadingStatic = $state(true);
	let sseConnected = $state(false);
	let error = $state<string | null>(null);

	let latestTick = $state<HostMetricsTick | null>(null);
	let cpuHistory = $state<number[]>([]);
	let ramHistory = $state<number[]>([]);
	let gpuHistory = $state<number[]>([]);
	let vramHistory = $state<number[]>([]);

	let unsubscribeMetrics: (() => void) | null = null;

	async function fetchStaticHostInfo() {
		try {
			loadingStatic = true;
			const data = await hostApi.getInfo();
			sysinfo = data;
			error = null;
		} catch (e: any) {
			error = e.message || 'Failed to connect to backend';
		} finally {
			loadingStatic = false;
		}
	}

	function reconnectMetricsStream() {
		fetchStaticHostInfo();
		connectMetricsStream();
	}

	function connectMetricsStream() {
		if (unsubscribeMetrics) {
			unsubscribeMetrics();
		}

		unsubscribeMetrics = hostApi.subscribeMetrics(
			(tick) => {
				latestTick = tick;
				sseConnected = true;
				error = null;
				headerState.setSysinfoStatus(true, reconnectMetricsStream);

				cpuHistory = [...cpuHistory.slice(-(MAX_CHART_POINTS - 1)), tick.cpu_usage];
				ramHistory = [...ramHistory.slice(-(MAX_CHART_POINTS - 1)), tick.ram_percentage];

				if (tick.gpu_usage !== null && tick.gpu_usage !== undefined) {
					gpuHistory = [...gpuHistory.slice(-(MAX_CHART_POINTS - 1)), tick.gpu_usage];
				}

				if (tick.vram_percentage !== null && tick.vram_percentage !== undefined) {
					vramHistory = [...vramHistory.slice(-(MAX_CHART_POINTS - 1)), tick.vram_percentage];
				}
			},
			() => {
				sseConnected = false;
				headerState.setSysinfoStatus(false, reconnectMetricsStream);
			}
		);
	}

	let stopRunningStatePolling: (() => void) | null = null;

	onMount(() => {
		fetchStaticHostInfo();
		connectMetricsStream();
		stopRunningStatePolling = startRunningStatePolling(2500);
	});

	onDestroy(() => {
		if (unsubscribeMetrics) {
			unsubscribeMetrics();
			unsubscribeMetrics = null;
		}
		if (stopRunningStatePolling) {
			stopRunningStatePolling();
			stopRunningStatePolling = null;
		}
		headerState.setSysinfoStatus(false);
	});

	let isGpuUnavailable = $derived.by(() => {
		if (sysinfo && (!sysinfo.gpu || !sysinfo.gpu.is_dedicated)) return true;
		if (!sysinfo && latestTick && latestTick.gpu_usage === null) return true;
		return false;
	});

	let isVramUnavailable = $derived.by(() => {
		if (isGpuUnavailable) return true;
		if (sysinfo?.vram && sysinfo.vram.total_bytes > 0) return false;
		if (
			latestTick &&
			latestTick.vram_percentage !== null &&
			latestTick.vram_percentage !== undefined
		)
			return false;
		if (
			latestTick &&
			latestTick.vram_total_bytes !== null &&
			latestTick.vram_total_bytes !== undefined &&
			latestTick.vram_total_bytes > 0
		)
			return false;
		if (sysinfo?.gpu?.memory_total_bytes && sysinfo.gpu.memory_total_bytes > 0) return false;
		return true;
	});
</script>

<svelte:head>
	<title>Monolai - System Info</title>
</svelte:head>

<div class="w-full px-4 sm:px-8 py-5 md:py-8 pt-16 md:pt-16 max-w-7xl mx-auto box-border">
	<div class="flex flex-col gap-6">
		<!-- Page Header (clean, without duplicate Live Stream badge which lives in the floating header) -->
		<PageHeader
			title={t('sysinfo.pageTitle')}
			subtitle={t('sysinfo.pageSubtitle')}
			icon={Activity}
		/>

		{#if error}
			<Alert variant="error" title="Backend Connection Failure:">
				<div class="flex items-center justify-between gap-3">
					<span>{error}</span>
					<button
						type="button"
						class="px-3 py-1 rounded-lg text-xs font-semibold bg-white/10 hover:bg-white/15 cursor-pointer text-[var(--text-primary)]"
						onclick={reconnectMetricsStream}
					>
						{t('common.refresh')}
					</button>
				</div>
			</Alert>
		{/if}

		<!-- Real-Time Mountain Charts Section -->
		<section class="flex flex-col gap-4">
			<div class="grid grid-cols-1 md:grid-cols-2 gap-5 items-stretch">
				<!-- CPU Usage Mountain Chart -->
				<AreaChart
					title="CPU Utilization"
					subtitle={sysinfo
						? `${sysinfo.cpu.brand} (${sysinfo.cpu.cores} cores)`
						: 'Host Processor'}
					currentValue={latestTick?.cpu_usage ?? sysinfo?.cpu?.usage ?? 0}
					unit="%"
					color="#9845e7"
					icon={Cpu}
					data={cpuHistory}
					maxVal={100}
				/>

				<!-- RAM Usage Mountain Chart -->
				<AreaChart
					title="RAM Usage"
					subtitle={sysinfo
						? `${formatBytes(latestTick?.ram_used_bytes ?? sysinfo.ram.used_bytes)} / ${formatBytes(sysinfo.ram.total_bytes)}`
						: 'System Memory'}
					currentValue={latestTick?.ram_percentage ?? sysinfo?.ram?.percentage ?? 0}
					unit="%"
					color="#c875ff"
					icon={MemoryStick}
					data={ramHistory}
					maxVal={100}
				/>

				<!-- GPU Utilization Mountain Chart -->
				<AreaChart
					title="GPU Utilization"
					subtitle={sysinfo?.gpu ? sysinfo.gpu.name : 'Dedicated Graphics'}
					currentValue={latestTick?.gpu_usage ?? (isGpuUnavailable ? 0 : 0)}
					unit="%"
					color="#00a971"
					icon={Gpu}
					data={gpuHistory}
					maxVal={100}
					unavailable={isGpuUnavailable}
					unavailableMessage="No Dedicated GPU Detected"
				/>

				<!-- VRAM Memory Mountain Chart -->
				<AreaChart
					title="VRAM Usage"
					subtitle={!isVramUnavailable
						? `${formatBytes(latestTick?.vram_used_bytes ?? sysinfo?.vram?.used_bytes ?? 0)} / ${formatBytes(latestTick?.vram_total_bytes ?? sysinfo?.vram?.total_bytes ?? sysinfo?.gpu?.memory_total_bytes ?? 0)}`
						: 'Dedicated Video RAM'}
					currentValue={latestTick?.vram_percentage ?? sysinfo?.vram?.percentage ?? 0}
					unit="%"
					color="#30d9a1"
					icon={MemoryStick}
					data={vramHistory}
					maxVal={100}
					unavailable={isVramUnavailable}
					unavailableMessage="No Dedicated VRAM Detected"
				/>
			</div>
		</section>

		<!-- Active Models Section -->
		<section class="bg-[var(--bg-surface)] rounded-2xl p-6 flex flex-col gap-4 border-0 shadow-xs">
			<div class="flex items-center justify-between">
				<div class="flex items-center gap-3">
					<div class="flex items-center gap-2">
						<Box size={16} class="text-[var(--primary)]" />
						<h3 class="app-card-title text-sm">Active Models</h3>
					</div>
				</div>

				{#if runningModelsList.length > 0}
					<button
						type="button"
						class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold text-red-400 bg-red-500/10 border border-red-500/20 cursor-pointer transition-all hover:bg-red-500/20 hover:border-red-500/30 active:scale-95"
						onclick={() => unloadAllModels()}
						title="Unload all running models"
					>
						<Power size={13} />
						<span>Unload All</span>
					</button>
				{/if}
			</div>

			{#if runningModelsList.length === 0}
				<div
					class="flex flex-col stretch items-center gap-2 p-4 rounded-xl bg-white/[0.02] border-0 text-[var(--text-muted)]"
				>
					<div
						class="flex items-center justify-center shrink-0 w-8 h-8 rounded-lg bg-white/[0.04] text-[var(--text-muted)]"
					>
						<Box size={16} />
					</div>
					<div>
						<div class="text-xs text-center font-semibold text-[var(--text-primary)]">
							No Models Currently Loaded
						</div>
						<div class="text-[0.75rem] text-center text-[var(--text-muted)] max-w-md">
							Inference processes spawn automatically when a chat or OpenAI API request is made, and
							auto-unload when idle.
						</div>
					</div>
				</div>
			{:else}
				<div class="grid grid-cols-1 md:grid-cols-2 gap-3.5">
					{#each runningModelsList as model (model.model_id)}
						{@const isUnloading = unloadingIds.has(model.model_id)}
						{@const rawStatus = getStatusType(model.state)}
						{@const statusType = isUnloading ? 'loading' : rawStatus}
						{@const parsed = parseModelDetails(model.model_id)}
						{@const modelDisplayName = parsed.name || model.model_id}
						<div
							class="flex flex-col justify-between gap-3 p-4 rounded-xl bg-white/[0.03] border-0 transition-all hover:bg-white/[0.06]"
						>
							<!-- Top row: indicators and runtime -->
							<div class="flex items-center justify-between">
								<div class="flex items-center gap-2">
									<span
										class="w-2.5 h-2.5 rounded-full block {statusType === 'ready'
											? 'bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.5)] animate-pulse'
											: statusType === 'loading'
												? 'bg-amber-500 animate-pulse'
												: 'bg-red-500'}"
										title={statusType === 'ready'
											? 'Ready'
											: statusType === 'loading'
												? 'Loading / Unloading...'
												: 'Error'}
									></span>
									<Box size={16} class="text-[var(--text-muted)]" />

									<span
										class="text-[0.625rem] font-semibold bg-[var(--bg-surface)] border border-[var(--border-color)] text-[var(--text-secondary)] px-2 py-0.5 rounded-md uppercase tracking-wider inline-block"
									>
										{model.runtime_id}
									</span>
								</div>

								<button
									type="button"
									class="flex items-center justify-center w-7 h-7 rounded-lg text-[var(--text-muted)] border border-[var(--border-color)] bg-[var(--bg-surface)] cursor-pointer transition-all hover:text-red-400 hover:border-red-500/30 hover:bg-red-500/10 active:scale-95 disabled:opacity-50 disabled:cursor-not-allowed shrink-0"
									disabled={isUnloading}
									onclick={() => unloadModel(model.model_id)}
									title="Unload {modelDisplayName}"
									aria-label="Unload {modelDisplayName}"
								>
									{#if isUnloading}
										<RefreshCw size={13} class="animate-spin text-amber-400" />
									{:else}
										<Power size={13} />
									{/if}
								</button>
							</div>

							<!-- Model Name -->
							<div class="min-w-0">
								<h4
									class="m-0 text-sm font-bold text-[var(--text-primary)] truncate"
									title={model.model_id}
								>
									{modelDisplayName}
								</h4>
							</div>

							<!-- Badges -->
							{#if parsed.tags.length > 0}
								<div class="flex items-center gap-1.5 flex-wrap">
									{#each parsed.tags as tag}
										<span
											class="text-[0.675rem] font-medium bg-white/[0.06] border border-white/10 text-[var(--text-secondary)] px-2 py-0.5 rounded-full tabular-nums"
										>
											{tag}
										</span>
									{/each}
								</div>
							{/if}

							<!-- Metrics Table -->
							<div class="mt-auto pt-2.5 border-t border-[var(--border-color)]/40">
								<table class="w-full text-[0.725rem] font-mono border-collapse">
									<thead>
										<tr
											class="text-[var(--text-muted)] text-[0.65rem] uppercase tracking-wider border-b border-white/[0.06]"
										>
											<th class="pb-1.5 text-left font-medium">PID</th>
											<th class="pb-1.5 text-left font-medium">Port</th>
											<th class="pb-1.5 text-right font-medium">Idle</th>
										</tr>
									</thead>
									<tbody>
										<tr class="text-[var(--text-primary)] font-medium">
											<td class="pt-1.5 text-left font-semibold"
												>{model.pid > 0 ? model.pid : '—'}</td
											>
											<td class="pt-1.5 text-left font-semibold"
												>{model.port > 0 ? model.port : '—'}</td
											>
											<td class="pt-1.5 text-right font-semibold"
												>{formatUptime(model.idle_seconds)}</td
											>
										</tr>
									</tbody>
								</table>
							</div>

							{#if typeof model.state === 'object' && model.state.status === 'Error'}
								<div
									class="text-[0.725rem] text-red-400 bg-red-500/10 border border-red-500/20 px-2.5 py-1.5 rounded-lg"
								>
									{model.state.message || 'Model process failed'}
								</div>
							{/if}
						</div>
					{/each}
				</div>
			{/if}
		</section>

		<!-- Static System Info Cards -->
		{#if sysinfo}
			<section
				class="bg-[var(--bg-surface)] rounded-2xl p-6 flex flex-col gap-5 border-0 shadow-xs"
			>
				<div class="flex items-center gap-3">
					<div class="flex items-center gap-2">
						<Monitor size={16} class="text-[var(--primary)]" />
						<h3 class="app-card-title text-sm">{t('sysinfo.hostDetails')}</h3>
					</div>
				</div>

				<div
					class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 {sysinfo.gpu
						? (sysinfo.vram?.total_bytes ?? sysinfo.gpu.memory_total_bytes)
							? 'xl:grid-cols-6'
							: 'xl:grid-cols-5'
						: ''} gap-4"
				>
					<div class="flex flex-col gap-1.5 p-3.5 bg-white/[0.02] rounded-xl border-0">
						<span
							class="text-[0.75rem] font-semibold uppercase tracking-wider text-[var(--text-muted)]"
							>{t('sysinfo.osTitle')}</span
						>
						<span class="text-sm font-semibold text-[var(--text-primary)]"
							>{sysinfo.os.name} {sysinfo.os.os_version}</span
						>
					</div>

					<div class="flex flex-col gap-1.5 p-3.5 bg-white/[0.02] rounded-xl border-0">
						<span
							class="text-[0.75rem] font-semibold uppercase tracking-wider text-[var(--text-muted)]"
							>{t('sysinfo.kernelVersion')}</span
						>
						<span class="text-sm font-semibold text-[var(--text-primary)]"
							>{sysinfo.os.kernel_version}</span
						>
					</div>

					<div class="flex flex-col gap-1.5 p-3.5 bg-white/[0.02] rounded-xl border-0">
						<span
							class="text-[0.75rem] font-semibold uppercase tracking-wider text-[var(--text-muted)]"
							>{t('sysinfo.hostname')}</span
						>
						<span class="text-sm font-semibold text-[var(--text-primary)]"
							>{sysinfo.os.hostname}</span
						>
					</div>

					<div class="flex flex-col gap-1.5 p-3.5 bg-white/[0.02] rounded-xl border-0">
						<span
							class="text-[0.75rem] font-semibold uppercase tracking-wider text-[var(--text-muted)]"
							>{t('sysinfo.uptimeTitle')}</span
						>
						<span class="text-sm font-semibold text-[var(--text-primary)]"
							>{formatUptime(sysinfo.os.uptime_seconds)}</span
						>
					</div>

					{#if sysinfo.gpu}
						<div class="flex flex-col gap-1.5 p-3.5 bg-white/[0.02] rounded-xl border-0">
							<span
								class="text-[0.75rem] font-semibold uppercase tracking-wider text-[var(--text-muted)]"
								>{t('sysinfo.dedicatedGpu')}</span
							>
							<span
								class="text-sm font-semibold text-[var(--text-primary)] truncate"
								title={sysinfo.gpu.name}>{sysinfo.gpu.name}</span
							>
						</div>
					{/if}

					{#if sysinfo.vram?.total_bytes ?? sysinfo.gpu?.memory_total_bytes}
						<div class="flex flex-col gap-1.5 p-3.5 bg-white/[0.02] rounded-xl border-0">
							<span
								class="text-[0.75rem] font-semibold uppercase tracking-wider text-[var(--text-muted)]"
								>{t('sysinfo.vramTitle')}</span
							>
							<span class="text-sm font-semibold text-[var(--text-primary)]">
								{formatBytes(sysinfo.vram?.total_bytes ?? sysinfo.gpu?.memory_total_bytes ?? 0)}
							</span>
						</div>
					{/if}
				</div>
			</section>
		{/if}
	</div>
</div>
