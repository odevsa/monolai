<script lang="ts">
	import { Button } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import { Copy, RotateCw, Trash } from '@lucide/svelte';

	interface ChatMessageActionsProps {
		role: 'user' | 'assistant';
		onCopy: () => void;
		onDelete: () => void;
		onRetry?: () => void;
	}

	let { role, onCopy, onDelete, onRetry }: ChatMessageActionsProps = $props();
</script>

<div class="flex items-center gap-1.5 mt-0.5 {role === 'user' ? 'justify-end' : ''}">
	<Button
		variant="ghost"
		size="icon-sm"
		class="text-[var(--text-muted)] hover:bg-[var(--bg-hover)] hover:border-[var(--border-color)] hover:text-[var(--text-primary)]"
		title={t('chat.copyTooltip')}
		ariaLabel={t('chat.copyTooltip')}
		onclick={onCopy}
	>
		<Copy size={14} />
	</Button>

	{#if role === 'assistant' && onRetry}
		<Button
			variant="ghost"
			size="icon-sm"
			class="text-[var(--text-muted)] hover:bg-[var(--bg-hover)] hover:border-[var(--border-color)] hover:text-[var(--text-primary)]"
			title={t('chat.retryTooltip')}
			ariaLabel={t('chat.retryTooltip')}
			onclick={onRetry}
		>
			<RotateCw size={14} />
		</Button>
	{/if}

	<Button
		variant="ghost"
		size="icon-sm"
		class="text-[var(--text-muted)] hover:bg-[var(--bg-hover)] hover:border-[var(--border-color)] hover:text-[var(--text-primary)]"
		title={t('chat.deleteTooltip')}
		ariaLabel={t('chat.deleteTooltip')}
		onclick={onDelete}
	>
		<Trash size={14} />
	</Button>
</div>
