<script lang="ts">
	import { HardwareMetricBar } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import { hostState } from '$lib/state';
	import { formatBytes } from '$lib/utils/format';
	import { Cpu, Gpu, HardDrive, Layers, MemoryStick, Monitor } from '@lucide/svelte';

	$effect(() => {
		const unsub = hostState.subscribe();
		return unsub;
	});

	let sysinfo = $derived(hostState.sysinfo);
	let latestTick = $derived(hostState.latestTick);

	let cpuUsage = $derived(latestTick?.cpu_usage ?? sysinfo?.cpu.usage ?? 0);
	let cpuBrand = $derived(sysinfo?.cpu.brand || t('sysinfo.hostProcessor'));

	let ramPercentage = $derived(latestTick?.ram_percentage ?? sysinfo?.ram.percentage ?? 0);
	let ramTotal = $derived(latestTick?.ram_total_bytes ?? sysinfo?.ram.total_bytes ?? 0);

	let gpusList = $derived.by(() => {
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

	let totalVramBytes = $derived.by(() => {
		if (latestTick?.vram_total_bytes && latestTick.vram_total_bytes > 0)
			return latestTick.vram_total_bytes;
		if (sysinfo?.vram?.total_bytes && sysinfo.vram.total_bytes > 0) return sysinfo.vram.total_bytes;
		let sum = 0;
		for (const g of gpusList) {
			if (g.memory_total_bytes) sum += g.memory_total_bytes;
		}
		return sum;
	});

	let usedVramBytes = $derived.by(() => {
		if (latestTick?.vram_used_bytes !== null && latestTick?.vram_used_bytes !== undefined)
			return latestTick.vram_used_bytes;
		if (sysinfo?.vram?.used_bytes) return sysinfo.vram.used_bytes;
		let sum = 0;
		for (const g of gpusList) {
			if (g.memory_used_bytes) sum += g.memory_used_bytes;
		}
		return sum;
	});

	let vramPercentage = $derived(totalVramBytes > 0 ? (usedVramBytes / totalVramBytes) * 100 : 0);

	let storageTotal = $derived(latestTick?.storage_total_bytes ?? sysinfo?.storage.total_bytes ?? 0);
	let storagePercentage = $derived(
		latestTick?.storage_percentage ?? sysinfo?.storage.percentage ?? 0
	);
</script>

<div class="flex flex-col gap-3">
	<div class="text-[0.75rem] font-bold text-[var(--text-muted)] uppercase tracking-wider px-1">
		{t('navigation.systemStatus')}
	</div>

	<div
		class="bg-[var(--bg-surface)] border border-[var(--border-color)] rounded-xl p-3 flex flex-col gap-3.5"
	>
		<!-- CPU Status -->
		<HardwareMetricBar
			label="CPU"
			detail={cpuBrand}
			value={cpuUsage}
			color="var(--chart-cpu)"
			icon={Cpu}
		/>

		<!-- RAM Status -->
		<HardwareMetricBar
			label="RAM"
			detail={formatBytes(ramTotal)}
			value={ramPercentage}
			color="var(--chart-ram)"
			icon={MemoryStick}
		/>

		<!-- GPUs Status -->
		{#if gpusList.length > 0}
			{#each gpusList as gpu (gpu.index)}
				{@const gpuLabel = gpu.is_dedicated ? 'GPU' : 'Video'}
				{@const gpuColor = gpu.is_dedicated ? 'var(--chart-gpu)' : 'var(--chart-gpu-shared)'}
				<HardwareMetricBar
					label={gpusList.length > 1 ? `${gpuLabel} ${gpu.index}` : gpuLabel}
					detail={gpu.name}
					value={gpu.usage}
					color={gpuColor}
					icon={gpu.is_dedicated ? Gpu : Monitor}
				/>
			{/each}
		{/if}

		<!-- VRAM Status -->
		{#if totalVramBytes > 0}
			<HardwareMetricBar
				label="VRAM"
				detail={formatBytes(totalVramBytes)}
				value={vramPercentage}
				color="var(--chart-vram)"
				icon={Layers}
			/>
		{/if}

		<!-- Storage (Disk) Status -->
		<HardwareMetricBar
			label={t('common.disk')}
			detail={formatBytes(storageTotal)}
			value={storagePercentage}
			color="var(--chart-disk)"
			icon={HardDrive}
		/>
	</div>
</div>
