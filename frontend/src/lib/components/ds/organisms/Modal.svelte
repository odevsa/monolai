<script lang="ts">
	import { X } from '@lucide/svelte';
	import type { Snippet } from 'svelte';

	let {
		open = $bindable(false),
		title = '',
		maxWidth = 'md',
		class: className = '',
		icon: IconComponent,
		onclose,
		header,
		actions,
		footer,
		children
	}: {
		open: boolean;
		title?: string;
		maxWidth?: 'sm' | 'md' | 'lg' | 'xl' | '2xl';
		class?: string;
		icon?: any;
		onclose?: () => void;
		header?: Snippet;
		actions?: Snippet;
		footer?: Snippet;
		children?: Snippet;
	} = $props();

	const maxWidthClasses = {
		sm: 'max-w-[400px]',
		md: 'max-w-[520px]',
		lg: 'max-w-[680px]',
		xl: 'max-w-[800px]',
		'2xl': 'max-w-[960px]'
	};

	function handleClose() {
		open = false;
		if (onclose) onclose();
	}

	function handleKeyDown(e: KeyboardEvent) {
		if (open && e.key === 'Escape') {
			e.preventDefault();
			handleClose();
		}
	}

	function handleBackdropClick(e: MouseEvent) {
		if (e.target === e.currentTarget) {
			handleClose();
		}
	}
</script>

<svelte:window onkeydown={handleKeyDown} />

{#if open}
	<div
		class="fixed inset-0 bg-black/70 backdrop-blur-md flex items-center justify-center z-[200] p-4 animate-in fade-in duration-150"
		onclick={handleBackdropClick}
		role="presentation"
	>
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
		<div
			class="bg-[var(--bg-surface)] border border-[var(--border-color)] rounded-2xl w-full {maxWidthClasses[
				maxWidth
			]} max-h-[90vh] flex flex-col shadow-2xl animate-in zoom-in-95 duration-150 overflow-hidden {className}"
			onclick={(e) => e.stopPropagation()}
			role="dialog"
			aria-modal="true"
			tabindex="-1"
		>
			<!-- Modal Header -->
			<div
				class="flex items-center justify-between px-5 py-4 border-b border-[var(--border-color)] shrink-0 bg-[var(--bg-sidebar)]/50"
			>
				{#if header}
					{@render header()}
				{:else}
					<div class="flex items-center gap-2.5 min-w-0">
						{#if IconComponent}
							<div
								class="w-8 h-8 rounded-lg flex items-center justify-center bg-[var(--primary-light)] text-[var(--primary)] shrink-0"
							>
								<IconComponent size={16} />
							</div>
						{/if}
						{#if title}
							<h3 class="m-0 text-sm font-bold text-[var(--text-primary)] truncate">{title}</h3>
						{/if}
					</div>
				{/if}

				<div class="flex items-center gap-2 shrink-0">
					{#if actions}
						{@render actions()}
					{/if}
					<button
						type="button"
						class="w-7 h-7 flex items-center justify-center rounded-lg text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)] transition-colors border-0 bg-transparent cursor-pointer"
						onclick={handleClose}
						aria-label="Close dialog"
					>
						<X size={16} />
					</button>
				</div>
			</div>

			<!-- Modal Body -->
			<div class="flex-1 overflow-y-auto p-5 text-xs text-[var(--text-primary)]">
				{@render children?.()}
			</div>

			<!-- Modal Footer -->
			{#if footer}
				<div
					class="flex items-center justify-end gap-2.5 px-5 py-3.5 border-t border-[var(--border-color)] bg-[var(--bg-sidebar)]/30 shrink-0"
				>
					{@render footer()}
				</div>
			{/if}
		</div>
	</div>
{/if}
