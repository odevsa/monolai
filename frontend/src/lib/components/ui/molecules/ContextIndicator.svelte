<script lang="ts">
	import { t } from '$lib/i18n';
	import { formatTokenCount, type PreparedContext } from '$lib/utils/context';
	import { ChevronDown, ChevronUp } from '@lucide/svelte';
	import Divider from '../atoms/Divider.svelte';

	let {
		contextInfo
	}: {
		contextInfo: PreparedContext;
	} = $props();

	let isOpen = $state(false);
	let showDetails = $state(false);
	let wrapperRef: HTMLDivElement | null = $state(null);
	let popoverRef: HTMLDivElement | null = $state(null);

	function togglePopover(e: MouseEvent) {
		e.stopPropagation();
		isOpen = !isOpen;
	}

	function toggleDetails(e: MouseEvent) {
		e.stopPropagation();
		showDetails = !showDetails;
	}

	function handleClickOutside(e: MouseEvent) {
		if (isOpen && wrapperRef && !wrapperRef.contains(e.target as Node)) {
			isOpen = false;
		}
	}

	$effect(() => {
		if (typeof window !== 'undefined') {
			if (isOpen) {
				document.addEventListener('click', handleClickOutside);
			} else {
				document.removeEventListener('click', handleClickOutside);
			}
			return () => {
				document.removeEventListener('click', handleClickOutside);
			};
		}
	});

	let details = $derived(contextInfo.usageDetails);

	function formatUsedInScale(used: number, max: number): string {
		if (max >= 1000) {
			return (used / 1000).toFixed(2).replace(/\.00$/, '') + 'K';
		}
		return String(used);
	}
</script>

<div class="relative inline-flex items-center" bind:this={wrapperRef}>
	<!-- Trigger circular button -->
	<button
		type="button"
		class="w-8 h-8 p-0.5 rounded-full bg-transparent border border-transparent flex items-center justify-center cursor-pointer text-[var(--text-muted)] transition-all duration-150 hover:bg-[var(--text-primary)]/8 hover:border-[var(--border-color)] hover:text-[var(--text-primary)] {isOpen
			? 'bg-[var(--text-primary)]/8 border-[var(--border-color)] text-[var(--text-primary)]'
			: ''}"
		onclick={togglePopover}
		title={t('chat.contextTooltip', {
			percent: contextInfo.percentage,
			used: contextInfo.usedTokens,
			max: formatTokenCount(contextInfo.maxTokens)
		})}
		aria-label={t('chat.contextUsage')}
	>
		<svg class="w-6 h-6 -rotate-90" viewBox="0 0 36 36">
			<!-- Background circle -->
			<path
				class="fill-none stroke-[var(--text-primary)]/20 stroke-[5]"
				d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831"
			/>
			<!-- Progress circle matching primary theme color -->
			{#if contextInfo.usedTokens > 0}
				<path
					class="fill-none stroke-[var(--primary)] stroke-[5] stroke-round transition-[stroke-dasharray] duration-300 ease-in-out"
					stroke-dasharray="{Math.max(2, contextInfo.percentage)}, 100"
					d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831"
				/>
			{/if}
		</svg>
	</button>

	<!-- Mobile dismiss backdrop -->
	{#if isOpen}
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<div
			class="fixed inset-0 bg-black/50 backdrop-blur-xs z-55 sm:hidden"
			onclick={() => (isOpen = false)}
			role="presentation"
		></div>

		<!-- Context Popover matching screenshot with exact typography and KV metrics -->
		<div
			class="absolute bottom-[calc(100%+0.65rem)] left-1/2 -translate-x-1/2 right-auto bg-[var(--bg-surface)] border border-[var(--border-color)] rounded-2xl p-3.5 w-[280px] max-h-[80vh] overflow-y-auto shadow-2xl z-60 flex flex-col gap-2 text-[var(--text-primary)] box-border max-sm:fixed max-sm:bottom-[calc(env(safe-area-inset-bottom,0px)+5.25rem)] max-sm:left-4 max-sm:right-4 max-sm:w-auto max-sm:max-w-[calc(100vw-2rem)] max-sm:translate-x-0 max-sm:rounded-2xl max-sm:max-h-[65vh]"
			role="dialog"
			bind:this={popoverRef}
		>
			<div class="flex items-center gap-1.5 text-xs">
				<span class="font-semibold text-[var(--text-primary)]">{t('chat.context')}</span>
				<span class="text-[var(--text-muted)]">·</span>
				<span class="text-[var(--text-secondary)] font-medium tabular-nums">
					{formatUsedInScale(
						(details.kvCacheTotal ?? contextInfo.usedTokens) || 0,
						contextInfo.maxTokens
					)} / {formatTokenCount(contextInfo.maxTokens)}
				</span>
			</div>

			<!-- Linear Progress Bar filled with Primary Theme Color -->
			<div class="w-full h-1 bg-[var(--text-primary)]/12 rounded-full overflow-hidden">
				<div
					class="h-full bg-[var(--primary)] rounded-full transition-[width] duration-300 ease-in-out"
					style="width: {contextInfo.usedTokens > 0
						? Math.max(1, Math.min(100, contextInfo.percentage))
						: 0}%;"
				></div>
			</div>

			<!-- Stats row -->
			<div
				class="flex items-center justify-between text-[0.725rem] text-[var(--text-muted)] tabular-nums"
			>
				<span class="font-medium">{t('chat.percentUsed', { percent: contextInfo.percentage })}</span
				>
				<span class="text-[var(--text-secondary)]"
					>{t('chat.remainingTokens', {
						count: formatTokenCount(contextInfo.remainingTokens)
					})}</span
				>
			</div>

			<div
				class="flex flex-col gap-4 p-2 text-xs bg-[var(--bg-primary)]/20 border-t border-b border-[var(--border-color)]"
			>
				<!-- Collapsible Token Usage Details -->
				<button
					type="button"
					class="bg-transparent border-0 text-[var(--text-muted)] text-xs flex items-center justify-between cursor-pointer transition-colors duration-150 hover:text-[var(--text-primary)]"
					onclick={toggleDetails}
				>
					<span>{t('chat.tokenUsageDetails')}</span>
					{#if showDetails}
						<ChevronUp size={13} />
					{:else}
						<ChevronDown size={13} />
					{/if}
				</button>

				{#if showDetails}
					<!-- Across all turns -->
					<div class="flex flex-col gap-1">
						<div class="text-[0.6875rem] font-bold text-[var(--primary)] tracking-wider">
							{t('chat.acrossAllTurns')}
						</div>

						<div class="flex flex-col gap-1 pl-2.5">
							<div>
								<div
									class="flex justify-between items-baseline text-[var(--text-secondary)] text-xs"
								>
									<span class="text-[var(--text-primary)]">{t('chat.promptTokensEvaluated')}</span>
									<span class="text-[var(--text-secondary)] font-medium tabular-nums"
										>{details.allPromptEvaluated} {t('chat.tok')}</span
									>
								</div>
								<div class="text-[0.68rem] text-[var(--text-muted)] opacity-80 pl-2.5">
									{details.allPromptCached}
									{t('chat.reusedFromKv')}
								</div>
							</div>

							<div class="flex justify-between items-baseline text-[var(--text-secondary)] text-xs">
								<span class="text-[var(--text-primary)]">{t('chat.tokensGenerated')}</span>
								<span class="text-[var(--text-secondary)] font-medium tabular-nums"
									>{details.allTokensGenerated} {t('chat.tok')}</span
								>
							</div>
						</div>
					</div>

					<!-- This turn · KV cache -->
					<div class="flex flex-col gap-1">
						<div class="text-[0.6875rem] font-bold text-[var(--primary)] tracking-wider">
							{t('chat.thisTurnKv')}
						</div>

						<div class="flex flex-col gap-1 pl-2.5">
							<div>
								<div
									class="flex justify-between items-baseline text-[var(--text-secondary)] text-xs"
								>
									<span class="text-[var(--text-primary)]">{t('common.prompt')}</span>
									<span class="text-[var(--text-secondary)] font-medium tabular-nums"
										>{details.thisTurnPrompt} {t('chat.tok')}</span
									>
								</div>
								<div class="text-[0.68rem] text-[var(--text-muted)] opacity-80 pl-2.5">
									{t('chat.freshAndCached', {
										fresh: details.thisTurnFresh,
										cached: details.thisTurnCached
									})}
								</div>
							</div>

							<div class="flex justify-between items-baseline text-[var(--text-secondary)] text-xs">
								<span class="text-[var(--text-primary)]">{t('chat.generated')}</span>
								<span class="text-[var(--text-secondary)] font-medium tabular-nums"
									>{details.thisTurnGenerated} {t('chat.tok')}</span
								>
							</div>
							<div class="flex justify-between items-baseline text-[var(--text-secondary)] text-xs">
								<span class="font-semibold text-[var(--warning)]">{t('chat.kvCacheTotal')}</span>
								<span class="font-semibold text-[var(--warning)] tabular-nums"
									>{details.kvCacheTotal} {t('chat.tok')}</span
								>
							</div>
						</div>
					</div>
				{/if}
			</div>

			<!-- Speed -->
			<div class="flex justify-between items-baseline text-[var(--text-secondary)] text-xs -mt-0.5">
				<span class="text-[var(--primary)] font-semibold">{t('chat.avgSpeed')}</span>
				<span class="text-[var(--primary)] font-semibold tabular-nums">{details.avgSpeed}</span>
			</div>
		</div>
	{/if}
</div>
