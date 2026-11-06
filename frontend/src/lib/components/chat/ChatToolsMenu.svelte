<script lang="ts">
	import { Button } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import { buildAcceptAttribute, type ModelSupportedModalities } from '$lib/utils/attachments';
	import { REASONING_PRESETS, type ReasoningPreset } from '$lib/utils/context';
	import {
		Check,
		ChevronRight,
		Files,
		FileText,
		Image,
		Lightbulb,
		Mic,
		Plus,
		Trash,
		Video
	} from '@lucide/svelte';
	import { onMount, tick } from 'svelte';

	interface ChatToolsMenuProps {
		selectedReasoning: ReasoningPreset['id'];
		promptHistoryCount: number;
		supportedModalities?: ModelSupportedModalities;
		onSelectReasoning: (presetId: ReasoningPreset['id']) => void;
		onClearPromptHistory: () => void;
		onAddFiles?: (files: FileList | File[]) => void;
	}

	let {
		selectedReasoning = $bindable('medium'),
		promptHistoryCount = 0,
		supportedModalities = { text: true, image: false, audio: false, video: false },
		onSelectReasoning,
		onClearPromptHistory,
		onAddFiles
	}: ChatToolsMenuProps = $props();

	let toolsMenuOpen = $state(false);
	let reasoningMenuOpen = $state(false);
	let toolsWrapper: HTMLDivElement | null = $state(null);
	let submenuRef: HTMLDivElement | null = $state(null);
	let submenuDirection = $state<'right' | 'left'>('right');
	let fileInputRef: HTMLInputElement | null = $state(null);

	let acceptAttribute = $derived(buildAcceptAttribute(supportedModalities));

	let currentReasoningPreset = $derived(
		REASONING_PRESETS.find((r) => r.id === selectedReasoning) || REASONING_PRESETS[3]
	);

	function toggleToolsMenu() {
		toolsMenuOpen = !toolsMenuOpen;
		if (!toolsMenuOpen) reasoningMenuOpen = false;
	}

	function triggerFileInput() {
		toolsMenuOpen = false;
		reasoningMenuOpen = false;
		fileInputRef?.click();
	}

	function handleFileChange(e: Event) {
		const input = e.target as HTMLInputElement;
		if (input.files && input.files.length > 0 && onAddFiles) {
			onAddFiles(input.files);
			input.value = '';
		}
	}

	function checkSubmenuPlacement() {
		if (submenuRef && typeof window !== 'undefined') {
			const rect = submenuRef.getBoundingClientRect();
			if (rect.right > window.innerWidth - 16) {
				submenuDirection = 'left';
			} else {
				submenuDirection = 'right';
			}
		}
	}

	function handleOutsideClick(e: MouseEvent) {
		if (toolsWrapper && !toolsWrapper.contains(e.target as Node)) {
			toolsMenuOpen = false;
			reasoningMenuOpen = false;
		}
	}

	onMount(() => {
		document.addEventListener('click', handleOutsideClick);
		return () => {
			document.removeEventListener('click', handleOutsideClick);
		};
	});
</script>

<div class="relative inline-flex items-center" bind:this={toolsWrapper}>
	<input
		bind:this={fileInputRef}
		type="file"
		multiple
		accept={acceptAttribute}
		onchange={handleFileChange}
		class="hidden"
	/>

	<Button
		variant="secondary"
		size="icon"
		class="!rounded-full {toolsMenuOpen
			? '!text-[var(--text-primary)] !bg-[var(--bg-surface-hover)] !border-[var(--border-hover)]'
			: ''}"
		onclick={toggleToolsMenu}
		ariaLabel={t('chat.toolsAndAttachments')}
		title={t('chat.toolsAndAttachments')}
	>
		<Plus size={18} />
	</Button>

	{#if toolsMenuOpen}
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<div
			class="fixed inset-0 bg-transparent z-[99] sm:hidden"
			onclick={() => {
				toolsMenuOpen = false;
				reasoningMenuOpen = false;
			}}
			role="presentation"
		></div>
		<div
			class="absolute bottom-[calc(100%+8px)] left-0 bg-[var(--bg-surface)] border border-[var(--border-color)] rounded-xl shadow-[0_8px_24px_rgba(0,0,0,0.3)] p-1.5 flex flex-col min-w-[210px] z-[100]"
		>
			<!-- Add files option -->
			<button
				type="button"
				class="flex items-center justify-between gap-3 py-2 px-2.5 rounded-lg border-0 bg-transparent text-[var(--text-secondary)] text-[0.8rem] cursor-pointer w-full transition-all duration-150 hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)]"
				onclick={triggerFileInput}
			>
				<div class="flex items-center gap-2">
					<Files size={16} />
					<span>{t('chat.attachments.addFiles')}</span>
				</div>
				<div class="flex items-center gap-1.5 text-[var(--text-muted)]">
					{#if supportedModalities.text}
						<FileText size={13} />
					{/if}
					{#if supportedModalities.image}
						<Image size={13} />
					{/if}
					{#if supportedModalities.audio}
						<Mic size={13} />
					{/if}
					{#if supportedModalities.video}
						<Video size={13} />
					{/if}
				</div>
			</button>

			<div class="h-px bg-[var(--border-color)] my-1"></div>

			<!-- Reasoning Selection item with auto-positioning submenu -->
			<div
				class="relative"
				role="none"
				onmouseenter={() => {
					if (typeof window !== 'undefined' && window.matchMedia('(hover: hover)').matches) {
						reasoningMenuOpen = true;
						tick().then(checkSubmenuPlacement);
					}
				}}
				onmouseleave={() => {
					if (typeof window !== 'undefined' && window.matchMedia('(hover: hover)').matches) {
						reasoningMenuOpen = false;
					}
				}}
			>
				<button
					type="button"
					class="flex items-center justify-between gap-3 py-2 px-2.5 rounded-lg border-0 bg-transparent text-[var(--text-secondary)] text-[0.8rem] cursor-pointer w-full transition-all duration-150 hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)] {reasoningMenuOpen
						? 'bg-[var(--bg-hover)] text-[var(--text-primary)]'
						: ''}"
					onclick={() => {
						reasoningMenuOpen = !reasoningMenuOpen;
						if (reasoningMenuOpen) {
							tick().then(checkSubmenuPlacement);
						}
					}}
				>
					<div class="flex items-center gap-2">
						<Lightbulb size={16} />
						<span>{t('chat.reasoning')}</span>
					</div>
					<div class="flex items-center gap-1.5">
						<span class="text-[0.7rem] text-[var(--text-muted)]">
							{t(`chat.reasoningPreset.${currentReasoningPreset.id}.label`)}
						</span>
						<ChevronRight
							size={14}
							class="transition-transform duration-150 {reasoningMenuOpen ? 'rotate-90' : ''}"
						/>
					</div>
				</button>

				{#if reasoningMenuOpen}
					<div
						class="absolute bottom-0 bg-[var(--bg-surface)] border border-[var(--border-color)] rounded-xl shadow-[0_8px_24px_rgba(0,0,0,0.3)] p-1.5 flex flex-col min-w-[180px] z-[101] {submenuDirection ===
						'right'
							? 'left-[calc(100%+6px)]'
							: 'right-[calc(100%+6px)]'}"
						bind:this={submenuRef}
					>
						{#each REASONING_PRESETS as preset}
							<button
								type="button"
								class="flex items-center justify-between gap-2 py-1.5 px-2.5 rounded-lg border-0 bg-transparent text-[0.8rem] cursor-pointer transition-all duration-150 text-left hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)] {selectedReasoning ===
								preset.id
									? 'text-[var(--primary)] font-medium'
									: 'text-[var(--text-secondary)]'}"
								onclick={() => {
									selectedReasoning = preset.id;
									onSelectReasoning(preset.id);
									reasoningMenuOpen = false;
									toolsMenuOpen = false;
								}}
							>
								<div class="flex items-center gap-1.5">
									<span class="w-3.5 h-3.5 flex items-center justify-center">
										{#if selectedReasoning === preset.id}
											<Check size={14} />
										{/if}
									</span>
									<span class="text-[0.8rem]">
										{t(`chat.reasoningPreset.${preset.id}.label`)}
									</span>
								</div>
								{#if preset.desc}
									<span class="text-[0.68rem] text-[var(--text-muted)]">
										{t(`chat.reasoningPreset.${preset.id}.desc`)}
									</span>
								{/if}
							</button>
						{/each}
					</div>
				{/if}
			</div>

			<div class="h-px bg-[var(--border-color)] my-1"></div>

			<button
				type="button"
				class="flex items-center justify-between gap-3 py-2 px-2.5 rounded-lg border-0 bg-transparent text-[var(--text-secondary)] text-[0.8rem] cursor-pointer w-full transition-all duration-150 {promptHistoryCount ===
				0
					? 'opacity-40 cursor-not-allowed'
					: 'hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)]'}"
				disabled={promptHistoryCount === 0}
				onclick={() => {
					if (promptHistoryCount > 0) {
						onClearPromptHistory();
						toolsMenuOpen = false;
					}
				}}
			>
				<div class="flex items-center gap-2">
					<Trash size={16} />
					<span>{t('chat.clearPromptHistory')}</span>
				</div>
			</button>
		</div>
	{/if}
</div>
