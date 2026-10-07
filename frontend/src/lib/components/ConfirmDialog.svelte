<script lang="ts">
	import { tick } from 'svelte';
	import { confirmState } from '$lib/state/confirm.svelte';
	import { AlertTriangle, Info, Trash2, X } from '@lucide/svelte';

	let current = $derived(confirmState.current);
	let confirmBtnRef = $state<HTMLButtonElement | null>(null);

	$effect(() => {
		if (current) {
			tick().then(() => {
				confirmBtnRef?.focus();
			});
		}
	});

	function handleConfirm() {
		confirmState.close(true);
	}

	function handleCancel() {
		confirmState.close(false);
	}

	function handleKeyDown(e: KeyboardEvent) {
		if (!current) return;
		if (e.key === 'Escape') {
			e.preventDefault();
			handleCancel();
		} else if (e.key === 'Enter') {
			e.preventDefault();
			handleConfirm();
		}
	}
</script>

<svelte:window onkeydown={handleKeyDown} />

{#if current}
	<div
		class="fixed inset-0 bg-black/65 backdrop-blur-md flex items-center justify-center z-[9999] p-4 animate-in fade-in duration-150"
		onclick={handleCancel}
		role="presentation"
	>
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
		<div
			class="bg-[var(--bg-surface)] border border-[var(--border-color)] rounded-2xl w-full max-w-[400px] p-5 flex flex-col gap-4 shadow-2xl animate-in zoom-in-95 duration-150"
			onclick={(e) => e.stopPropagation()}
			role="dialog"
			aria-modal="true"
			tabindex="-1"
		>
			<div class="flex items-center justify-between">
				<div
					class="w-10 h-10 rounded-xl flex items-center justify-center shrink-0 {current.variant ===
					'danger'
						? 'bg-red-500/15 text-red-500'
						: current.variant === 'warning'
							? 'bg-amber-500/15 text-amber-500'
							: 'bg-[var(--primary-light)] text-[var(--primary)]'}"
				>
					{#if current.variant === 'danger'}
						<Trash2 size={20} />
					{:else if current.variant === 'warning'}
						<AlertTriangle size={20} />
					{:else}
						<Info size={20} />
					{/if}
				</div>
				<button
					type="button"
					class="bg-transparent border-0 text-[var(--text-muted)] cursor-pointer p-1 rounded-md flex items-center justify-center transition-colors hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)]"
					onclick={handleCancel}
					aria-label="Close"
				>
					<X size={16} />
				</button>
			</div>

			<div class="flex flex-col gap-1.5">
				<h3 class="m-0 text-base font-bold text-[var(--text-primary)]">{current.title}</h3>
				<p class="m-0 text-xs text-[var(--text-secondary)] leading-relaxed">{current.message}</p>
			</div>

			<div class="flex items-center justify-end gap-2.5 mt-1">
				{#if !current.isAlert}
					<button
						type="button"
						class="bg-[var(--btn-bg)] border border-[var(--border-color)] text-[var(--text-primary)] px-3.5 py-1.5 rounded-lg text-xs font-medium cursor-pointer transition-all duration-150 hover:bg-[var(--btn-hover)] hover:border-[var(--border-hover)]"
						onclick={handleCancel}
					>
						{current.cancelText || 'Cancel'}
					</button>
				{/if}
				<button
					type="button"
					bind:this={confirmBtnRef}
					class="border-0 px-4 py-1.5 rounded-lg text-xs font-semibold cursor-pointer transition-all duration-150 text-white {current.variant ===
					'danger'
						? 'bg-red-500 hover:bg-red-600'
						: current.variant === 'warning'
							? 'bg-amber-500 hover:bg-amber-600'
							: 'bg-[var(--primary)] hover:bg-[var(--primary-hover)]'}"
					onclick={handleConfirm}
				>
					{current.confirmText || 'Confirm'}
				</button>
			</div>
		</div>
	</div>
{/if}
