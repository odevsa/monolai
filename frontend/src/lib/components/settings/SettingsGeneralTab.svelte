<script lang="ts">
	import { configApi } from '$lib/api/config';
	import { settingsApi } from '$lib/api/settings';
	import { Alert, Button, Input, Select, Toggle } from '$lib/components/ui';
	import { AVAILABLE_LOCALES, i18n, isLocaleSupported, t } from '$lib/i18n';
	import { headerState } from '$lib/state/header.svelte';
	import { THEMES, type ThemeId } from '$lib/themes';
	import { formatTimeoutDuration } from '$lib/utils/format';
	import {
		Check,
		Clock,
		Cpu,
		FileText,
		Globe,
		HardDrive,
		Network,
		Palette,
		Radio,
		RotateCcw,
		Save,
		ScrollText,
		ShieldCheck
	} from '@lucide/svelte';
	import { onMount } from 'svelte';
	import SettingGroup from './SettingGroup.svelte';
	import SettingRow from './SettingRow.svelte';

	let {
		themeMode = $bindable<ThemeId>('system')
	}: {
		themeMode: ThemeId;
	} = $props();

	// Server config state
	let host = $state('0.0.0.0');
	let port = $state(8080);
	let modelsDir = $state('');
	let runtimesDir = $state('');
	let isDocker = $state(configApi.getCachedStatus()?.is_docker ?? false);

	// Settings state
	let idleAutoUnload = $state(true);
	let idleTimeoutSeconds = $state(300);
	let enableLog = $state(true);
	let maxLogSizeMb = $state(10);

	// Status & state
	let isLoading = $state(true);
	let isSaving = $state(false);
	let saveSuccess = $state(false);
	let errorMessage = $state<string | null>(null);

	let themeOptions = $derived(
		THEMES.map((th) => ({
			value: th.id,
			label: `${th.name} (${th.type})`
		}))
	);

	const languageOptions = AVAILABLE_LOCALES.map((l) => ({
		value: l.id,
		label: l.label
	}));

	async function loadAllData() {
		isLoading = true;
		errorMessage = null;

		try {
			const [cfgStatus, appSettings] = await Promise.all([
				configApi.getStatus().catch(() => null),
				settingsApi.getAll().catch(() => null)
			]);

			if (cfgStatus) {
				host = cfgStatus.host || '0.0.0.0';
				port = cfgStatus.port ?? 8080;
				modelsDir = cfgStatus.models_dir || '';
				runtimesDir = cfgStatus.runtimes_dir || '';
				isDocker = !!cfgStatus.is_docker;
			}

			if (appSettings) {
				const savedPreference =
					typeof window !== 'undefined' ? localStorage.getItem('monolai_language') : null;
				if (!savedPreference && appSettings.language && isLocaleSupported(appSettings.language)) {
					i18n.setLocale(appSettings.language);
				}
				if (appSettings.idle_auto_unload_enabled !== undefined) {
					idleAutoUnload = appSettings.idle_auto_unload_enabled === 'true';
				}
				if (appSettings.idle_timeout_seconds !== undefined) {
					const parsed = parseInt(appSettings.idle_timeout_seconds, 10);
					if (!isNaN(parsed) && parsed >= 0) idleTimeoutSeconds = parsed;
				}
				if (appSettings.enable_log !== undefined) {
					enableLog = appSettings.enable_log !== 'false';
					headerState.logsEnabled = enableLog;
				}
				if (appSettings.max_log_size_mb !== undefined) {
					const parsed = parseInt(appSettings.max_log_size_mb, 10);
					if (!isNaN(parsed) && parsed > 0) maxLogSizeMb = parsed;
				}
			}
		} catch (e: any) {
			errorMessage = e.message || 'Failed to load configuration';
		} finally {
			isLoading = false;
		}
	}

	async function saveSettings() {
		isSaving = true;
		errorMessage = null;
		saveSuccess = false;

		try {
			// Save config.yaml only when not running inside Docker
			if (!isDocker) {
				await configApi.setup({
					host: host.trim() || '0.0.0.0',
					port: Number(port) || 8080,
					models: modelsDir.trim(),
					runtimes: runtimesDir.trim()
				});
			}

			// Save settings in database
			await settingsApi.updateAll({
				language: i18n.currentLocale,
				idle_auto_unload_enabled: idleAutoUnload ? 'true' : 'false',
				idle_timeout_seconds: String(Math.max(0, idleTimeoutSeconds)),
				enable_log: enableLog ? 'true' : 'false',
				max_log_size_mb: String(Math.max(1, maxLogSizeMb))
			});

			headerState.logsEnabled = enableLog;

			saveSuccess = true;
			setTimeout(() => {
				saveSuccess = false;
			}, 3000);
		} catch (e: any) {
			errorMessage = e.message || 'Failed to save configuration';
		} finally {
			isSaving = false;
		}
	}

	onMount(() => {
		loadAllData();
	});
</script>

<div class="flex flex-col h-full min-h-0 w-full">
	<!-- Scrollable content area -->
	<div class="flex-1 overflow-y-auto p-4 sm:p-6 flex flex-col gap-5 min-h-0">
		{#if errorMessage}
			<Alert variant="error" dismissible ondismiss={() => (errorMessage = null)}>
				{errorMessage}
			</Alert>
		{/if}

		<!-- Group 1: Appearance & Localization -->
		<div class="relative z-30">
			<SettingGroup title={t('settings.groupAppearance')}>
				<SettingRow
					icon={Globe}
					title={t('settings.languageTitle')}
					subtitle={t('settings.languageDesc')}
				>
					<div class="w-40 sm:w-48">
						<Select
							value={i18n.currentLocale}
							options={languageOptions}
							onchange={(val: string) => {
								i18n.setLocale(val);
								settingsApi.updateAll({ language: val }).catch(console.error);
							}}
							placeholder={t('settings.languageTitle')}
							ariaLabel={t('settings.languageTitle')}
						/>
					</div>
				</SettingRow>

				<SettingRow
					icon={Palette}
					title={t('settings.theme')}
					subtitle={t('settings.themesSubtitle')}
				>
					<div class="w-40 sm:w-48">
						<Select
							value={themeMode}
							options={themeOptions}
							onchange={(val: string) => {
								themeMode = val as ThemeId;
							}}
							placeholder={t('settings.theme')}
							ariaLabel={t('settings.theme')}
						/>
					</div>
				</SettingRow>
			</SettingGroup>
		</div>

		<!-- Group 2: Server & Environment (Hidden in Docker) -->
		{#if !isDocker}
			<div class="relative z-20">
				<SettingGroup title={t('settings.groupServer')}>
					<SettingRow icon={Network} title={t('common.host')} subtitle="0.0.0.0 / 127.0.0.1">
						<div class="w-32 sm:w-40">
							<Input bind:value={host} placeholder="0.0.0.0" />
						</div>
					</SettingRow>

					<SettingRow icon={Radio} title={t('common.port')} subtitle="TCP port">
						<div class="w-24 sm:w-28">
							<Input type="number" bind:value={port} min="1" max="65535" placeholder="8080" />
						</div>
					</SettingRow>

					<SettingRow
						icon={HardDrive}
						title={t('settings.modelsDir')}
						subtitle="GGUF / weights folder"
					>
						<div class="w-48 sm:w-64">
							<Input bind:value={modelsDir} placeholder="e.g. ~/.local/share/monolai/models" />
						</div>
					</SettingRow>

					<SettingRow icon={Cpu} title={t('settings.runtimesDir')} subtitle="Binaries folder">
						<div class="w-48 sm:w-64">
							<Input bind:value={runtimesDir} placeholder="e.g. ~/.local/share/monolai/runtimes" />
						</div>
					</SettingRow>
				</SettingGroup>
			</div>
		{/if}

		<!-- Group 3: Process & Memory -->
		<div class="relative z-10">
			<SettingGroup title={t('settings.groupProcess')}>
				<SettingRow
					icon={RotateCcw}
					title={t('settings.idleAutoUnload')}
					subtitle={t('settings.idleAutoUnloadDesc')}
				>
					<Toggle bind:checked={idleAutoUnload} ariaLabel={t('settings.idleAutoUnload')} />
				</SettingRow>

				<SettingRow
					icon={Clock}
					title={t('settings.idleTimeout')}
					subtitle={idleAutoUnload
						? `${idleTimeoutSeconds}s (${formatTimeoutDuration(idleTimeoutSeconds)})`
						: t('common.disabled')}
				>
					<div class="w-24 sm:w-28">
						<Input
							type="number"
							bind:value={idleTimeoutSeconds}
							min="0"
							step="10"
							disabled={!idleAutoUnload}
							placeholder="300"
						/>
					</div>
				</SettingRow>
			</SettingGroup>
		</div>

		<!-- Group 4: Logging -->
		<div class="relative z-0">
			<SettingGroup title={t('settings.groupLogging')}>
				<SettingRow
					icon={FileText}
					title={t('settings.enableLog')}
					subtitle={t('settings.enableLogDesc')}
				>
					<Toggle bind:checked={enableLog} ariaLabel={t('settings.enableLog')} />
				</SettingRow>

				<SettingRow
					icon={ScrollText}
					title={t('settings.maxLogSize')}
					subtitle={enableLog ? `${maxLogSizeMb} MB` : t('common.disabled')}
				>
					<div class="w-24 sm:w-28">
						<Input
							type="number"
							bind:value={maxLogSizeMb}
							min="1"
							max="1024"
							disabled={!enableLog}
							placeholder="10"
						/>
					</div>
				</SettingRow>
			</SettingGroup>
		</div>
	</div>

	<!-- Dedicated Solid Dialog Footer (Completely separate from scroll area) -->
	<div
		class="shrink-0 bg-[var(--bg-surface)] border-t border-[var(--border-color)] px-4 sm:px-6 py-3.5 flex items-center justify-between gap-3 shadow-md"
	>
		<div class="flex items-center gap-2">
			{#if saveSuccess}
				<div
					class="flex items-center gap-1.5 text-xs font-semibold text-[var(--success-text)] animate-in fade-in"
				>
					<ShieldCheck size={16} />
					<span>{t('settings.saveSuccess')}</span>
				</div>
			{:else}
				<span class="text-[11px] text-[var(--text-muted)]">
					{t('settings.changesImmediate')}
				</span>
			{/if}
		</div>

		<Button variant="primary" size="md" loading={isSaving} onclick={saveSettings}>
			{#if saveSuccess}
				<Check size={14} />
				<span>{t('common.saved')}</span>
			{:else}
				<Save size={14} />
				<span>{t('common.save')}</span>
			{/if}
		</Button>
	</div>
</div>
