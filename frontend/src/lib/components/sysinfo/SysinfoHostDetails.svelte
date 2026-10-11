<script lang="ts">
	import { t } from '$lib/i18n';
	import type { SysInfo } from '$lib/types/host';
	import { formatUptime } from '$lib/utils/format';
	import { Monitor } from '@lucide/svelte';
	import SysinfoDetailTile from './SysinfoDetailTile.svelte';

	interface SysinfoHostDetailsProps {
		sysinfo: SysInfo;
	}

	let { sysinfo }: SysinfoHostDetailsProps = $props();

	let details = $derived([
		{
			label: t('sysinfo.osTitle'),
			value: `${sysinfo.os.name} ${sysinfo.os.os_version}`
		},
		{
			label: t('sysinfo.kernelVersion'),
			value: sysinfo.os.kernel_version
		},
		{
			label: t('sysinfo.hostname'),
			value: sysinfo.os.hostname
		},
		{
			label: t('sysinfo.uptimeTitle'),
			value: formatUptime(sysinfo.os.uptime_seconds)
		}
	]);
</script>

<section class="bg-[var(--bg-surface)] rounded-2xl p-6 flex flex-col gap-5 border-0 shadow-xs">
	<div class="flex items-center gap-3">
		<div class="flex items-center gap-2">
			<Monitor size={16} class="text-[var(--primary)]" />
			<h3 class="app-card-title text-sm">{t('sysinfo.hostDetails')}</h3>
		</div>
	</div>

	<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
		{#each details as item (item.label)}
			<SysinfoDetailTile label={item.label} value={item.value} />
		{/each}
	</div>
</section>
