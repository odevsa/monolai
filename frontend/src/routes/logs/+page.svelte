<script lang="ts">
	import { logsApi } from '$lib/api/logs';
	import { PageHeader, Select } from '$lib/components/ds';
	import type { SelectOption } from '$lib/components/ds/molecules/Select.svelte';
	import { askConfirm } from '$lib/confirmStore';
	import { t } from '$lib/i18n';
	import { headerState } from '$lib/state/header.svelte';
	import type { LogEntry } from '$lib/types/logs';
	import {
		ArrowDown,
		Box,
		ChevronDown,
		Download,
		Filter,
		RefreshCw,
		ScrollText,
		Search,
		Terminal,
		Trash2
	} from '@lucide/svelte';
	import { onDestroy, onMount, tick } from 'svelte';

	const MAX_DISPLAY_LINES = 1500;

	let logEntries = $state<LogEntry[]>([]);
	let availableServices = $state<string[]>(['all', 'monolai']);
	let selectedService = $state('all');
	let selectedLevel = $state('ALL');
	let searchQuery = $state('');
	let autoScroll = $state(true);
	let isScrolledUp = $state(false);
	let showModelTag = $state(false);
	let isLoading = $state(true);
	let isConnected = $state(false);

	let terminalContainer = $state<HTMLDivElement | null>(null);
	let unsubscribeStream: (() => void) | null = null;

	let serviceOptions = $derived<SelectOption[]>(
		availableServices.map((svc) => ({
			value: svc,
			label: svc === 'all' ? t('logs.allServices') : svc
		}))
	);

	const levelOptions: SelectOption[] = [
		{ value: 'ALL', label: 'All Levels' },
		{ value: 'INFO', label: 'INFO' },
		{ value: 'WARN', label: 'WARN' },
		{ value: 'ERROR', label: 'ERROR' },
		{ value: 'DEBUG', label: 'DEBUG' }
	];

	let filteredEntries = $derived.by(() => {
		let list = logEntries;
		if (selectedService && selectedService !== 'all') {
			const target = selectedService.toLowerCase();
			list = list.filter((e) => e.service.toLowerCase().includes(target));
		}
		if (selectedLevel && selectedLevel !== 'ALL') {
			const lvl = selectedLevel.toUpperCase();
			list = list.filter((e) => e.level.toUpperCase() === lvl);
		}
		if (searchQuery.trim()) {
			const q = searchQuery.toLowerCase().trim();
			list = list.filter(
				(e) => e.message.toLowerCase().includes(q) || e.service.toLowerCase().includes(q)
			);
		}
		return list;
	});

	function scrollToBottom(smooth = false) {
		if (!terminalContainer) return;
		terminalContainer.scrollTo({
			top: terminalContainer.scrollHeight,
			behavior: smooth ? 'smooth' : 'auto'
		});
		isScrolledUp = false;
	}

	function checkIsScrolledUp() {
		if (!terminalContainer) return;
		const { scrollTop, scrollHeight, clientHeight } = terminalContainer;
		const distanceToBottom = scrollHeight - scrollTop - clientHeight;
		isScrolledUp = distanceToBottom > 35;
	}

	function handleTerminalScroll() {
		if (!terminalContainer) return;
		checkIsScrolledUp();
		if (isScrolledUp) {
			if (autoScroll) {
				autoScroll = false;
			}
		} else {
			if (!autoScroll) {
				autoScroll = true;
			}
		}
	}

	async function loadInitialData() {
		isLoading = true;
		try {
			const [servicesList, initialEntries] = await Promise.all([
				logsApi.getServices().catch(() => ['all', 'monolai']),
				logsApi
					.getLogs({
						service: selectedService,
						level: selectedLevel !== 'ALL' ? selectedLevel : undefined,
						limit: MAX_DISPLAY_LINES
					})
					.catch(() => [])
			]);

			const unique = Array.from(new Set(['all', 'monolai', ...servicesList]));
			availableServices = unique;
			logEntries = initialEntries;

			await tick();
			if (autoScroll) {
				scrollToBottom(false);
			} else {
				checkIsScrolledUp();
			}
		} catch (err) {
			console.error('Failed to load initial logs:', err);
		} finally {
			isLoading = false;
		}
	}

	function reconnectStream() {
		startStreaming();
		loadInitialData();
	}

	function startStreaming() {
		if (unsubscribeStream) {
			unsubscribeStream();
		}

		isConnected = true;
		headerState.setLogsStatus(true, reconnectStream);

		unsubscribeStream = logsApi.subscribeStream(
			async (entry: LogEntry) => {
				if (!isConnected) {
					isConnected = true;
					headerState.setLogsStatus(true, reconnectStream);
				}

				// 1. Add new service to options dynamically
				if (!availableServices.includes(entry.service)) {
					availableServices = [...availableServices, entry.service];
				}

				// 2. Add entry to in-memory list (capped at MAX_DISPLAY_LINES)
				if (logEntries.length >= MAX_DISPLAY_LINES) {
					logEntries = [...logEntries.slice(logEntries.length - MAX_DISPLAY_LINES + 1), entry];
				} else {
					logEntries = [...logEntries, entry];
				}

				// 3. Autoscroll if enabled
				if (autoScroll) {
					await tick();
					scrollToBottom(false);
				} else {
					await tick();
					checkIsScrolledUp();
				}
			},
			() => {
				isConnected = false;
				headerState.setLogsStatus(false, reconnectStream);
			}
		);
	}

	async function handleServiceChange(newSvc: string) {
		selectedService = newSvc;
		await loadInitialData();
	}

	async function handleClearLogs() {
		const confirmed = await askConfirm(
			t('logs.confirmClear'),
			t('logs.clearLogs'),
			'danger',
			t('common.clear'),
			t('common.cancel')
		);
		if (!confirmed) return;

		try {
			await logsApi.clearLogs(selectedService);
			if (selectedService === 'all') {
				logEntries = [];
			} else {
				logEntries = logEntries.filter(
					(e) => !e.service.toLowerCase().includes(selectedService.toLowerCase())
				);
			}
			isScrolledUp = false;

			// Refresh available services from backend to clean view
			const servicesList = await logsApi.getServices().catch(() => ['all', 'monolai']);
			availableServices = Array.from(new Set(['all', 'monolai', ...servicesList]));
			if (!availableServices.includes(selectedService)) {
				selectedService = 'all';
			}
		} catch (err) {
			console.error('Failed to clear logs:', err);
		}
	}

	function handleDownloadLogs() {
		const url = logsApi.getDownloadUrl(selectedService);
		window.open(url, '_blank');
	}

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

	function getServiceBadgeColor(service: string): string {
		const s = service.toLowerCase();
		if (s === 'monolai') {
			return 'bg-indigo-500/15 text-indigo-400 border-indigo-500/30';
		}
		if (s.includes('llama')) {
			return 'bg-emerald-500/15 text-emerald-400 border-emerald-500/30';
		}
		if (s.includes('sd') || s.includes('diffus') || s.includes('stable')) {
			return 'bg-fuchsia-500/15 text-fuchsia-400 border-fuchsia-500/30';
		}
		return 'bg-cyan-500/15 text-cyan-400 border-cyan-500/30';
	}

	function getLevelBadgeColor(level: string): string {
		switch (level.toUpperCase()) {
			case 'ERROR':
				return 'bg-rose-500/20 text-rose-400 border-rose-500/40 font-bold';
			case 'WARN':
			case 'WARNING':
				return 'bg-amber-500/20 text-amber-400 border-amber-500/40 font-semibold';
			case 'DEBUG':
				return 'bg-slate-500/20 text-slate-400 border-slate-500/40';
			case 'INFO':
			default:
				return 'bg-sky-500/20 text-sky-400 border-sky-500/40';
		}
	}

	function getMessageColor(level: string): string {
		switch (level.toUpperCase()) {
			case 'ERROR':
				return 'text-rose-300 font-medium';
			case 'WARN':
			case 'WARNING':
				return 'text-amber-200';
			case 'DEBUG':
				return 'text-zinc-400';
			case 'INFO':
			default:
				return 'text-zinc-200';
		}
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

	onMount(() => {
		loadInitialData();
		startStreaming();
	});

	onDestroy(() => {
		if (unsubscribeStream) {
			unsubscribeStream();
		}
		headerState.setLogsStatus(false);
	});
</script>

<svelte:head>
	<title>Monolai - Logs</title>
</svelte:head>

<div
	class="w-full h-full min-h-full px-4 sm:px-8 py-5 md:py-6 pt-16 md:pt-16 max-w-7xl mx-auto box-border flex flex-col gap-4"
>
	<!-- Header -->
	<PageHeader title={t('logs.title')} subtitle={t('logs.subtitle')} icon={ScrollText} />

	<!-- Clean Filters Bar (Fully Responsive) -->
	<div class="bg-[var(--bg-surface)] rounded-xl p-3.5 shadow-none border-0 shrink-0">
		<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-12 gap-3 items-center">
			<!-- Service Select -->
			<div class="lg:col-span-4 flex items-center gap-2">
				<span
					class="text-xs text-[var(--text-muted)] font-medium shrink-0 flex items-center gap-1.5"
				>
					<Terminal size={14} class="text-[var(--primary)]" />
					Service:
				</span>
				<div class="flex-1 min-w-0">
					<Select
						value={selectedService}
						options={serviceOptions}
						searchable={true}
						onchange={handleServiceChange}
					/>
				</div>
			</div>

			<!-- Level Select -->
			<div class="lg:col-span-3 flex items-center gap-2">
				<span
					class="text-xs text-[var(--text-muted)] font-medium shrink-0 flex items-center gap-1.5"
				>
					<Filter size={14} class="text-[var(--primary)]" />
					Level:
				</span>
				<div class="flex-1 min-w-0">
					<Select
						value={selectedLevel}
						options={levelOptions}
						onchange={(val) => (selectedLevel = val)}
					/>
				</div>
			</div>

			<!-- Search Filter -->
			<div class="sm:col-span-2 lg:col-span-5 relative">
				<Search
					size={14}
					class="absolute left-3 top-1/2 -translate-y-1/2 text-[var(--text-muted)] pointer-events-none"
				/>
				<input
					type="text"
					bind:value={searchQuery}
					placeholder={t('logs.filterLogs')}
					class="w-full pl-9 pr-3 py-2 bg-[var(--bg-primary)] border border-transparent hover:border-[var(--border-color)]/40 focus:border-[var(--primary)] rounded-lg text-xs text-[var(--text-primary)] placeholder-[var(--text-muted)] focus:outline-none transition-all"
				/>
			</div>
		</div>
	</div>

	<!-- Clean Terminal Card (Matching CodeBlock Style, expands to screen bottom, min-h 400px) -->
	<div
		class="relative flex flex-col flex-1 min-h-[400px] rounded-xl overflow-hidden bg-[var(--bg-surface)] shadow-none border-0"
	>
		<!-- Clean Top Bar (CodeBlock header style: transparent, no border) -->
		<div
			class="flex items-center justify-between px-3.5 py-2.5 bg-transparent border-0 select-none shrink-0"
		>
			<!-- Left: Title Badge -->
			<div class="flex items-center gap-2">
				<span
					class="text-xs font-semibold tracking-wider uppercase text-[var(--text-secondary)] font-mono"
				>
					LOGS
				</span>
				<span class="text-[11px] text-[var(--text-muted)] font-mono">
					({filteredEntries.length} lines)
				</span>
			</div>

			<!-- Right: Action Icons (Clean, Transparent, Hover-only) -->
			<div class="flex items-center gap-1 text-[var(--text-muted)]">
				<!-- Model Tag Visibility Toggle Icon -->
				<button
					type="button"
					class="p-1.5 rounded-lg transition-colors cursor-pointer border-0 bg-transparent hover:text-[var(--text-primary)] hover:bg-white/5 {showModelTag
						? 'text-[var(--primary)]'
						: 'text-[var(--text-muted)]'}"
					onclick={() => (showModelTag = !showModelTag)}
					title={showModelTag ? t('logs.hideModelTag') : t('logs.showModelTag')}
					aria-label={showModelTag ? t('logs.hideModelTag') : t('logs.showModelTag')}
				>
					<Box size={15} class={showModelTag ? 'text-[var(--primary)]' : 'opacity-60'} />
				</button>

				<!-- AutoScroll Toggle Icon -->
				<button
					type="button"
					class="p-1.5 rounded-lg transition-colors cursor-pointer border-0 bg-transparent hover:text-[var(--text-primary)] hover:bg-white/5 {autoScroll
						? 'text-[var(--primary)]'
						: 'text-[var(--text-muted)]'}"
					onclick={() => {
						autoScroll = !autoScroll;
						if (autoScroll) {
							scrollToBottom(true);
						}
					}}
					title={autoScroll ? t('logs.autoScrollEnabled') : t('logs.autoScrollDisabled')}
					aria-label={t('logs.autoScroll')}
				>
					<ArrowDown size={16} class={autoScroll ? 'text-[var(--primary)]' : 'opacity-60'} />
				</button>

				<!-- Refresh Logs Icon -->
				<button
					type="button"
					class="p-1.5 rounded-lg transition-colors cursor-pointer border-0 bg-transparent hover:text-[var(--text-primary)] hover:bg-white/5 text-[var(--text-muted)]"
					onclick={loadInitialData}
					title={t('common.refresh')}
					aria-label={t('common.refresh')}
				>
					<RefreshCw size={15} class={isLoading ? 'animate-spin text-[var(--primary)]' : ''} />
				</button>

				<!-- Download Log File Icon -->
				<button
					type="button"
					class="p-1.5 rounded-lg transition-colors cursor-pointer border-0 bg-transparent hover:text-[var(--text-primary)] hover:bg-white/5 text-[var(--text-muted)]"
					onclick={handleDownloadLogs}
					title={t('logs.downloadLogs')}
					aria-label={t('logs.downloadLogs')}
				>
					<Download size={15} />
				</button>

				<!-- Clear and Delete Log File Icon -->
				<button
					type="button"
					class="p-1.5 rounded-lg transition-colors cursor-pointer border-0 bg-transparent hover:text-rose-400 hover:bg-rose-500/10 text-[var(--text-muted)]"
					onclick={handleClearLogs}
					title={t('logs.clearLogs')}
					aria-label={t('logs.clearLogs')}
				>
					<Trash2 size={15} />
				</button>
			</div>
		</div>

		<!-- Terminal Body (Unified seamless flow, flexes to fill available card height) -->
		<div
			bind:this={terminalContainer}
			onscroll={handleTerminalScroll}
			class="flex-1 min-h-[340px] overflow-y-auto overflow-x-auto p-3.5 pt-1 font-mono text-[11px] leading-relaxed select-text space-y-0.5 scroll-smooth bg-transparent"
		>
			{#if isLoading && logEntries.length === 0}
				<div class="h-full flex items-center justify-center text-zinc-500 gap-2">
					<RefreshCw size={16} class="animate-spin text-[var(--primary)]" />
					<span>{t('common.loading')}</span>
				</div>
			{:else if filteredEntries.length === 0}
				<div class="h-full flex flex-col items-center justify-center text-zinc-500 gap-2">
					<ScrollText size={28} class="opacity-30" />
					<p class="text-xs">{t('logs.noLogs')}</p>
				</div>
			{:else}
				{#each filteredEntries as entry (entry.id)}
					{@const info = parseServiceInfo(entry.service)}
					<div
						class="flex items-baseline gap-2.5 hover:bg-white/[0.04] px-2 py-0.5 rounded transition-colors group min-w-0"
					>
						<!-- Timestamp -->
						<span class="text-zinc-500 shrink-0 font-mono text-[10.5px] tabular-nums select-none">
							{formatTime(entry.timestamp)}
						</span>

						<!-- Level Badge -->
						<span
							class="shrink-0 px-1 py-0.2 rounded text-[9.5px] border uppercase {getLevelBadgeColor(
								entry.level
							)}"
						>
							{entry.level}
						</span>

						<!-- Runtime Service Pill (Clean runtime name only) -->
						<span
							class="shrink-0 px-1.5 py-0.2 rounded text-[10px] font-semibold border {getServiceBadgeColor(
								info.runtime
							)}"
							title={info.runtime}
						>
							{info.runtime}
						</span>

						<!-- Model Tag (Toggled via card top icon, default hidden) -->
						{#if showModelTag && info.model}
							<span
								class="shrink-0 max-w-[180px] sm:max-w-[240px] truncate text-ellipsis whitespace-nowrap px-1.5 py-0.2 rounded text-[10px] font-mono border border-zinc-700/60 bg-zinc-800/60 text-zinc-300"
								title={info.model}
							>
								{info.model}
							</span>
						{/if}

						<!-- Log Content Line -->
						<span
							class="flex-1 min-w-0 whitespace-pre-wrap break-all leading-normal {getMessageColor(
								entry.level
							)}">{entry.message}</span
						>
					</div>
				{/each}
			{/if}
		</div>

		<!-- Floating Center-Bottom Icon-Only Resume Scroll Button -->
		{#if !autoScroll && isScrolledUp && filteredEntries.length > 0}
			<div
				class="absolute bottom-4 left-1/2 -translate-x-1/2 z-20 animate-in fade-in zoom-in-95 duration-150 pointer-events-auto"
			>
				<button
					type="button"
					class="scroll-down-btn backdrop-blur-xl bg-[var(--bg-surface)]/65 border border-[var(--border-color)]/60"
					onclick={() => {
						autoScroll = true;
						scrollToBottom(true);
					}}
					title="Scroll to bottom"
				>
					<ChevronDown size={18} />
				</button>
			</div>
		{/if}
	</div>
</div>

<style>
	.scroll-down-btn {
		position: absolute;
		top: -2.75rem;
		left: 50%;
		transform: translateX(-50%);
		width: 34px;
		height: 34px;
		border-radius: 50%;
		background: color-mix(in srgb, var(--bg-surface) 65%, transparent);
		-webkit-backdrop-filter: blur(16px);
		backdrop-filter: blur(16px);
		border: 1px solid color-mix(in srgb, var(--border-color) 60%, transparent);
		color: var(--text-primary);
		display: flex;
		align-items: center;
		justify-content: center;
		cursor: pointer;
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
		transition: all 0.15s ease;
		pointer-events: auto;
	}

	.scroll-down-btn:hover {
		background-color: var(--bg-hover);
		transform: translateX(-50%) scale(1.05);
	}
</style>
