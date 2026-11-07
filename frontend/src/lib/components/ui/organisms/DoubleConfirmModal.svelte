<script lang="ts">
	import { t } from '$lib/i18n';
	import { AlertTriangle, ShieldAlert, Trash, Trash2, TriangleAlert } from '@lucide/svelte';
	import Button from '../atoms/Button.svelte';
	import Input from '../atoms/Input.svelte';
	import Modal from './Modal.svelte';

	let {
		open = $bindable(false),
		title = '',
		warningMessage = '',
		confirmPhrase = '',
		confirmButtonText = '',
		loading = false,
		onconfirm,
		oncancel
	}: {
		open: boolean;
		title: string;
		warningMessage: string;
		confirmPhrase: string;
		confirmButtonText: string;
		loading?: boolean;
		onconfirm: () => void | Promise<void>;
		oncancel?: () => void;
	} = $props();

	let step = $state<1 | 2>(1);
	let typedPhrase = $state('');

	$effect(() => {
		if (open) {
			step = 1;
			typedPhrase = '';
		}
	});

	let isPhraseValid = $derived(typedPhrase.trim() === confirmPhrase.trim());

	function handleClose() {
		open = false;
		if (oncancel) oncancel();
	}

	function handleProceedToStep2() {
		step = 2;
	}

	async function handleExecute() {
		if (!isPhraseValid || loading) return;
		await onconfirm();
	}

	function handleKeyDown(e: KeyboardEvent) {
		if (e.key === 'Enter' && step === 2 && isPhraseValid && !loading) {
			e.preventDefault();
			handleExecute();
		}
	}
</script>

<svelte:window onkeydown={handleKeyDown} />

<Modal bind:open maxWidth="sm" {title} onclose={handleClose}>
	{#snippet children()}
		<div class="flex flex-col gap-4">
			{#if step === 1}
				<div
					class="flex flex-col items-center gap-2 text-xs text-[var(--danger-text)] leading-relaxed"
				>
					<div class="rounded-lg text-[var(--danger-text)] shrink-0">
						<TriangleAlert size={48} />
					</div>
					<span class="font-bold text-sm text-center text-[var(--danger-text)]"
						>{t('settings.maintenanceIrreversibleTitle')}</span
					>
					<span class="text-center">{warningMessage}</span>
				</div>

				<p class="text-xs text-[var(--text-secondary)] text-center leading-relaxed m-0">
					{t('settings.maintenanceWarningDesc')}
				</p>
			{:else}
				<div
					class="flex flex-col items-center gap-2 text-xs text-[var(--danger-text)] leading-relaxed"
				>
					<div class="rounded-lg text-[var(--danger-text)] shrink-0">
						<ShieldAlert size={48} />
					</div>
					<span class="font-bold text-sm text-center text-[var(--danger-text)]"
						>{t('settings.maintenanceTypePromptTitle')}</span
					>
					<span class="text-center">{t('settings.maintenanceTypePrompt')}</span>
				</div>

				<div class="flex flex-col gap-2">
					<div
						class="px-3 py-2 bg-[var(--bg-surface-hover)] border border-[var(--border-color)] rounded-lg font-mono text-xs font-bold text-center text-[var(--text-primary)] select-all tracking-wider"
					>
						{confirmPhrase}
					</div>

					<Input
						bind:value={typedPhrase}
						placeholder={confirmPhrase}
						autofocus
						class="font-mono text-center text-xs"
					/>
				</div>
			{/if}
		</div>
	{/snippet}

	{#snippet footer()}
		<div class="flex items-center justify-between w-full">
			<Button variant="secondary" size="md" disabled={loading} onclick={handleClose}>
				{t('common.cancel')}
			</Button>

			{#if step === 1}
				<Button variant="danger" size="md" onclick={handleProceedToStep2}>
					<span>{t('settings.maintenanceProceed')}</span>
				</Button>
			{:else}
				<Button
					variant="danger"
					size="md"
					disabled={!isPhraseValid}
					{loading}
					onclick={handleExecute}
				>
					<Trash size={14} />
					<span>{confirmButtonText}</span>
				</Button>
			{/if}
		</div>
	{/snippet}
</Modal>
