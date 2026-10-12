<script lang="ts">
	import { Button, ContextIndicator } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import type { PreparedContext, ReasoningPreset } from '$lib/utils/context';
	import { ArrowUp, ChevronDown, Square } from '@lucide/svelte';
	import { tick } from 'svelte';
	import ChatModelSelector from './ChatModelSelector.svelte';
	import ChatToolsMenu from './ChatToolsMenu.svelte';

	interface RegisteredModel {
		id: string;
		runtime: string;
		flags: string;
	}

	interface ChatInputBarProps {
		inputMessage: string;
		isGenerating: boolean;
		selectedModel: string;
		selectedReasoning: ReasoningPreset['id'];
		registeredChatModels: RegisteredModel[];
		promptHistory: string[];
		contextInfo: PreparedContext;
		showScrollButton: boolean;
		scrollbarWidth: number;
		inputSectionHeight?: number;
		onSendMessage: () => void;
		onStopGeneration: () => void;
		onScrollToBottom: () => void;
		onClearPromptHistory: () => void;
		onSelectModel: (modelId: string) => void;
		onSelectReasoning: (presetId: ReasoningPreset['id']) => void;
		onFocus?: () => void;
		onBlur?: () => void;
	}

	let {
		inputMessage = $bindable(''),
		isGenerating,
		selectedModel = $bindable(''),
		selectedReasoning = $bindable('medium'),
		registeredChatModels = [],
		promptHistory = [],
		contextInfo,
		showScrollButton,
		scrollbarWidth,
		inputSectionHeight = $bindable(110),
		onSendMessage,
		onStopGeneration,
		onScrollToBottom,
		onClearPromptHistory,
		onSelectModel,
		onSelectReasoning,
		onFocus,
		onBlur
	}: ChatInputBarProps = $props();

	let inputSectionRef = $state<HTMLDivElement | null>(null);
	let textareaRef = $state<HTMLTextAreaElement | null>(null);
	let historyIndex = $state<number>(-1);
	let currentDraft = $state<string>('');

	function adjustTextareaHeight() {
		if (textareaRef) {
			textareaRef.style.height = 'auto';
			textareaRef.style.height = Math.min(textareaRef.scrollHeight, 200) + 'px';
		}
	}

	export function focus() {
		textareaRef?.focus();
	}

	export function resetHeight() {
		tick().then(() => {
			if (textareaRef) {
				textareaRef.style.height = 'auto';
			}
		});
	}

	function isMobileDevice(): boolean {
		if (typeof window === 'undefined') return false;
		const isTouch =
			window.matchMedia('(pointer: coarse)').matches ||
			(typeof navigator !== 'undefined' && navigator.maxTouchPoints > 0);
		const isMobileWidth =
			window.innerWidth <= 768 || window.matchMedia('(max-width: 768px)').matches;
		const isMobileUA =
			typeof navigator !== 'undefined' &&
			/Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(navigator.userAgent);
		return (isTouch && isMobileWidth) || isMobileUA;
	}

	function handleKeyDown(e: KeyboardEvent) {
		if (e.key === 'Enter' && !e.shiftKey) {
			if (isMobileDevice()) {
				setTimeout(adjustTextareaHeight, 0);
				return;
			}
			e.preventDefault();
			onSendMessage();
			historyIndex = -1;
			currentDraft = '';
			return;
		}

		if (e.key === 'ArrowUp') {
			const isAtStart = textareaRef ? textareaRef.selectionStart === 0 : true;
			if (isAtStart && promptHistory.length > 0) {
				e.preventDefault();
				if (historyIndex === -1) {
					currentDraft = inputMessage;
				}
				if (historyIndex < promptHistory.length - 1) {
					historyIndex++;
					inputMessage = promptHistory[historyIndex];
					adjustTextareaHeight();
				}
			}
		} else if (e.key === 'ArrowDown') {
			if (historyIndex > -1) {
				e.preventDefault();
				historyIndex--;
				if (historyIndex >= 0) {
					inputMessage = promptHistory[historyIndex];
				} else {
					inputMessage = currentDraft;
				}
				adjustTextareaHeight();
			}
		}
	}

	$effect(() => {
		if (!inputSectionRef) return;
		const ro = new ResizeObserver((entries) => {
			for (const entry of entries) {
				inputSectionHeight =
					entry.borderBoxSize?.[0]?.blockSize ??
					entry.contentRect?.height ??
					inputSectionRef?.offsetHeight ??
					110;
			}
		});
		ro.observe(inputSectionRef);
		return () => ro.disconnect();
	});
</script>

<div
	class="absolute bottom-6 left-0 right-0 flex flex-col items-center px-6 pointer-events-none z-40 transition-[right] duration-200 ease-out max-sm:bottom-[calc(env(safe-area-inset-bottom,0px)+0.75rem)] max-sm:px-4 max-sm:max-w-[calc(840px+2rem)] max-h-[480px]:landscape:bottom-[0.4rem]"
	bind:this={inputSectionRef}
	style:right="{scrollbarWidth}px"
>
	{#if showScrollButton}
		<Button
			variant="glass"
			size="icon"
			class="!w-9 !h-9 !rounded-full mb-3 pointer-events-auto shadow-[0_4px_12px_rgba(0,0,0,0.2)] hover:-translate-y-0.5"
			onclick={onScrollToBottom}
			title={t('common.scrollToBottom')}
			ariaLabel={t('common.scrollToBottom')}
		>
			<ChevronDown size={18} />
		</Button>
	{/if}

	<div
		class="w-full max-w-[840px] rounded-3xl py-3.5 pr-4 pb-3 pl-4.5 flex flex-col gap-2 pointer-events-auto shadow-[0_8px_32px_rgba(0,0,0,0.25)] transition-[border-color,box-shadow] duration-150 focus-within:border-[var(--border-hover)] focus-within:shadow-[0_8px_32px_rgba(0,0,0,0.35)] backdrop-blur-xl bg-[var(--bg-surface)]/65 border border-[var(--border-color)]/60"
	>
		<textarea
			bind:this={textareaRef}
			bind:value={inputMessage}
			oninput={adjustTextareaHeight}
			onkeydown={handleKeyDown}
			onfocus={onFocus}
			onblur={onBlur}
			placeholder={t('chat.typeMessagePlaceholder')}
			rows="1"
			class="w-full bg-transparent border-0 outline-none resize-none font-sans text-[0.95rem] leading-normal text-[var(--text-primary)] max-h-[200px] p-0 box-border placeholder:text-[var(--text-muted)]"
		></textarea>

		<!-- Input Controls Row -->
		<div class="flex items-center justify-between gap-2 mt-1">
			<div class="flex items-center gap-2">
				<ChatToolsMenu
					bind:selectedReasoning
					promptHistoryCount={promptHistory.length}
					{onSelectReasoning}
					{onClearPromptHistory}
				/>
			</div>

			<div class="flex items-center gap-2 max-sm:gap-1.5">
				<!-- Context Indicator -->
				<ContextIndicator {contextInfo} />

				<!-- Model Selector Badge -->
				<ChatModelSelector bind:selectedModel models={registeredChatModels} {onSelectModel} />

				<!-- Send / Stop Message Button -->
				{#if isGenerating}
					<Button
						variant="primary"
						size="icon"
						class="!rounded-full hover:not-disabled:scale-105"
						onclick={onStopGeneration}
						ariaLabel={t('chat.stopGeneration')}
						title={t('chat.stopGeneration')}
					>
						<Square size={13} fill="currentColor" />
					</Button>
				{:else}
					<Button
						variant="primary"
						size="icon"
						class="!rounded-full hover:not-disabled:scale-105"
						disabled={!inputMessage.trim()}
						onclick={onSendMessage}
						ariaLabel={t('chat.sendMessage')}
						title={t('chat.sendMessage')}
					>
						<ArrowUp size={18} />
					</Button>
				{/if}
			</div>
		</div>
	</div>
</div>
