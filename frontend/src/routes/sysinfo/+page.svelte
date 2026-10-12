<script lang="ts">
	import { MAX_CHART_POINTS } from '$lib';
	import {
		SysinfoActiveModels,
		SysinfoHostDetails,
		SysinfoTelemetryGrid
	} from '$lib/components/sysinfo';
	import { Alert, Button, PageHeader } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import {
		runningModelsState,
		startRunningStatePolling,
		unloadAllModels,
		unloadModel
	} from '$lib/runningModelsStore';
	import { headerState } from '$lib/state/header.svelte';
	import { hostState } from '$lib/state/host.svelte';
	import type { GpuDeviceInfo } from '$lib/types/host';
	import { Activity } from '@lucide/svelte';
	import { onDestroy, onMount } from 'svelte';

	let runningModelsList = $derived(runningModelsState.runningModels);
	let unloadingIds = $derived(runningModelsState.unloadingModelIds);

	let sysinfo = $derived(hostState.sysinfo);
	let latestTick = $derived(hostState.latestTick);
	let loadingStatic = $state(true);
	let error = $derived(hostState.error);

	let cpuHistory = $state<number[]>([]);
	let ramHistory = $state<number[]>([]);
	let diskIoHistory = $state<number[]>([]);
	let netHistory = $state<number[]>([]);
	let gpuHistoryMap = $state<Record<number, number[]>>({});
	let vramHistoryMap = $state<Record<number, number[]>>({});

	let unsubscribeMetrics: (() => void) | null = null;
	let stopRunningStatePolling: (() => void) | null = null;

	let gpusList = $derived.by<GpuDeviceInfo[]>(() => {
		if (latestTick?.gpus && latestTick.gpus.length > 0) return latestTick.gpus;
		if (sysinfo?.gpus && sysinfo.gpus.length > 0) return sysinfo.gpus;
		if (sysinfo?.gpu) {
			return [
				{
					index: 0,
					name: sysinfo.gpu.name,
					vendor: sysinfo.gpu.vendor,
					memory_total_bytes: sysinfo.gpu.memory_total_bytes,
					memory_used_bytes: latestTick?.vram_used_bytes ?? sysinfo.vram?.used_bytes ?? null,
					memory_free_bytes: sysinfo.vram?.free_bytes ?? null,
					driver_version: sysinfo.gpu.driver_version,
					is_dedicated: sysinfo.gpu.is_dedicated,
					usage: latestTick?.gpu_usage ?? null,
					temperature_c: null,
					clock_mhz: null
				}
			];
		}
		return [];
	});

	function reconnectMetricsStream() {
		hostState.reconnect();
		connectMetricsStream();
	}

	function connectMetricsStream() {
		if (unsubscribeMetrics) {
			unsubscribeMetrics();
		}

		unsubscribeMetrics = hostState.subscribe((tick) => {
			headerState.setSysinfoStatus(true, reconnectMetricsStream);

			cpuHistory = [...cpuHistory.slice(-(MAX_CHART_POINTS - 1)), tick.cpu_usage];
			ramHistory = [...ramHistory.slice(-(MAX_CHART_POINTS - 1)), tick.ram_percentage];

			// Disk I/O activity in raw bytes/sec
			const diskIoBytes = tick.disk_read_bytes_sec + tick.disk_write_bytes_sec;
			diskIoHistory = [...diskIoHistory.slice(-(MAX_CHART_POINTS - 1)), diskIoBytes];

			// Network throughput in raw bytes/sec
			const netBytes = tick.net_rx_bytes_sec + tick.net_tx_bytes_sec;
			netHistory = [...netHistory.slice(-(MAX_CHART_POINTS - 1)), netBytes];

			if (tick.gpus && tick.gpus.length > 0) {
				for (const g of tick.gpus) {
					const prevGpu = gpuHistoryMap[g.index] || [];
					gpuHistoryMap[g.index] = [...prevGpu.slice(-(MAX_CHART_POINTS - 1)), g.usage ?? 0];

					const vramPct =
						g.memory_total_bytes && g.memory_used_bytes
							? (g.memory_used_bytes / g.memory_total_bytes) * 100
							: 0;
					const prevVram = vramHistoryMap[g.index] || [];
					vramHistoryMap[g.index] = [...prevVram.slice(-(MAX_CHART_POINTS - 1)), vramPct];
				}
			} else if (tick.gpu_usage !== null && tick.gpu_usage !== undefined) {
				const prevGpu = gpuHistoryMap[0] || [];
				gpuHistoryMap[0] = [...prevGpu.slice(-(MAX_CHART_POINTS - 1)), tick.gpu_usage];

				if (tick.vram_percentage !== null && tick.vram_percentage !== undefined) {
					const prevVram = vramHistoryMap[0] || [];
					vramHistoryMap[0] = [...prevVram.slice(-(MAX_CHART_POINTS - 1)), tick.vram_percentage];
				}
			}
		});
	}

	onMount(async () => {
		loadingStatic = true;
		await hostState.fetchStaticInfo();
		loadingStatic = false;
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
</script>

<svelte:head>
	<title>{t('app.title')} - {t('sysinfo.pageTitle')}</title>
</svelte:head>

<div class="w-full px-4 sm:px-8 py-5 md:py-8 pt-16 md:pt-16 max-w-7xl mx-auto box-border">
	<div class="flex flex-col gap-6">
		<!-- Page Header -->
		<PageHeader
			title={t('sysinfo.pageTitle')}
			subtitle={t('sysinfo.pageSubtitle')}
			icon={Activity}
		/>

		{#if error}
			<Alert variant="error" title={t('common.connectionFailure')}>
				<div class="flex items-center justify-between gap-3">
					<span>{error}</span>
					<Button variant="secondary" size="sm" onclick={reconnectMetricsStream}>
						{t('common.refresh')}
					</Button>
				</div>
			</Alert>
		{/if}

		<!-- Real-Time Telemetry Mountain Charts Section -->
		<SysinfoTelemetryGrid
			{sysinfo}
			{latestTick}
			{cpuHistory}
			{ramHistory}
			{diskIoHistory}
			{netHistory}
			{gpuHistoryMap}
			{vramHistoryMap}
			{gpusList}
		/>

		<!-- Active Models Section -->
		<SysinfoActiveModels
			runningModels={runningModelsList}
			{unloadingIds}
			onUnloadModel={unloadModel}
			onUnloadAll={unloadAllModels}
		/>

		<!-- Static System Info Section -->
		{#if sysinfo}
			<SysinfoHostDetails {sysinfo} />
		{/if}
	</div>
</div>
