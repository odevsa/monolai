<script lang="ts">
	import { MarkdownRenderer, ModelBadge } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import { Box, CircleAlert, Clock, Square, Zap } from '@lucide/svelte';
	import ChatMessageActions from './ChatMessageActions.svelte';
	import ChatReasoningBlock from './ChatReasoningBlock.svelte';

	export interface Message {
		id: string;
		role: 'user' | 'assistant';
		content: string;
		reasoning?: string;
		reasoningDuration?: string;
		tokens?: number;
		duration?: string;
		speed?: string;
		modelTags?: string[];
		status?: string;
	}

	interface ChatMessageItemProps {
		message: Message;
		isGenerating?: boolean;
		onCopy: (content: string) => void;
		onDelete: (id: string) => void;
		onRetry?: (id: string) => void;
	}

	let { message, isGenerating = false, onCopy, onDelete, onRetry }: ChatMessageItemProps = $props();

	let isTooltipActive = $state(false);

	function toggleTooltip(e: MouseEvent) {
		e.stopPropagation();
		isTooltipActive = !isTooltipActive;
	}
</script>

<div class="group/message flex flex-col gap-2 {message.role === 'user' ? 'items-end' : ''}">
	{#if message.role === 'user'}
		<div class="flex justify-end max-w-[80%] max-sm:max-w-[95%]">
			<div
				class="bg-[var(--bg-surface)] border border-[var(--border-color)] text-[var(--text-primary)] py-3.5 px-4.5 rounded-[1.25rem_1.25rem_0.25rem_1.25rem] text-base leading-normal whitespace-pre-wrap break-words"
			>
				{message.content}
			</div>
		</div>
		<ChatMessageActions
			role="user"
			onCopy={() => onCopy(message.content)}
			onDelete={() => onDelete(message.id)}
		/>
	{:else}
		<div class="flex flex-col gap-2 max-w-[85%] max-sm:max-w-[95%]">
			<div class="text-[var(--text-primary)] text-[0.95rem] leading-relaxed break-words">
				{#if message.reasoning}
					<ChatReasoningBlock
						reasoning={message.reasoning}
						reasoningDuration={message.reasoningDuration}
						{isGenerating}
						hasContent={!!message.content}
					/>
				{/if}

				{#if !message.content && !message.reasoning && isGenerating}
					<div
						class="inline-flex items-center gap-1.5 py-2 px-3 bg-[var(--bg-surface)] rounded-2xl border border-[var(--border-color)]"
					>
						<span class="w-1.5 h-1.5 rounded-full bg-[var(--text-muted)] animate-pulse"></span>
						<span
							class="w-1.5 h-1.5 rounded-full bg-[var(--text-muted)] animate-pulse [animation-delay:200ms]"
						></span>
						<span
							class="w-1.5 h-1.5 rounded-full bg-[var(--text-muted)] animate-pulse [animation-delay:400ms]"
						></span>
					</div>
				{:else if !message.content && (message.status === 'CANCELLED' || message.status === 'INTERRUPTED')}
					<span class="italic text-[var(--text-muted)] text-[0.85rem]">
						({message.status === 'INTERRUPTED' ? t('chat.interrupted') : t('chat.cancelledByUser')})
					</span>
				{:else if message.content}
					<MarkdownRenderer content={message.content} />
				{/if}
			</div>

			<div class="flex items-center flex-wrap gap-2 text-xs text-[var(--text-muted)] mt-1">
				{#if message.modelTags && message.modelTags.length > 0}
					<ModelBadge model={message.modelTags} />
				{/if}

				{#if message.status === 'CANCELLED' || message.status === 'INTERRUPTED'}
					<div class="group/tooltip relative inline-flex items-center">
						<button
							type="button"
							class="inline-flex items-center justify-center bg-[var(--danger-light)] border border-[var(--danger-border)] text-[var(--danger-text)] rounded-md py-0.5 px-1.5 cursor-pointer transition-all duration-150 hover:bg-[var(--danger-focus)]"
							onclick={toggleTooltip}
							aria-label={message.status === 'INTERRUPTED'
								? t('chat.interrupted')
								: t('chat.cancelledByUser')}
						>
							{#if message.status === 'INTERRUPTED'}
								<CircleAlert size={12} class="text-[var(--danger-text)]" />
							{:else}
								<Square size={10} fill="currentColor" class="text-[var(--danger-text)]" />
							{/if}
						</button>
						<div
							class="absolute bottom-[calc(100%+6px)] left-1/2 -translate-x-1/2 bg-[var(--bg-surface)] border border-[var(--border-color)] text-[var(--text-primary)] text-[0.7rem] py-1.5 px-2.5 rounded-md whitespace-nowrap shadow-md pointer-events-none transition-all duration-150 z-50 {isTooltipActive
								? 'opacity-100 visible'
								: 'opacity-0 invisible group-hover/tooltip:opacity-100 group-hover/tooltip:visible'}"
							role="tooltip"
						>
							<span>
								{message.status === 'INTERRUPTED'
									? t('chat.interrupted')
									: t('chat.cancelledByUser')}
							</span>
							<div
								class="absolute top-full left-1/2 -translate-x-1/2 border-4 border-transparent border-t-[var(--border-color)]"
							></div>
						</div>
					</div>
				{/if}

				<div class="flex items-center flex-wrap gap-2.5">
					{#if message.tokens}
						<span class="inline-flex items-center gap-1"
							><Box size={12} /> {message.tokens} {t('chat.tokens')}</span
						>
					{/if}
					{#if message.duration}
						<span class="inline-flex items-center gap-1"
							><Clock size={12} /> {message.duration}</span
						>
					{/if}
					{#if message.speed}
						<span class="inline-flex items-center gap-1"><Zap size={12} /> {message.speed}</span>
					{/if}
				</div>
			</div>

			<ChatMessageActions
				role="assistant"
				onCopy={() => onCopy(message.content)}
				onDelete={() => onDelete(message.id)}
				onRetry={onRetry ? () => onRetry(message.id) : undefined}
			/>
		</div>
	{/if}
</div>
