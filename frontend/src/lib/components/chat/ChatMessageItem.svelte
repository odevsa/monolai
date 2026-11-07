<script lang="ts">
	import { AttachmentChip, Button, MarkdownRenderer, ModelBadge } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import type { ChatAttachment } from '$lib/types/chat';
	import { Box, CircleAlert, Clock, Square, Zap } from '@lucide/svelte';
	import { tick } from 'svelte';
	import Loader from '../ui/atoms/Loader.svelte';
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
		attachments?: ChatAttachment[];
	}

	interface ChatMessageItemProps {
		message: Message;
		isGenerating?: boolean;
		onCopy: (content: string) => void;
		onDelete: (id: string) => void;
		onRetry?: (id: string) => void;
		canEdit?: boolean;
		onSaveEdit?: (id: string, newContent: string, newAttachments: ChatAttachment[]) => void;
	}

	let {
		message,
		isGenerating = false,
		onCopy,
		onDelete,
		onRetry,
		canEdit = false,
		onSaveEdit
	}: ChatMessageItemProps = $props();

	let isTooltipActive = $state(false);
	let isEditing = $state(false);
	let editText = $state('');
	let editAttachments = $state<ChatAttachment[]>([]);
	let editTextareaRef = $state<HTMLTextAreaElement | null>(null);

	function autoResizeTextarea() {
		if (editTextareaRef) {
			editTextareaRef.style.height = 'auto';
			editTextareaRef.style.height = Math.min(editTextareaRef.scrollHeight, 250) + 'px';
		}
	}

	function startEdit() {
		if (isGenerating) return;
		isEditing = true;
		editText = message.content || '';
		editAttachments = message.attachments ? [...message.attachments] : [];
		tick().then(() => {
			autoResizeTextarea();
			if (editTextareaRef) {
				editTextareaRef.focus();
				const len = editTextareaRef.value.length;
				editTextareaRef.setSelectionRange(len, len);
			}
		});
	}

	function cancelEdit() {
		isEditing = false;
		editText = '';
		editAttachments = [];
	}

	function submitEdit() {
		if (isGenerating) return;
		const trimmed = editText.trim();
		if (!trimmed && editAttachments.length === 0) return;
		isEditing = false;
		if (onSaveEdit) {
			onSaveEdit(message.id, trimmed, editAttachments);
		}
	}

	function handleKeyDown(e: KeyboardEvent) {
		if (e.key === 'Enter' && !e.shiftKey) {
			e.preventDefault();
			submitEdit();
		} else if (e.key === 'Escape') {
			e.preventDefault();
			cancelEdit();
		}
	}

	function toggleTooltip(e: MouseEvent) {
		e.stopPropagation();
		isTooltipActive = !isTooltipActive;
	}
</script>

<div class="group/message flex flex-col gap-2 {message.role === 'user' ? 'items-end' : ''}">
	{#if message.role === 'user'}
		{#if isEditing}
			<div class="flex flex-col items-end gap-2 w-full max-w-[80%] max-sm:max-w-[95%]">
				{#if editAttachments.length > 0}
					<div class="flex flex-wrap gap-2 w-full justify-end">
						{#each editAttachments as att (att.id)}
							<AttachmentChip
								attachment={att}
								variant="prompt"
								onRemove={(id) => {
									editAttachments = editAttachments.filter((a) => a.id !== id);
								}}
							/>
						{/each}
					</div>
				{/if}

				<div
					class="w-full bg-[var(--bg-surface)] border border-[var(--primary)] rounded-[1.25rem_1.25rem_0.25rem_1.25rem] py-3 px-4 shadow-sm focus-within:ring-2 focus-within:ring-[var(--primary)]/20"
				>
					<textarea
						bind:this={editTextareaRef}
						bind:value={editText}
						oninput={autoResizeTextarea}
						onkeydown={handleKeyDown}
						rows="1"
						class="w-full bg-transparent border-0 outline-none resize-none font-sans text-base leading-normal text-[var(--text-primary)] max-h-[250px] p-0 box-border placeholder:text-[var(--text-muted)] block"
						placeholder={t('chat.typeMessagePlaceholder')}></textarea>
				</div>

				<!-- Inline Actions Underneath: Cancel and Send -->
				<div class="flex items-center gap-2 justify-end mt-0.5">
					<Button variant="secondary" size="xs" onclick={cancelEdit} disabled={isGenerating}>
						{t('common.cancel')}
					</Button>
					<Button
						variant="primary"
						size="xs"
						onclick={submitEdit}
						disabled={isGenerating || (!editText.trim() && editAttachments.length === 0)}
					>
						{t('common.send')}
					</Button>
				</div>
			</div>
		{:else}
			<div class="flex flex-col items-end gap-1.5 max-w-[80%] max-sm:max-w-[95%]">
				{#if message.attachments && message.attachments.length > 0}
					<div class="flex flex-wrap items-center justify-end gap-2 my-1 max-w-full">
						{#each message.attachments as att (att.id)}
							<AttachmentChip attachment={att} variant="history" />
						{/each}
					</div>
				{/if}
				{#if message.content}
					<div
						class="bg-[var(--bg-surface)] border border-[var(--border-color)] text-[var(--text-primary)] py-3.5 px-4.5 rounded-[1.25rem_1.25rem_0.25rem_1.25rem] text-base leading-normal whitespace-pre-wrap break-words"
					>
						{message.content}
					</div>
				{/if}
			</div>
			<ChatMessageActions
				role="user"
				onCopy={() => onCopy(message.content)}
				onDelete={() => onDelete(message.id)}
				onEdit={canEdit ? startEdit : undefined}
			/>
		{/if}
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
					<Loader />
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
