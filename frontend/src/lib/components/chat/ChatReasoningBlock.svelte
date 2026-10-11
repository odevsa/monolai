<script lang="ts">
	import { t } from '$lib/i18n';
	import { ChevronDown, Sparkles } from '@lucide/svelte';
	import { onMount } from 'svelte';

	interface ChatReasoningBlockProps {
		reasoning: string;
		reasoningDuration?: string;
		isGenerating?: boolean;
		hasContent?: boolean;
	}

	let {
		reasoning,
		reasoningDuration,
		isGenerating = false,
		hasContent = false
	}: ChatReasoningBlockProps = $props();

	let collapsed = $state(false);
	let autoScroll = $state(true);
	let scrollContainer: HTMLDivElement | null = $state(null);
	let hasAutoCollapsed = $state(false);

	onMount(() => {
		if (!isGenerating && hasContent) {
			collapsed = true;
		}
	});

	$effect(() => {
		if (!hasAutoCollapsed && isGenerating && reasoning && hasContent) {
			collapsed = true;
			hasAutoCollapsed = true;
		}
	});

	$effect(() => {
		if (reasoning && scrollContainer && autoScroll) {
			scrollContainer.scrollTop = scrollContainer.scrollHeight;
		}
	});

	function toggleCollapse() {
		collapsed = !collapsed;
	}

	function handleScroll(e: Event) {
		const target = e.currentTarget as HTMLElement;
		if (!target) return;
		const distanceFromBottom = target.scrollHeight - target.scrollTop - target.clientHeight;
		if (distanceFromBottom > 35) {
			autoScroll = false;
		} else if (distanceFromBottom <= 15) {
			autoScroll = true;
		}
	}
</script>

<div
	class="reasoning-container group/reasoning mb-3 rounded-xl overflow-hidden transition-colors duration-150 {!collapsed
		? 'bg-[var(--bg-surface)]'
		: 'bg-transparent hover:bg-[var(--bg-surface)]'}"
>
	<button
		type="button"
		class="flex items-center justify-between w-full px-3.5 py-2 text-xs text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors cursor-pointer border-0 bg-transparent select-none"
		onclick={toggleCollapse}
		aria-expanded={!collapsed}
	>
		<div class="flex items-center gap-2 font-medium">
			{#if isGenerating && !hasContent}
				<Sparkles size={13} class="text-[var(--primary)] animate-pulse shrink-0" />
				<span class="text-[var(--primary)] font-semibold">{t('chat.thinkingActive')}</span>
			{:else}
				<Sparkles size={13} class="text-[var(--text-muted)] shrink-0" />
				<span>
					{reasoningDuration
						? t('chat.thoughtFor', { duration: reasoningDuration })
						: t('chat.thinkingProcess')}
				</span>
			{/if}
		</div>
		<ChevronDown
			size={13}
			class="text-[var(--text-muted)] transition-all duration-150 shrink-0 {!collapsed
				? 'opacity-100 rotate-180'
				: 'opacity-0 group-hover/reasoning:opacity-100'}"
		/>
	</button>

	{#if !collapsed}
		<div
			bind:this={scrollContainer}
			onscroll={handleScroll}
			class="px-3.5 pb-3 pt-1 text-xs text-[var(--text-secondary)] bg-transparent leading-relaxed font-sans whitespace-pre-wrap max-h-36 sm:max-h-40 overflow-y-auto"
		>
			{reasoning}
		</div>
	{/if}
</div>
