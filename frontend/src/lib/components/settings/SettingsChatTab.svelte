<script lang="ts">
	import { settingsApi } from '$lib/api/settings';
	import { Alert, Button, Textarea } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import { DEFAULT_SYSTEM_PROMPT, estimateTokens } from '$lib/utils/context';
	import { Check, MessageSquare, RotateCcw, Save, ShieldCheck } from '@lucide/svelte';
	import { onMount } from 'svelte';
	import SettingGroup from './SettingGroup.svelte';
	import { SettingRow } from '.';

	let systemPrompt = $state(DEFAULT_SYSTEM_PROMPT);
	let isLoading = $state(true);
	let isSaving = $state(false);
	let saveSuccess = $state(false);
	let errorMessage = $state<string | null>(null);

	let tokenCount = $derived(estimateTokens(systemPrompt));

	async function loadSettings() {
		isLoading = true;
		errorMessage = null;

		try {
			const res = await settingsApi.get('system_prompt');
			if (res && res.value && res.value.trim()) {
				systemPrompt = res.value;
			}
		} catch {
			// If not set yet, keep DEFAULT_SYSTEM_PROMPT
		} finally {
			isLoading = false;
		}
	}

	async function saveChatPrompt() {
		isSaving = true;
		errorMessage = null;
		saveSuccess = false;

		try {
			await settingsApi.updateAll({
				system_prompt: systemPrompt.trim()
			});

			saveSuccess = true;
			setTimeout(() => {
				saveSuccess = false;
			}, 3000);
		} catch (e: any) {
			errorMessage = e.message || 'Failed to save system prompt';
		} finally {
			isSaving = false;
		}
	}

	function resetToDefault() {
		systemPrompt = DEFAULT_SYSTEM_PROMPT;
	}

	onMount(() => {
		loadSettings();
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

		<SettingGroup title={t('settings.groupChat')} description={t('settings.systemPromptDesc')}>
			<SettingRow
				icon={MessageSquare}
				title={t('settings.defaultSystemPrompt')}
				subtitle={`~${tokenCount} tokens`}
				vertical={true}
			>
				{#snippet actionSnippet()}
					<Button variant="ghost" size="xs" onclick={resetToDefault}>
						<RotateCcw size={12} />
						<span>{t('settings.resetDefault')}</span>
					</Button>
				{/snippet}

				<Textarea
					bind:value={systemPrompt}
					rows={8}
					placeholder={t('settings.systemPromptPlaceholder')}
					class="w-full text-xs font-sans leading-relaxed"
				/>
			</SettingRow>
		</SettingGroup>
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

		<Button variant="primary" size="md" loading={isSaving} onclick={saveChatPrompt}>
			{#if saveSuccess}
				<Check size={14} />
				<span>{t('common.saved')}</span>
			{:else}
				<Save size={14} />
				<span>{t('settings.saveChat')}</span>
			{/if}
		</Button>
	</div>
</div>
