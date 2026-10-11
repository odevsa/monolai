<script lang="ts">
	import { Badge } from '$lib/components/ui';
	import type { LogEntry } from '$lib/types/logs';

	interface LogsEntryRowProps {
		entry: LogEntry;
		showModelTag: boolean;
	}

	let { entry, showModelTag }: LogsEntryRowProps = $props();

	function parseServiceInfo(rawService: string): { runtime: string; model: string | null } {
		if (!rawService) return { runtime: '', model: null };
		const parts = rawService.split(':');
		if (parts.length >= 2) {
			return {
				runtime: parts[0].trim(),
				model: parts.slice(1).join(':').trim()
			};
		}
		return {
			runtime: rawService.trim(),
			model: null
		};
	}

	function getLevelBadgeVariant(level: string): 'danger' | 'warning' | 'info' | 'muted' {
		switch (level.toUpperCase()) {
			case 'ERROR':
				return 'danger';
			case 'WARN':
			case 'WARNING':
				return 'warning';
			case 'DEBUG':
				return 'muted';
			case 'INFO':
			default:
				return 'info';
		}
	}

	function getMessageColor(level: string): string {
		switch (level.toUpperCase()) {
			case 'ERROR':
				return 'text-[var(--danger-text)] font-medium';
			case 'WARN':
			case 'WARNING':
				return 'text-[var(--warning-text)]';
			case 'DEBUG':
				return 'text-[var(--text-secondary)]';
			case 'INFO':
			default:
				return 'text-[var(--text-primary)]';
		}
	}

	function getServiceBadgeClasses(runtime: string): string {
		const s = runtime.toLowerCase();
		if (s === 'monolai') {
			return 'bg-[var(--primary-light)] text-[var(--primary)] border-[var(--primary-focus)]';
		}
		if (s.includes('llama')) {
			return 'bg-[var(--success-light)] text-[var(--success-text)] border-[var(--success-border)]';
		}
		if (s.includes('sd') || s.includes('diffus') || s.includes('stable')) {
			return 'bg-[var(--warning-light)] text-[var(--warning-text)] border-[var(--warning-border)]';
		}
		return 'bg-[var(--info-light)] text-[var(--info-text)] border-[var(--info-border)]';
	}

	function formatTime(iso: string): string {
		try {
			const d = new Date(iso);
			if (isNaN(d.getTime())) return iso;
			return d.toLocaleTimeString(undefined, {
				hour12: false,
				hour: '2-digit',
				minute: '2-digit',
				second: '2-digit',
				fractionalSecondDigits: 3
			});
		} catch {
			return iso;
		}
	}

	let info = $derived(parseServiceInfo(entry.service));
	let levelVariant = $derived(getLevelBadgeVariant(entry.level));
	let messageColor = $derived(getMessageColor(entry.level));
	let serviceClasses = $derived(getServiceBadgeClasses(info.runtime));
	let formattedTime = $derived(formatTime(entry.timestamp));
</script>

<div
	class="flex items-baseline gap-2.5 hover:bg-[var(--bg-hover)] px-2 py-0.5 rounded transition-colors group min-w-0"
>
	<!-- Timestamp -->
	<span class="text-[var(--text-muted)] shrink-0 font-mono text-[10.5px] tabular-nums select-none">
		{formattedTime}
	</span>

	<!-- Level Badge -->
	<Badge variant={levelVariant} rounded="rounded-sm" class="text-[9.5px] px-1 py-0 uppercase">
		{entry.level}
	</Badge>

	<!-- Runtime Service Pill -->
	<span
		class="shrink-0 px-1.5 py-0.5 rounded text-[10px] font-semibold border {serviceClasses}"
		title={info.runtime}
	>
		{info.runtime}
	</span>

	<!-- Optional Model Tag -->
	{#if showModelTag && info.model}
		<span
			class="shrink-0 max-w-[180px] sm:max-w-[240px] truncate text-ellipsis whitespace-nowrap px-1.5 py-0.5 rounded text-[10px] font-mono border border-[var(--border-color)] bg-[var(--bg-surface)] text-[var(--text-secondary)]"
			title={info.model}
		>
			{info.model}
		</span>
	{/if}

	<!-- Log Content Message -->
	<span class="flex-1 min-w-0 whitespace-pre-wrap break-all leading-normal {messageColor}">
		{entry.message}
	</span>
</div>
