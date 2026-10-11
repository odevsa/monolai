<script lang="ts">
	import { t } from '$lib/i18n';
	import { Copy, RotateCw, Trash2 } from '@lucide/svelte';

	interface ChatMessageActionsProps {
		role: 'user' | 'assistant';
		onCopy: () => void;
		onDelete: () => void;
		onRetry?: () => void;
	}

	let { role, onCopy, onDelete, onRetry }: ChatMessageActionsProps = $props();
</script>

<div
	class="flex items-center gap-1.5 opacity-0 group-hover/message:opacity-100 max-sm:opacity-100 transition-opacity duration-150 mt-0.5 {role ===
	'user'
		? 'justify-end'
		: ''}"
>
	<button
		type="button"
		class="flex items-center justify-center w-6 h-6 rounded-md bg-transparent border border-transparent text-[var(--text-muted)] cursor-pointer transition-all duration-150 p-0 hover:bg-[var(--bg-hover)] hover:border-[var(--border-color)] hover:text-[var(--text-primary)]"
		title={t('chat.copyTooltip')}
		aria-label={t('chat.copyTooltip')}
		onclick={onCopy}
	>
		<Copy size={14} />
	</button>

	{#if role === 'assistant' && onRetry}
		<button
			type="button"
			class="flex items-center justify-center w-6 h-6 rounded-md bg-transparent border border-transparent text-[var(--text-muted)] cursor-pointer transition-all duration-150 p-0 hover:bg-[var(--bg-hover)] hover:border-[var(--border-color)] hover:text-[var(--text-primary)]"
			title={t('chat.retryTooltip')}
			aria-label={t('chat.retryTooltip')}
			onclick={onRetry}
		>
			<RotateCw size={14} />
		</button>
	{/if}

	<button
		type="button"
		class="flex items-center justify-center w-6 h-6 rounded-md bg-transparent border border-transparent text-[var(--text-muted)] cursor-pointer transition-all duration-150 p-0 hover:bg-[var(--bg-hover)] hover:border-[var(--border-color)] hover:text-[var(--text-primary)]"
		title={t('chat.deleteTooltip')}
		aria-label={t('chat.deleteTooltip')}
		onclick={onDelete}
	>
		<Trash2 size={14} />
	</button>
</div>
