<script lang="ts">
	import { t } from '$lib/i18n';
	import { REASONING_PRESETS, type ReasoningPreset } from '$lib/utils/context';
	import { Check, ChevronRight, Lightbulb, Plus, Trash2 } from '@lucide/svelte';
	import { onMount, tick } from 'svelte';

	interface ChatToolsMenuProps {
		selectedReasoning: ReasoningPreset['id'];
		promptHistoryCount: number;
		onSelectReasoning: (presetId: ReasoningPreset['id']) => void;
		onClearPromptHistory: () => void;
	}

	let {
		selectedReasoning = $bindable('medium'),
		promptHistoryCount = 0,
		onSelectReasoning,
		onClearPromptHistory
	}: ChatToolsMenuProps = $props();

	let toolsMenuOpen = $state(false);
	let reasoningMenuOpen = $state(false);
	let toolsWrapper: HTMLDivElement | null = $state(null);
	let submenuRef: HTMLDivElement | null = $state(null);
	let submenuDirection = $state<'right' | 'left'>('right');

	let currentReasoningPreset = $derived(
		REASONING_PRESETS.find((r) => r.id === selectedReasoning) || REASONING_PRESETS[3]
	);

	function toggleToolsMenu() {
		toolsMenuOpen = !toolsMenuOpen;
		if (!toolsMenuOpen) reasoningMenuOpen = false;
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
	<button
		type="button"
		class="w-8 h-8 rounded-full border border-[var(--border-color)] bg-[var(--bg-surface)] text-[var(--text-secondary)] flex items-center justify-center cursor-pointer transition-all duration-150 p-0 hover:text-[var(--text-primary)] hover:bg-[var(--bg-surface-hover)] hover:border-[var(--border-hover)] {toolsMenuOpen
			? 'text-[var(--text-primary)] bg-[var(--bg-surface-hover)] border-[var(--border-hover)]'
			: ''}"
		onclick={toggleToolsMenu}
		aria-label={t('chat.toolsAndAttachments')}
		title={t('chat.toolsAndAttachments')}
	>
		<Plus size={18} />
	</button>

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
			class="absolute bottom-[calc(100%+8px)] left-0 bg-[var(--bg-surface)] border border-[var(--border-color)] rounded-xl shadow-[0_8px_24px_rgba(0,0,0,0.3)] p-1.5 flex flex-col min-w-[200px] z-[100]"
		>
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
						<span class="text-[0.7rem] text-[var(--text-muted)]"
							>{currentReasoningPreset.label}</span
						>
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
									<span class="text-[0.8rem]">{preset.label}</span>
								</div>
								{#if preset.desc}
									<span class="text-[0.68rem] text-[var(--text-muted)]">{preset.desc}</span>
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
					<Trash2 size={16} />
					<span>{t('chat.clearPromptHistory')}</span>
				</div>
			</button>
		</div>
	{/if}
</div>
