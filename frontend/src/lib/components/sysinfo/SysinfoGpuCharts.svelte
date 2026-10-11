<script lang="ts">
	import { AreaChart } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import type { GpuDeviceInfo } from '$lib/types/host';
	import { formatBytes } from '$lib/utils/format';
	import { Gpu, Layers, Monitor, MonitorStop } from '@lucide/svelte';

	interface SysinfoGpuChartsProps {
		gpu: GpuDeviceInfo;
		gpuHistory?: number[];
		vramHistory?: number[];
		isMultiGpu?: boolean;
		cpuTemp?: number | null;
	}

	let {
		gpu,
		gpuHistory = [],
		vramHistory = [],
		isMultiGpu = false,
		cpuTemp = null
	}: SysinfoGpuChartsProps = $props();

	let gpuTitle = $derived(isMultiGpu ? `GPU ${gpu.index} (${gpu.vendor})` : t('sysinfo.gpuTitle'));
	let vramTitle = $derived(isMultiGpu ? `VRAM (GPU ${gpu.index})` : t('sysinfo.vramTitle'));

	let vramTotal = $derived(gpu.memory_total_bytes ?? 0);
	let vramUsed = $derived(gpu.memory_used_bytes ?? 0);
	let vramPct = $derived(vramTotal > 0 ? (vramUsed / vramTotal) * 100 : 0);

	let vramSubtitle = $derived(
		vramTotal > 0
			? `${formatBytes(vramUsed)} / ${formatBytes(vramTotal)}`
			: gpu.is_dedicated
				? t('sysinfo.dedicatedVram')
				: t('sysinfo.sharedVram')
	);

	let effectiveTemperature = $derived(
		gpu.temperature_c ?? (gpu.is_dedicated ? null : (cpuTemp ?? null))
	);
</script>

<!-- GPU Utilization -->
<AreaChart
	title={gpuTitle}
	subtitle={gpu.name}
	currentValue={gpu.usage ?? 0}
	unit="%"
	color={gpu.is_dedicated ? 'var(--chart-gpu)' : 'var(--chart-gpu-shared)'}
	icon={gpu.is_dedicated ? Gpu : Monitor}
	data={gpuHistory}
	maxVal={100}
	temperature={effectiveTemperature}
	clock={gpu.clock_mhz ? `${gpu.clock_mhz} MHz` : null}
/>

<!-- GPU VRAM Usage -->
<AreaChart
	title={vramTitle}
	subtitle={vramSubtitle}
	currentValue={vramPct}
	unit="%"
	color="var(--chart-vram)"
	icon={gpu.is_dedicated ? Layers : MonitorStop}
	data={vramHistory}
	maxVal={100}
	unavailable={vramTotal === 0 && !gpu.is_dedicated}
	unavailableMessage={t('sysinfo.sharedHostMemory')}
/>
