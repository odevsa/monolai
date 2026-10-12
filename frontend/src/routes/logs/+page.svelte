<script lang="ts">
	import { logsApi } from '$lib/api/logs';
	import { settingsApi } from '$lib/api/settings';
	import { LogsFilterBar, LogsTerminalHeader, LogsTerminalViewer } from '$lib/components/logs';
	import { PageHeader } from '$lib/components/ui';
	import type { SelectOption } from '$lib/components/ui/molecules/Select.svelte';
	import { askConfirm } from '$lib/confirmStore';
	import { t } from '$lib/i18n';
	import { headerState } from '$lib/state/header.svelte';
	import type { LogEntry } from '$lib/types/logs';
	import { ScrollText } from '@lucide/svelte';
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
	let isInitialLoad = $state(true);

	let terminalContainer = $state<HTMLDivElement | null>(null);
	let unsubscribeStream: (() => void) | null = null;

	let serviceOptions = $derived<SelectOption[]>(
		availableServices.map((svc) => ({
			value: svc,
			label: svc === 'all' ? t('logs.allServices') : svc
		}))
	);

	let levelOptions = $derived<SelectOption[]>([
		{ value: 'ALL', label: t('logs.allLevels') },
		{ value: 'INFO', label: 'INFO' },
		{ value: 'WARN', label: 'WARN' },
		{ value: 'ERROR', label: 'ERROR' },
		{ value: 'DEBUG', label: 'DEBUG' }
	]);

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

	let userInteracting = false;
	let userInteractionTimer: ReturnType<typeof setTimeout> | null = null;

	function markUserInteraction() {
		userInteracting = true;
		if (userInteractionTimer) clearTimeout(userInteractionTimer);
		userInteractionTimer = setTimeout(() => {
			userInteracting = false;
		}, 300);
	}

	function scrollToBottom(smooth = false) {
		if (!terminalContainer) return;
		if (smooth) {
			terminalContainer.scrollTo({
				top: terminalContainer.scrollHeight,
				behavior: 'smooth'
			});
		} else {
			terminalContainer.scrollTop = terminalContainer.scrollHeight;
		}
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
		if (!isScrolledUp) {
			if (!autoScroll) {
				autoScroll = true;
			}
		} else if (userInteracting) {
			if (autoScroll) {
				autoScroll = false;
			}
		}
	}

	async function loadInitialData() {
		isLoading = true;
		try {
			const [servicesList, initialEntries, appSettings] = await Promise.all([
				logsApi.getServices().catch(() => ['all', 'monolai']),
				logsApi
					.getLogs({
						service: selectedService,
						level: selectedLevel !== 'ALL' ? selectedLevel : undefined,
						limit: MAX_DISPLAY_LINES
					})
					.catch(() => []),
				settingsApi.getAll().catch(() => null)
			]);

			if (appSettings && appSettings.enable_log !== undefined) {
				headerState.logsEnabled = appSettings.enable_log !== 'false';
			}
			headerState.setLogsStatus(isConnected, reconnectStream);

			const unique = Array.from(new Set(['all', 'monolai', ...servicesList]));
			availableServices = unique;
			logEntries = initialEntries;

			await tick();
			if (autoScroll && terminalContainer) {
				terminalContainer.scrollTo({
					top: terminalContainer.scrollHeight,
					behavior: 'auto'
				});
				isScrolledUp = false;
			} else {
				checkIsScrolledUp();
			}
			isInitialLoad = false;
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

				if (!availableServices.includes(entry.service)) {
					availableServices = [...availableServices, entry.service];
				}

				if (logEntries.length >= MAX_DISPLAY_LINES) {
					logEntries = [...logEntries.slice(logEntries.length - MAX_DISPLAY_LINES + 1), entry];
				} else {
					logEntries = [...logEntries, entry];
				}

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

	onMount(() => {
		loadInitialData();
		startStreaming();
	});

	onDestroy(() => {
		if (unsubscribeStream) {
			unsubscribeStream();
		}
		if (userInteractionTimer) {
			clearTimeout(userInteractionTimer);
		}
		headerState.setLogsStatus(false);
	});
</script>

<svelte:head>
	<title>{t('app.title')} - {t('logs.title')}</title>
</svelte:head>

<div
	class="w-full h-full min-h-full px-4 sm:px-8 py-5 md:py-6 pt-16 md:pt-16 max-w-7xl mx-auto box-border flex flex-col gap-4"
>
	<!-- Header -->
	<PageHeader title={t('logs.title')} subtitle={t('logs.subtitle')} icon={ScrollText} />

	<!-- Filters Bar -->
	<LogsFilterBar
		bind:selectedService
		bind:selectedLevel
		bind:searchQuery
		{serviceOptions}
		{levelOptions}
		onServiceChange={handleServiceChange}
	/>

	<!-- Terminal Card -->
	<div
		class="relative flex flex-col flex-1 min-h-[400px] rounded-xl overflow-hidden bg-[var(--bg-surface)] shadow-none border-0"
	>
		<!-- Action Header -->
		<LogsTerminalHeader
			filteredCount={filteredEntries.length}
			{showModelTag}
			{autoScroll}
			{isLoading}
			onToggleModelTag={() => (showModelTag = !showModelTag)}
			onToggleAutoScroll={() => {
				autoScroll = !autoScroll;
				if (autoScroll) scrollToBottom(true);
			}}
			onRefresh={loadInitialData}
			onDownload={handleDownloadLogs}
			onClear={handleClearLogs}
		/>

		<!-- Scrollable Output Body -->
		<LogsTerminalViewer
			entries={filteredEntries}
			{isLoading}
			{showModelTag}
			{autoScroll}
			{isScrolledUp}
			bind:containerElement={terminalContainer}
			onScroll={handleTerminalScroll}
			onUserInteraction={markUserInteraction}
			onScrollToBottom={() => {
				autoScroll = true;
				scrollToBottom(true);
			}}
		/>
	</div>
</div>
