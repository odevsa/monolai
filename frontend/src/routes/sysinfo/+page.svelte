<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { Activity, Box, Cpu, HardDrive, Monitor, Power, RefreshCw, Zap } from '@lucide/svelte';
	import { MAX_CHART_POINTS } from '$lib';
	import AreaChart from '$lib/components/AreaChart.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import { setSysinfoStatus } from '$lib/headerStore';
	import { formatBytes, formatUptime } from '$lib/utils/format';
	import { parseModelDetails } from '$lib/utils/model';
	import {
		runningModels,
		unloadingModelIds,
		unloadModel,
		unloadAllModels,
		getStatusType,
		startRunningStatePolling
	} from '$lib/runningModelsStore';

	interface CpuInfo {
		usage: number;
		cores: number;
		brand: string;
		frequency_mhz: number;
	}

	interface RamInfo {
		total_bytes: number;
		used_bytes: number;
		free_bytes: number;
		percentage: number;
	}

	interface OsInfo {
		name: string;
		kernel_version: string;
		os_version: string;
		hostname: string;
		uptime_seconds: number;
	}

	interface SysInfo {
		cpu: CpuInfo;
		ram: RamInfo;
		os: OsInfo;
		gpu: {
			name: string;
			vendor: string;
			memory_total_bytes?: number | null;
			driver_version?: string | null;
			is_dedicated: boolean;
		} | null;
		timestamp: number;
	}

	interface HostMetricsTick {
		cpu_usage: number;
		ram_used_bytes: number;
		ram_total_bytes: number;
		ram_free_bytes: number;
		ram_percentage: number;
		gpu_usage: number | null;
		timestamp: number;
	}

	let sysinfo = $state<SysInfo | null>(null);
	let loadingStatic = $state(true);
	let sseConnected = $state(false);
	let error = $state<string | null>(null);

	let latestTick = $state<HostMetricsTick | null>(null);
	let cpuHistory = $state<number[]>([]);
	let ramHistory = $state<number[]>([]);
	let gpuHistory = $state<number[]>([]);

	let eventSource: EventSource | null = null;

	async function fetchStaticHostInfo() {
		try {
			loadingStatic = true;
			const res = await fetch('/api/host');
			if (!res.ok) throw new Error(`HTTP error! status: ${res.status}`);
			sysinfo = await res.json();
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
		if (eventSource) {
			eventSource.close();
		}

		eventSource = new EventSource('/api/host/usage');

		eventSource.onopen = () => {
			sseConnected = true;
			error = null;
			setSysinfoStatus(true, reconnectMetricsStream);
		};

		eventSource.onmessage = (event) => {
			try {
				const tick: HostMetricsTick = JSON.parse(event.data);
				latestTick = tick;

				cpuHistory = [...cpuHistory.slice(-(MAX_CHART_POINTS - 1)), tick.cpu_usage];
				ramHistory = [...ramHistory.slice(-(MAX_CHART_POINTS - 1)), tick.ram_percentage];

				if (tick.gpu_usage !== null && tick.gpu_usage !== undefined) {
					gpuHistory = [...gpuHistory.slice(-(MAX_CHART_POINTS - 1)), tick.gpu_usage];
				}
			} catch (err) {
				console.error('Failed to parse SSE metrics tick:', err);
			}
		};

		eventSource.onerror = (err) => {
			console.warn('SSE connection lost or error:', err);
			sseConnected = false;
			setSysinfoStatus(false, reconnectMetricsStream);
		};
	}

	let stopRunningStatePolling: (() => void) | null = null;

	onMount(() => {
		fetchStaticHostInfo();
		connectMetricsStream();
		stopRunningStatePolling = startRunningStatePolling(3000);
	});

	onDestroy(() => {
		if (eventSource) {
			eventSource.close();
			eventSource = null;
		}
		if (stopRunningStatePolling) {
			stopRunningStatePolling();
			stopRunningStatePolling = null;
		}
	});

	let isGpuUnavailable = $derived.by(() => {
		if (sysinfo && sysinfo.gpu === null) return true;
		if (!sysinfo && latestTick && latestTick.gpu_usage === null) return true;
		return false;
	});
</script>

<svelte:head>
	<title>Monolai - System Info</title>
</svelte:head>

<div class="page-container">
	<div class="page-inner gap-8">
		<PageHeader
			title="System Info"
			subtitle="Real-time resource utilization streaming"
			icon={Activity}
		/>

		{#if error}
			<div
				class="flex items-center justify-between p-4 rounded-xl bg-[var(--bg-surface)] border-0 text-[var(--text-primary)] text-xs"
			>
				<div>
					<strong class="text-red-500">Backend Connection Failure:</strong>
					{error}
				</div>
				<button
					type="button"
					class="px-3 py-1.5 text-xs font-medium border-0 bg-[var(--btn-bg)] text-[var(--btn-text)] rounded-lg cursor-pointer transition-colors hover:bg-[var(--btn-hover)]"
					onclick={() => {
						fetchStaticHostInfo();
						connectMetricsStream();
					}}
				>
					Retry Connection
				</button>
			</div>
		{/if}

		<!-- Real-Time Mountain Charts Section -->
		<section class="flex flex-col gap-4">
			<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5 items-stretch">
				<!-- CPU Usage Mountain Chart -->
				<AreaChart
					title="CPU Utilization"
					subtitle={sysinfo
						? `${sysinfo.cpu.brand} (${sysinfo.cpu.cores} Cores)`
						: 'Host Processor'}
					currentValue={latestTick ? latestTick.cpu_usage : (sysinfo?.cpu.usage ?? 0)}
					unit="%"
					color="#38bdf8"
					icon={Cpu}
					data={cpuHistory}
					maxVal={100}
				/>

				<!-- RAM Memory Mountain Chart -->
				<AreaChart
					title="RAM Memory Usage"
					subtitle={sysinfo
						? `${formatBytes(latestTick?.ram_used_bytes ?? sysinfo.ram.used_bytes)} / ${formatBytes(sysinfo.ram.total_bytes)}`
						: 'Physical RAM'}
					currentValue={latestTick ? latestTick.ram_percentage : (sysinfo?.ram.percentage ?? 0)}
					unit="%"
					color="#34d399"
					icon={HardDrive}
					data={ramHistory}
					maxVal={100}
				/>

				<!-- GPU Usage Mountain Chart -->
				<AreaChart
					title="GPU Engine"
					subtitle={!isGpuUnavailable ? (sysinfo?.gpu?.name || 'Dedicated Acceleration') : 'No Dedicated GPU Detected'}
					currentValue={latestTick?.gpu_usage ?? 0}
					unit="%"
					color="#a855f7"
					icon={Zap}
					data={gpuHistory}
					maxVal={100}
					unavailable={isGpuUnavailable}
					unavailableMessage="No Dedicated GPU Detected"
				/>
			</div>
		</section>

		<!-- Loaded Models Section -->
		<section class="bg-[var(--bg-surface)] rounded-2xl p-6 flex flex-col gap-5 border-0 shadow-sm">
			<div class="flex items-center justify-between flex-wrap gap-3">
				<div class="flex items-center gap-3">
					<div
						class="w-9 h-9 rounded-xl bg-[var(--primary)]/15 text-[var(--primary)] flex items-center justify-center shrink-0"
					>
						<Box size={18} />
					</div>
					<div>
						<div class="flex items-center gap-2">
							<h3 class="m-0 text-base font-bold text-[var(--text-primary)]">Loaded Models</h3>
							{#if $runningModels.length > 0}
								<span
									class="text-[0.7rem] font-bold px-2 py-0.5 rounded-full bg-emerald-500/15 text-emerald-400 border border-emerald-500/30 flex items-center gap-1.5"
								>
									<span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span>
									{$runningModels.length}
									{$runningModels.length === 1 ? 'model' : 'models'} active
								</span>
							{:else}
								<span
									class="text-[0.7rem] font-medium px-2 py-0.5 rounded-full bg-white/5 text-[var(--text-muted)] border border-[var(--border-color)]"
								>
									0 active
								</span>
							{/if}
						</div>
						<p class="m-0 text-xs text-[var(--text-muted)] mt-0.5">
							Active upstream inference processes currently running in memory
						</p>
					</div>
				</div>

				{#if $runningModels.length > 0}
					<button
						type="button"
						class="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-semibold bg-red-500/10 text-red-400 border border-red-500/25 rounded-lg cursor-pointer transition-all hover:bg-red-500/20 hover:border-red-500/40 active:scale-95 disabled:opacity-50 disabled:cursor-not-allowed"
						onclick={() => unloadAllModels()}
						title="Unload all running models from memory"
					>
						<Power size={14} />
						<span>Unload All</span>
					</button>
				{/if}
			</div>

			{#if $runningModels.length === 0}
				<div
					class="flex flex-col items-center justify-center gap-2.5 py-8 px-4 rounded-xl border border-dashed border-[var(--border-color)] bg-black/10 text-center"
				>
					<div
						class="w-10 h-10 rounded-xl bg-white/5 text-[var(--text-muted)] flex items-center justify-center"
					>
						<Box size={20} class="opacity-60" />
					</div>
					<div class="flex flex-col gap-0.5">
						<span class="text-xs font-semibold text-[var(--text-primary)]"
							>No Models Currently Loaded</span
						>
						<span class="text-[0.75rem] text-[var(--text-muted)] max-w-md">
							Inference processes spawn automatically when a chat or OpenAI API request is made, and
							auto-unload when idle.
						</span>
					</div>
				</div>
			{:else}
				<div class="grid grid-cols-1 md:grid-cols-2 gap-3.5">
					{#each $runningModels as model (model.model_id)}
						{@const isUnloading = $unloadingModelIds.has(model.model_id)}
						{@const rawStatus = getStatusType(model.state)}
						{@const statusType = isUnloading ? 'loading' : rawStatus}
						{@const parsed = parseModelDetails(model.model_id)}
						{@const modelDisplayName = parsed.name || model.model_id}
						<div
							class="flex flex-col justify-between gap-3 p-4 rounded-xl bg-white/[0.03] border-0 transition-all hover:bg-white/[0.06]"
						>
							<!-- Top row: icons only -->
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

									<!-- Runtime on a line below badges -->
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

							<!-- Model Name in a single bold line -->
							<div class="min-w-0">
								<h4
									class="m-0 text-sm font-bold text-[var(--text-primary)] truncate"
									title={model.model_id}
								>
									{modelDisplayName}
								</h4>
							</div>

							<!-- Badges on their own line -->
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

							<!-- Metrics: PID, Port, Idle formatted as a table -->
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
			<section class="bg-[var(--bg-surface)] rounded-2xl p-6 flex flex-col gap-5 border-0">
				<div class="flex items-center gap-3">
					<div
						class="w-9 h-9 rounded-xl bg-blue-500/15 text-blue-400 flex items-center justify-center shrink-0"
					>
						<Monitor size={18} />
					</div>
					<div>
						<h3 class="m-0 text-base font-bold text-[var(--text-primary)]">Host System Details</h3>
					</div>
				</div>

				<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 {sysinfo.gpu ? 'xl:grid-cols-5' : ''} gap-4">
					<div class="flex flex-col gap-1.5 p-3.5 bg-white/[0.02] rounded-xl border-0">
						<span
							class="text-[0.75rem] font-semibold uppercase tracking-wider text-[var(--text-muted)]"
							>Operating System</span
						>
						<span class="text-sm font-semibold text-[var(--text-primary)]"
							>{sysinfo.os.name} {sysinfo.os.os_version}</span
						>
					</div>

					<div class="flex flex-col gap-1.5 p-3.5 bg-white/[0.02] rounded-xl border-0">
						<span
							class="text-[0.75rem] font-semibold uppercase tracking-wider text-[var(--text-muted)]"
							>Kernel Version</span
						>
						<span class="text-sm font-semibold text-[var(--text-primary)]"
							>{sysinfo.os.kernel_version}</span
						>
					</div>

					<div class="flex flex-col gap-1.5 p-3.5 bg-white/[0.02] rounded-xl border-0">
						<span
							class="text-[0.75rem] font-semibold uppercase tracking-wider text-[var(--text-muted)]"
							>Hostname</span
						>
						<span class="text-sm font-semibold text-[var(--text-primary)]"
							>{sysinfo.os.hostname}</span
						>
					</div>

					<div class="flex flex-col gap-1.5 p-3.5 bg-white/[0.02] rounded-xl border-0">
						<span
							class="text-[0.75rem] font-semibold uppercase tracking-wider text-[var(--text-muted)]"
							>System Uptime</span
						>
						<span class="text-sm font-semibold text-[var(--text-primary)]"
							>{formatUptime(sysinfo.os.uptime_seconds)}</span
						>
					</div>

					{#if sysinfo.gpu}
						<div class="flex flex-col gap-1.5 p-3.5 bg-white/[0.02] rounded-xl border-0">
							<span
								class="text-[0.75rem] font-semibold uppercase tracking-wider text-[var(--text-muted)]"
								>Dedicated GPU</span
							>
							<span
								class="text-sm font-semibold text-[var(--text-primary)] truncate"
								title={sysinfo.gpu.name}
								>{sysinfo.gpu.name}</span
							>
						</div>
					{/if}
				</div>
			</section>
		{/if}
	</div>
</div>
