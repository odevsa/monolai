<script lang="ts">
	import { AreaChart } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import type { GpuDeviceInfo, HostMetricsTick, SysInfo } from '$lib/types/host';
	import { formatBytes, splitBytesValue } from '$lib/utils/format';
	import { Cpu, Gpu, HardDrive, Layers, MemoryStick, Network } from '@lucide/svelte';
	import SysinfoGpuCharts from './SysinfoGpuCharts.svelte';

	interface SysinfoTelemetryGridProps {
		sysinfo: SysInfo | null;
		latestTick: HostMetricsTick | null;
		cpuHistory: number[];
		ramHistory: number[];
		diskIoHistory: number[];
		netHistory: number[];
		gpuHistoryMap: Record<number, number[]>;
		vramHistoryMap: Record<number, number[]>;
		gpusList: GpuDeviceInfo[];
	}

	let {
		sysinfo,
		latestTick,
		cpuHistory,
		ramHistory,
		diskIoHistory,
		netHistory,
		gpuHistoryMap,
		vramHistoryMap,
		gpusList
	}: SysinfoTelemetryGridProps = $props();

	let currentDiskIoBytes = $derived(
		latestTick ? latestTick.disk_read_bytes_sec + latestTick.disk_write_bytes_sec : 0
	);
	let formattedDiskIo = $derived(splitBytesValue(currentDiskIoBytes, true));

	let currentNetBytes = $derived(
		latestTick ? latestTick.net_rx_bytes_sec + latestTick.net_tx_bytes_sec : 0
	);
	let formattedNet = $derived(splitBytesValue(currentNetBytes, true));

	let cpuClock = $derived(
		latestTick?.cpu_frequency_mhz
			? `${(latestTick.cpu_frequency_mhz / 1000).toFixed(2)} GHz`
			: sysinfo?.cpu?.frequency_mhz
				? `${(sysinfo.cpu.frequency_mhz / 1000).toFixed(2)} GHz`
				: null
	);

	let cpuSubtitle = $derived(
		sysinfo
			? `${sysinfo.cpu.brand} (${sysinfo.cpu.cores} ${t('sysinfo.cores')})`
			: t('sysinfo.hostProcessor')
	);

	let ramSubtitle = $derived(
		sysinfo
			? `${formatBytes(latestTick?.ram_used_bytes ?? sysinfo.ram.used_bytes)} / ${formatBytes(sysinfo.ram.total_bytes)}`
			: t('sysinfo.systemMemory')
	);

	let diskSubtitle = $derived(
		latestTick
			? `${formatBytes(latestTick.storage_used_bytes)} / ${formatBytes(latestTick.storage_total_bytes)} • R: ${formatBytes(latestTick.disk_read_bytes_sec)}/s  W: ${formatBytes(latestTick.disk_write_bytes_sec)}/s`
			: sysinfo?.storage
				? `${formatBytes(sysinfo.storage.used_bytes)} / ${formatBytes(sysinfo.storage.total_bytes)}`
				: t('sysinfo.storageIo')
	);

	let netSubtitle = $derived(
		latestTick
			? `↓ ${formatBytes(latestTick.net_rx_bytes_sec)}/s  ↑ ${formatBytes(latestTick.net_tx_bytes_sec)}/s`
			: t('sysinfo.networkThroughput')
	);
</script>

<section class="flex flex-col gap-5">
	<div class="grid grid-cols-1 md:grid-cols-2 gap-5 items-stretch">
		<!-- 1. CPU Usage Mountain Chart -->
		<AreaChart
			title={t('sysinfo.cpuTitle')}
			subtitle={cpuSubtitle}
			currentValue={latestTick?.cpu_usage ?? sysinfo?.cpu?.usage ?? 0}
			unit="%"
			color="var(--chart-cpu)"
			icon={Cpu}
			data={cpuHistory}
			maxVal={100}
			temperature={latestTick?.cpu_temp_c ?? sysinfo?.cpu?.temperature_c ?? null}
			clock={cpuClock}
		/>

		<!-- 2. RAM Usage Mountain Chart -->
		<AreaChart
			title={t('sysinfo.ramTitle')}
			subtitle={ramSubtitle}
			currentValue={latestTick?.ram_percentage ?? sysinfo?.ram?.percentage ?? 0}
			unit="%"
			color="var(--chart-ram)"
			icon={MemoryStick}
			data={ramHistory}
			maxVal={100}
		/>

		<!-- 3/4. Multiple GPUs Charts -->
		{#if gpusList.length > 0}
			{#each gpusList as gpu (gpu.index)}
				<SysinfoGpuCharts
					{gpu}
					gpuHistory={gpuHistoryMap[gpu.index] || []}
					vramHistory={vramHistoryMap[gpu.index] || []}
					isMultiGpu={gpusList.length > 1}
					cpuTemp={latestTick?.cpu_temp_c ?? sysinfo?.cpu?.temperature_c ?? null}
				/>
			{/each}
		{:else}
			<!-- Fallback when no GPU is detected -->
			<AreaChart
				title={t('sysinfo.gpuTitle')}
				subtitle={t('sysinfo.graphicsProcessor')}
				currentValue={0}
				unit="%"
				color="var(--chart-gpu)"
				icon={Gpu}
				data={[]}
				unavailable={true}
				unavailableMessage={t('sysinfo.noGpuDetected')}
			/>

			<AreaChart
				title={t('sysinfo.vramTitle')}
				subtitle={t('sysinfo.dedicatedVram')}
				currentValue={0}
				unit="%"
				color="var(--chart-vram)"
				icon={Layers}
				data={[]}
				unavailable={true}
				unavailableMessage={t('sysinfo.noDedicatedVram')}
			/>
		{/if}

		<!-- 5. Local Storage (Disk I/O) Mountain Chart -->
		<AreaChart
			title={t('sysinfo.diskTitle')}
			subtitle={diskSubtitle}
			currentValue={formattedDiskIo.value}
			unit={formattedDiskIo.unit}
			color="var(--chart-disk)"
			icon={HardDrive}
			data={diskIoHistory}
			maxVal={Math.max(1024, ...diskIoHistory)}
		/>

		<!-- 6. Network Traffic Mountain Chart -->
		<AreaChart
			title={t('sysinfo.networkTitle')}
			subtitle={netSubtitle}
			currentValue={formattedNet.value}
			unit={formattedNet.unit}
			color="var(--chart-net)"
			icon={Network}
			data={netHistory}
			maxVal={Math.max(1024, ...netHistory)}
		/>
	</div>
</section>
