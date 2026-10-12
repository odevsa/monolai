<script lang="ts">
	import { Badge, Button } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import { getStatusType } from '$lib/runningModelsStore';
	import type { RunningModelStatus } from '$lib/types/models';
	import { formatUptime } from '$lib/utils/format';
	import { parseModelDetails } from '$lib/utils/model';
	import { Box, Boxes, CircleAlert, Clock, Cpu, Network, Power, RefreshCw } from '@lucide/svelte';

	interface SysinfoModelCardProps {
		model: RunningModelStatus;
		isUnloading: boolean;
		onUnload: (modelId: string) => void;
	}

	let { model, isUnloading, onUnload }: SysinfoModelCardProps = $props();

	let rawStatus = $derived(getStatusType(model.state));
	let statusType = $derived(isUnloading ? 'loading' : rawStatus);
	let parsed = $derived(parseModelDetails(model.model_id));
	let modelDisplayName = $derived(parsed.name || model.model_id);

	let statusLabel = $derived(
		statusType === 'ready'
			? t('sysinfo.statusReady')
			: statusType === 'loading'
				? t('sysinfo.statusLoading')
				: t('sysinfo.statusError')
	);
</script>

<div
	class="flex flex-col justify-between gap-2 p-4.5 rounded-xl bg-[var(--bg-surface-hover)] border border-[var(--border-color)] hover:border-[var(--border-hover)] transition-all shadow-xs"
>
	<!-- Top Row: Identity and Status -->
	<div class="flex items-start justify-between gap-3 min-w-0">
		<div class="flex items-center gap-2.5 min-w-0 flex-1">
			<Box size={32} class="text-[var(--text-primary)]" />

			<div class="min-w-0 flex-1">
				<div class="flex items-center gap-2 flex-wrap">
					<h4
						class="m-0 text-sm font-bold text-[var(--text-primary)] truncate"
						title={modelDisplayName}
					>
						{modelDisplayName}
					</h4>
				</div>

				<div class="text-xs text-[var(--text-muted)] truncate mt-0.5" title={model.model_id}>
					{model.model_id}
				</div>
			</div>
		</div>

		<!-- Status Badge -->
		<Badge
			variant={statusType === 'ready' ? 'success' : statusType === 'loading' ? 'warning' : 'danger'}
			pulse={statusType === 'ready'}
		>
			{statusLabel}
		</Badge>
	</div>

	<!-- Tags (if any) -->
	{#if parsed.tags.length > 0}
		<div class="flex items-center gap-1.5 flex-wrap -mt-1">
			{#each parsed.tags as tag}
				<Badge>
					{tag}
				</Badge>
			{/each}
		</div>
	{/if}

	<!-- Telemetry Chips Grid -->
	<div
		class="grid grid-cols-4 gap-2 p-2.5 rounded-lg bg-[var(--bg-surface)] border border-[var(--border-color)]"
	>
		<!-- Runtime ID -->
		<div class="flex flex-col items-center justify-center text-center">
			<span
				class="flex items-center gap-1 text-[0.65rem] uppercase font-semibold text-[var(--text-muted)] tracking-wider"
			>
				<Boxes size={11} class="text-[var(--text-muted)]" />
				<span>{t('sysinfo.tableRuntime')}</span>
			</span>
			<span class="text-xs font-mono font-bold text-[var(--text-primary)] mt-1">
				{model.runtime_id}
			</span>
		</div>

		<!-- PID -->
		<div
			class="flex flex-col items-center justify-center text-center border-x border-[var(--border-color)]"
		>
			<span
				class="flex items-center gap-1 text-[0.65rem] uppercase font-semibold text-[var(--text-muted)] tracking-wider"
			>
				<Cpu size={11} class="text-[var(--text-muted)]" />
				<span>{t('sysinfo.tablePid')}</span>
			</span>
			<span class="text-xs font-mono font-bold text-[var(--text-primary)] mt-1">
				{model.pid > 0 ? `#${model.pid}` : '—'}
			</span>
		</div>

		<!-- Port -->
		<div
			class="flex flex-col items-center justify-center text-center border-x border-[var(--border-color)]"
		>
			<span
				class="flex items-center gap-1 text-[0.65rem] uppercase font-semibold text-[var(--text-muted)] tracking-wider"
			>
				<Network size={11} class="text-[var(--text-muted)]" />
				<span>{t('sysinfo.tablePort')}</span>
			</span>
			<span class="text-xs font-mono font-bold text-[var(--text-primary)] mt-1">
				{model.port > 0 ? `:${model.port}` : '—'}
			</span>
		</div>

		<!-- Idle -->
		<div class="flex flex-col items-center justify-center text-center">
			<span
				class="flex items-center gap-1 text-[0.65rem] uppercase font-semibold text-[var(--text-muted)] tracking-wider"
			>
				<Clock size={11} class="text-[var(--text-muted)]" />
				<span>{t('sysinfo.tableIdle')}</span>
			</span>
			<span class="text-xs font-mono font-bold text-[var(--text-primary)] mt-1">
				{formatUptime(model.idle_seconds)}
			</span>
		</div>
	</div>

	<!-- Error Banner (if any) -->
	{#if typeof model.state === 'object' && model.state.status === 'Error'}
		<div
			class="text-[0.725rem] text-[var(--danger-text)] bg-[var(--danger-light)] border border-[var(--danger-border)] px-3 py-2 rounded-lg flex items-start gap-2"
		>
			<CircleAlert size={14} class="shrink-0 mt-0.5" />
			<span>{model.state.message || t('sysinfo.modelProcessFailed')}</span>
		</div>
	{/if}

	<!-- Card Footer Action -->
	<div class="flex items-center justify-end pt-1 border-t border-[var(--border-color)]">
		<Button
			variant="danger"
			disabled={isUnloading}
			onclick={() => onUnload(model.model_id)}
			title={t('sysinfo.unloadModelTitle', { model: modelDisplayName })}
			ariaLabel={t('sysinfo.unloadModelTitle', { model: modelDisplayName })}
		>
			{#if isUnloading}
				<RefreshCw size={13} class="animate-spin text-[var(--warning-text)]" />
				<span>{t('sysinfo.statusLoading')}</span>
			{:else}
				<Power size={13} />
				<span>{t('models.unloadModel')}</span>
			{/if}
		</Button>
	</div>
</div>
