<script lang="ts">
	import { MAX_CHART_POINTS } from '$lib';
	import { hostApi } from '$lib/api/host';
	import { Alert, AreaChart, PageHeader } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import {
		getStatusType,
		runningModelsState,
		startRunningStatePolling,
		unloadAllModels,
		unloadModel
	} from '$lib/runningModelsStore';
	import { headerState } from '$lib/state/header.svelte';
	import { hostState } from '$lib/state/host.svelte';
	import type { GpuDeviceInfo, HostMetricsTick, SysInfo } from '$lib/types/host';
	import { formatBytes, formatUptime, splitBytesValue } from '$lib/utils/format';
	import { parseModelDetails } from '$lib/utils/model';
	import {
		Activity,
		Box,
		Cpu,
		Gpu,
		HardDrive,
		Layers,
		MemoryStick,
		Monitor,
		MonitorStop,
		Network,
		Power,
		RefreshCw
	} from '@lucide/svelte';
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

	let currentDiskIoBytes = $derived(
		latestTick ? latestTick.disk_read_bytes_sec + latestTick.disk_write_bytes_sec : 0
	);
	let formattedDiskIo = $derived(splitBytesValue(currentDiskIoBytes, true));

	let currentNetBytes = $derived(
		latestTick ? latestTick.net_rx_bytes_sec + latestTick.net_tx_bytes_sec : 0
	);
	let formattedNet = $derived(splitBytesValue(currentNetBytes, true));

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

	let stopRunningStatePolling: (() => void) | null = null;

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
	<title>Monolai - System Info</title>
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

		<!-- Real-Time Telemetry Mountain Charts Section -->
		<section class="flex flex-col gap-5">
			<div class="grid grid-cols-1 md:grid-cols-2 gap-5 items-stretch">
				<!-- 1. CPU Usage Mountain Chart -->
				<AreaChart
					title={t('sysinfo.cpuTitle')}
					subtitle={sysinfo
						? `${sysinfo.cpu.brand} (${sysinfo.cpu.cores} ${t('sysinfo.cores')})`
						: 'Host Processor'}
					currentValue={latestTick?.cpu_usage ?? sysinfo?.cpu?.usage ?? 0}
					unit="%"
					color="#9845e7"
					icon={Cpu}
					data={cpuHistory}
					maxVal={100}
					temperature={latestTick?.cpu_temp_c ?? sysinfo?.cpu?.temperature_c ?? null}
					clock={latestTick?.cpu_frequency_mhz
						? `${(latestTick.cpu_frequency_mhz / 1000).toFixed(2)} GHz`
						: sysinfo?.cpu?.frequency_mhz
							? `${(sysinfo.cpu.frequency_mhz / 1000).toFixed(2)} GHz`
							: null}
				/>

				<!-- 2. RAM Usage Mountain Chart -->
				<AreaChart
					title={t('sysinfo.ramTitle')}
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

				<!-- 3/4. Multiple GPUs Charts (Render for EVERY GPU: dedicated or integrated) -->
				{#if gpusList.length > 0}
					{#each gpusList as gpu (gpu.index)}
						{@const gpuTitle =
							gpusList.length > 1 ? `GPU ${gpu.index} (${gpu.vendor})` : t('sysinfo.gpuTitle')}
						{@const vramTitle =
							gpusList.length > 1 ? `VRAM (GPU ${gpu.index})` : t('sysinfo.vramTitle')}
						{@const gpuHistory = gpuHistoryMap[gpu.index] || []}
						{@const vramHistory = vramHistoryMap[gpu.index] || []}
						{@const vramTotal = gpu.memory_total_bytes ?? 0}
						{@const vramUsed = gpu.memory_used_bytes ?? 0}
						{@const vramPct = vramTotal > 0 ? (vramUsed / vramTotal) * 100 : 0}

						<!-- GPU Utilization -->
						<AreaChart
							title={gpuTitle}
							subtitle={gpu.name}
							currentValue={gpu.usage ?? 0}
							unit="%"
							color={gpu.is_dedicated ? '#00a971' : '#10b981'}
							icon={gpu.is_dedicated ? Gpu : Monitor}
							data={gpuHistory}
							maxVal={100}
							temperature={gpu.temperature_c ??
								(gpu.is_dedicated
									? null
									: (latestTick?.cpu_temp_c ?? sysinfo?.cpu.temperature_c ?? null))}
							clock={gpu.clock_mhz ? `${gpu.clock_mhz} MHz` : null}
						/>

						<!-- GPU VRAM Usage -->
						<AreaChart
							title={vramTitle}
							subtitle={vramTotal > 0
								? `${formatBytes(vramUsed)} / ${formatBytes(vramTotal)}`
								: gpu.is_dedicated
									? 'Dedicated Video RAM'
									: 'Shared System Video RAM'}
							currentValue={vramPct}
							unit="%"
							color="#30d9a1"
							icon={gpu.is_dedicated ? Layers : MonitorStop}
							data={vramHistory}
							maxVal={100}
							unavailable={vramTotal === 0 && !gpu.is_dedicated}
							unavailableMessage="Shared Host Memory"
						/>
					{/each}
				{:else}
					<!-- Fallback when no GPU is found -->
					<AreaChart
						title={t('sysinfo.gpuTitle')}
						subtitle="Graphics Processor"
						currentValue={0}
						unit="%"
						color="#00a971"
						icon={Gpu}
						data={[]}
						unavailable={true}
						unavailableMessage={t('sysinfo.noGpuDetected')}
					/>

					<AreaChart
						title={t('sysinfo.vramTitle')}
						subtitle="Dedicated Video RAM"
						currentValue={0}
						unit="%"
						color="#30d9a1"
						icon={Layers}
						data={[]}
						unavailable={true}
						unavailableMessage="No Dedicated VRAM Detected"
					/>
				{/if}

				<!-- 5. Local Storage (Disk I/O) Mountain Chart -->
				<AreaChart
					title={t('sysinfo.diskTitle')}
					subtitle={latestTick
						? `${formatBytes(latestTick.storage_used_bytes)} / ${formatBytes(latestTick.storage_total_bytes)} • R: ${formatBytes(latestTick.disk_read_bytes_sec)}/s  W: ${formatBytes(latestTick.disk_write_bytes_sec)}/s`
						: sysinfo?.storage
							? `${formatBytes(sysinfo.storage.used_bytes)} / ${formatBytes(sysinfo.storage.total_bytes)}`
							: 'Storage I/O'}
					currentValue={formattedDiskIo.value}
					unit={formattedDiskIo.unit}
					color="#3b82f6"
					icon={HardDrive}
					data={diskIoHistory}
					maxVal={Math.max(1024, ...diskIoHistory)}
				/>

				<!-- 6. Network Traffic Mountain Chart -->
				<AreaChart
					title={t('sysinfo.networkTitle')}
					subtitle={latestTick
						? `↓ ${formatBytes(latestTick.net_rx_bytes_sec)}/s  ↑ ${formatBytes(latestTick.net_tx_bytes_sec)}/s`
						: 'Network Throughput'}
					currentValue={formattedNet.value}
					unit={formattedNet.unit}
					color="#06b6d4"
					icon={Network}
					data={netHistory}
					maxVal={Math.max(1024, ...netHistory)}
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

				<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
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
				</div>
			</section>
		{/if}
	</div>
</div>
