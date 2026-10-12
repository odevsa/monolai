<script lang="ts">
	import { Badge, Input } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import { Network } from '@lucide/svelte';

	let {
		isDocker = false,
		isStorageConfigured = false,
		host = $bindable('0.0.0.0'),
		port = $bindable(8080)
	}: {
		isDocker?: boolean;
		isStorageConfigured?: boolean;
		host?: string;
		port?: number;
	} = $props();
</script>

<div
	class="flex flex-col gap-3 p-4 rounded-xl border border-[var(--border-color)] bg-[var(--bg-surface-hover)]/30"
>
	<div class="flex items-center justify-between">
		<span
			class="flex items-center gap-2 text-xs font-bold uppercase tracking-wider text-[var(--text-secondary)]"
		>
			<Network size={15} class="text-[var(--primary)]" />
			<span>
				{t(isStorageConfigured ? 'setup.networkAndPort' : 'setup.stepNetwork')}
			</span>
		</span>
		<span class="text-[11px] text-[var(--text-muted)] font-mono">
			{t('setup.defaultNetworkHint')}
		</span>
	</div>

	<div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
		<div class="flex flex-col gap-1.5">
			<div class="flex items-center justify-between">
				<label for="host-input" class="text-xs font-semibold text-[var(--text-secondary)]">
					{t('common.host')}
				</label>
				{#if isDocker}
					<Badge variant="info" rounded="rounded-sm">
						{t('setup.lockedInDocker')}
					</Badge>
				{/if}
			</div>
			<Input
				id="host-input"
				type="text"
				bind:value={host}
				disabled={isDocker}
				placeholder="0.0.0.0"
				class="font-mono text-xs"
			/>
		</div>

		<div class="flex flex-col gap-1.5">
			<div class="flex items-center justify-between">
				<label for="port-input" class="text-xs font-semibold text-[var(--text-secondary)]">
					{t('setup.httpPort')}
				</label>
				{#if isDocker}
					<Badge variant="info" rounded="rounded-sm">
						{t('setup.lockedInDocker')}
					</Badge>
				{/if}
			</div>
			<Input
				id="port-input"
				type="number"
				bind:value={port}
				disabled={isDocker}
				placeholder="8080"
				class="font-mono text-xs"
			/>
		</div>
	</div>
</div>
