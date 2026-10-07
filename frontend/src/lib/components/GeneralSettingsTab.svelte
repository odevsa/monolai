<script lang="ts">
	import { onMount } from 'svelte';
	import { Clock, Save, Check, RefreshCw, Cpu, ShieldCheck, MessageSquare } from '@lucide/svelte';
	import { formatTimeoutDuration as formatDuration } from '$lib/utils/format';
	import { DEFAULT_SYSTEM_PROMPT } from '$lib/utils/context';
	import { settingsApi } from '$lib/api/settings';
	import { t } from '$lib/i18n';
	import SystemConfigCard from './SystemConfigCard.svelte';
	import Button from './ds/atoms/Button.svelte';
	import Alert from './ds/molecules/Alert.svelte';

	let idleAutoUnloadEnabled = $state<boolean>(true);
	let idleTimeoutSeconds = $state<number>(300);
	let readinessTimeout = $state<number>(120);
	let unloadTimeout = $state<number>(10);
	let systemPrompt = $state<string>(DEFAULT_SYSTEM_PROMPT);

	let isLoading = $state(false);
	let isSaving = $state(false);
	let saveSuccess = $state(false);
	let errorMessage = $state<string | null>(null);

	async function loadSettings() {
		isLoading = true;
		errorMessage = null;
		try {
			const settings = await settingsApi.getAll();
			if (settings) {
				if (settings.idle_auto_unload_enabled !== undefined) {
					idleAutoUnloadEnabled = settings.idle_auto_unload_enabled === 'true';
				}
				if (settings.idle_timeout_seconds !== undefined) {
					const parsed = parseInt(settings.idle_timeout_seconds, 10);
					if (!isNaN(parsed) && parsed >= 0) {
						idleTimeoutSeconds = parsed;
					}
				}
				if (settings.readiness_timeout !== undefined) {
					const parsed = parseInt(settings.readiness_timeout, 10);
					if (!isNaN(parsed) && parsed > 0) {
						readinessTimeout = parsed;
					}
				}
				if (settings.unload_timeout !== undefined) {
					const parsed = parseInt(settings.unload_timeout, 10);
					if (!isNaN(parsed) && parsed > 0) {
						unloadTimeout = parsed;
					}
				}
				if (settings.system_prompt !== undefined && settings.system_prompt.trim()) {
					systemPrompt = settings.system_prompt;
				}
			}
		} catch (e: any) {
			errorMessage = e.message || 'Failed to load settings';
		} finally {
			isLoading = false;
		}
	}

	async function saveSettings() {
		isSaving = true;
		errorMessage = null;
		saveSuccess = false;
		try {
			const payload: Record<string, string> = {
				idle_auto_unload_enabled: idleAutoUnloadEnabled ? 'true' : 'false',
				idle_timeout_seconds: String(Math.max(0, idleTimeoutSeconds)),
				readiness_timeout: String(Math.max(1, readinessTimeout)),
				unload_timeout: String(Math.max(1, unloadTimeout)),
				system_prompt: systemPrompt.trim()
			};

			await settingsApi.updateAll(payload);

			saveSuccess = true;
			setTimeout(() => {
				saveSuccess = false;
			}, 3000);
		} catch (e: any) {
			errorMessage = e.message || 'Error saving settings';
		} finally {
			isSaving = false;
		}
	}

	function setPreset(seconds: number) {
		idleTimeoutSeconds = seconds;
	}

	onMount(() => {
		loadSettings();
	});
</script>

<div class="flex flex-col gap-6">
	<!-- Active System Configuration & Storage -->
	<SystemConfigCard />

	<div class="border-t border-[var(--border-color)] pt-2 flex flex-col gap-4">
		<div class="border-b border-[var(--border-color)] pb-3">
			<h4
				class="m-0 text-sm sm:text-base font-bold text-[var(--text-primary)] flex items-center gap-2"
			>
				<Cpu size={18} class="text-[var(--primary)]" />
				Process Lifecycle & Memory Management
			</h4>
			<p class="m-0 text-xs text-[var(--text-muted)] mt-1 leading-relaxed">
				Configure automatic model unloading and RAM/VRAM resource recycling.
			</p>
		</div>

		{#if errorMessage}
			<Alert variant="error" dismissible ondismiss={() => (errorMessage = null)}>
				{errorMessage}
			</Alert>
		{/if}

		{#if isLoading}
			<div
				class="flex flex-col items-center justify-center gap-3 py-12 text-[var(--text-muted)] text-xs"
			>
				<RefreshCw size={22} class="animate-spin text-[var(--primary)]" />
				<span>{t('common.loading')}</span>
			</div>
		{:else}
			<div class="flex flex-col gap-4">
				<!-- Auto Unload Toggle Card -->
				<div
					class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 p-4 bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-xl shadow-2xs"
				>
					<div class="flex flex-col gap-1">
						<span class="text-xs sm:text-sm font-semibold text-[var(--text-primary)]">
							{t('settings.idleAutoUnload')}
						</span>
						<span class="text-[0.75rem] text-[var(--text-muted)] leading-relaxed">
							{t('settings.idleAutoUnloadDesc')}
						</span>
					</div>

					<label
						class="relative inline-flex items-center cursor-pointer shrink-0 self-start sm:self-center"
					>
						<input type="checkbox" bind:checked={idleAutoUnloadEnabled} class="sr-only peer" />
						<div
							class="w-11 h-6 bg-[var(--border-color)] rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-[var(--primary)]"
						></div>
					</label>
				</div>

				<!-- Idle Timeout Config Card -->
				<div
					class="bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-xl overflow-hidden transition-all duration-200 shadow-2xs {idleAutoUnloadEnabled
						? ''
						: 'opacity-50 pointer-events-none'}"
				>
					<div
						class="flex items-center gap-2 px-4 py-3 bg-[var(--bg-sidebar)]/60 border-b border-[var(--border-color)] text-xs font-semibold text-[var(--text-primary)]"
					>
						<Clock size={16} class="text-[var(--primary)]" />
						<span
							>{t('settings.idleTimeout')} (<code
								class="bg-[var(--bg-surface)] px-1.5 py-0.5 rounded text-[0.725rem] text-[var(--primary)] font-mono"
								>idle_timeout_seconds</code
							>)</span
						>
					</div>

					<div class="p-4 flex flex-col gap-4">
						<div class="flex flex-col gap-2">
							<label for="idle-timeout-input" class="text-xs text-[var(--text-muted)] font-medium">
								Timeout duration (in seconds):
							</label>
							<div class="flex flex-wrap items-center gap-2.5">
								<input
									id="idle-timeout-input"
									type="number"
									min="0"
									step="10"
									disabled={!idleAutoUnloadEnabled}
									bind:value={idleTimeoutSeconds}
									class="w-32 px-3 py-2 bg-[var(--bg-surface)] border border-[var(--border-color)] rounded-lg text-xs text-[var(--text-primary)] focus:outline-none focus:border-[var(--primary)] focus:ring-1 focus:ring-[var(--primary)]"
								/>
								<span
									class="text-xs text-[var(--primary)] bg-[var(--primary-light)] px-3 py-2 rounded-lg border border-[var(--primary)]/20 font-semibold"
								>
									{formatDuration(idleTimeoutSeconds)}
								</span>
							</div>
						</div>

						<!-- Presets -->
						<div class="flex flex-col gap-2 pt-2 border-t border-[var(--border-color)]/50">
							<span class="text-[0.725rem] text-[var(--text-muted)] font-medium"
								>Quick Presets:</span
							>
							<div class="flex flex-wrap gap-2">
								{#each [60, 300, 600, 900, 1800] as seconds}
									{@const isActive = idleTimeoutSeconds === seconds}
									<button
										type="button"
										class="px-3 py-1.5 bg-[var(--bg-surface)] border text-[var(--text-muted)] rounded-lg text-xs font-medium cursor-pointer transition-all hover:border-[var(--primary)] hover:text-[var(--text-primary)] disabled:cursor-not-allowed {isActive
											? '!bg-[var(--primary)] !text-white !border-[var(--primary)] font-semibold shadow-xs'
											: 'border-[var(--border-color)]'}"
										disabled={!idleAutoUnloadEnabled}
										onclick={() => setPreset(seconds)}
									>
										{seconds < 60 ? `${seconds}s` : `${seconds / 60} min`}{seconds === 300
											? ' (default)'
											: ''}
									</button>
								{/each}
							</div>
						</div>
					</div>
				</div>

				<!-- System Prompt Section -->
				<div class="border-t border-[var(--border-color)] pt-5 flex flex-col gap-3">
					<div class="flex items-center justify-between flex-wrap gap-2">
						<div class="flex flex-col gap-0.5">
							<h5
								class="m-0 text-xs sm:text-sm font-bold text-[var(--text-primary)] flex items-center gap-2"
							>
								<MessageSquare size={16} class="text-[var(--primary)]" />
								{t('settings.defaultSystemPrompt')}
							</h5>
							<p class="m-0 text-[0.725rem] text-[var(--text-muted)] leading-relaxed">
								Instructions sent to models at the beginning of each chat session to define behavior
								and context.
							</p>
						</div>
						<button
							type="button"
							class="text-[0.725rem] text-[var(--text-muted)] hover:text-[var(--primary)] underline cursor-pointer bg-transparent border-0 font-medium"
							onclick={() => (systemPrompt = DEFAULT_SYSTEM_PROMPT)}
						>
							Reset to default
						</button>
					</div>

					<textarea
						bind:value={systemPrompt}
						rows="4"
						placeholder="Enter default system instructions..."
						class="w-full px-3.5 py-2.5 bg-[var(--bg-surface)] border border-[var(--border-color)] rounded-xl text-xs text-[var(--text-primary)] focus:outline-none focus:border-[var(--primary)] resize-y leading-relaxed font-sans"
					></textarea>
				</div>

				<!-- Sticky / Fixed Save Bar -->
				<div
					class="flex items-center justify-between pt-3 mt-2 border-t border-[var(--border-color)]"
				>
					{#if saveSuccess}
						<div
							class="flex items-center gap-1.5 text-xs font-semibold text-emerald-500 animate-in fade-in"
						>
							<ShieldCheck size={16} />
							<span>{t('settings.saveSuccess')}</span>
						</div>
					{:else}
						<span class="text-[0.725rem] text-[var(--text-muted)]"
							>Changes take effect immediately</span
						>
					{/if}

					<Button variant="primary" size="md" loading={isSaving} onclick={saveSettings}>
						{#if saveSuccess}
							<Check size={14} />
							<span>{t('common.saved')}</span>
						{:else}
							<Save size={14} />
							<span>{t('settings.saveSettings')}</span>
						{/if}
					</Button>
				</div>
			</div>
		{/if}
	</div>
</div>
