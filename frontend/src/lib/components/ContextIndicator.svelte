<script lang="ts">
	import { ChevronDown, ChevronUp } from '@lucide/svelte';
	import { formatTokenCount, type PreparedContext } from '$lib/utils/context';
	import { tick } from 'svelte';

	let {
		contextInfo
	}: {
		contextInfo: PreparedContext;
	} = $props();

	let isOpen = $state(false);
	let showDetails = $state(false);
	let wrapperRef: HTMLDivElement | null = $state(null);
	let popoverRef: HTMLDivElement | null = $state(null);
	let desktopStyle = $state('');

	function updatePlacement() {
		if (typeof window === 'undefined' || !wrapperRef) return;

		// On mobile (< 640px), let CSS handle fixed bottom card placement
		if (window.innerWidth < 640) {
			desktopStyle = '';
			return;
		}

		// On desktop (>= 640px), center above the button and clamp to screen bounds
		const triggerRect = wrapperRef.getBoundingClientRect();
		const popoverWidth = 280;
		const triggerCenter = triggerRect.left + triggerRect.width / 2;
		let targetLeft = triggerCenter - popoverWidth / 2;
		const padding = 16;

		if (targetLeft + popoverWidth > window.innerWidth - padding) {
			targetLeft = window.innerWidth - padding - popoverWidth;
		}
		if (targetLeft < padding) {
			targetLeft = padding;
		}

		const relativeLeft = targetLeft - triggerRect.left;
		desktopStyle = `left: ${relativeLeft}px; right: auto; width: ${popoverWidth}px; transform: none;`;
	}

	function togglePopover(e: MouseEvent) {
		e.stopPropagation();
		isOpen = !isOpen;
		if (isOpen) {
			tick().then(updatePlacement);
		}
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
				window.addEventListener('resize', updatePlacement);
				tick().then(updatePlacement);
			} else {
				document.removeEventListener('click', handleClickOutside);
				window.removeEventListener('resize', updatePlacement);
			}
			return () => {
				document.removeEventListener('click', handleClickOutside);
				window.removeEventListener('resize', updatePlacement);
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

<div class="context-indicator-wrapper" bind:this={wrapperRef}>
	<!-- Trigger circular button -->
	<button
		type="button"
		class="context-trigger-btn {isOpen ? 'active' : ''}"
		onclick={togglePopover}
		title="Context window: {contextInfo.percentage}% used ({contextInfo.usedTokens} / {formatTokenCount(contextInfo.maxTokens)})"
		aria-label="Context usage"
	>
		<svg class="circular-progress" viewBox="0 0 36 36">
			<!-- Background circle -->
			<path
				class="circle-bg"
				d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831"
			/>
			<!-- Progress circle matching primary theme color -->
			{#if contextInfo.usedTokens > 0}
				<path
					class="circle-progress"
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
			class="mobile-backdrop sm:hidden"
			onclick={() => (isOpen = false)}
			role="presentation"
		></div>

		<!-- Context Popover matching screenshot with exact typography and KV metrics -->
		<div
			class="context-popover"
			role="dialog"
			bind:this={popoverRef}
			style={desktopStyle}
		>
			<div class="context-header">
				<span class="context-title">Context</span>
				<span class="context-divider">·</span>
				<span class="context-numbers font-medium">
					{formatUsedInScale((details.kvCacheTotal ?? contextInfo.usedTokens) || 0, contextInfo.maxTokens)} / {formatTokenCount(contextInfo.maxTokens)}
				</span>
			</div>

			<!-- Linear Progress Bar filled with Primary Theme Color -->
			<div class="progress-bar-track">
				<div
					class="progress-bar-fill"
					style="width: {contextInfo.usedTokens > 0 ? Math.max(1, Math.min(100, contextInfo.percentage)) : 0}%;"
				></div>
			</div>

			<!-- Stats row -->
			<div class="context-stats-row">
				<span class="stat-used">{contextInfo.percentage}% used</span>
				<span class="stat-remaining">{formatTokenCount(contextInfo.remainingTokens)} remaining</span>
			</div>

			<div class="context-separator"></div>

			<!-- Collapsible Token Usage Details -->
			<button type="button" class="details-toggle-btn" onclick={toggleDetails}>
				<span>Token usage details</span>
				{#if showDetails}
					<ChevronUp size={13} />
				{:else}
					<ChevronDown size={13} />
				{/if}
			</button>

			{#if showDetails}
				<div class="details-breakdown">
					<!-- Across all turns -->
					<div class="details-group">
						<div class="section-label">ACROSS ALL TURNS</div>
						<div class="breakdown-row">
							<span class="breakdown-label">Prompt tokens evaluated</span>
							<span class="breakdown-val">{details.allPromptEvaluated} tok</span>
						</div>
						<div class="breakdown-subtext">
							{details.allPromptCached} reused from KV cache
						</div>
						<div class="breakdown-row">
							<span class="breakdown-label">Tokens generated</span>
							<span class="breakdown-val">{details.allTokensGenerated} tok</span>
						</div>
					</div>

					<!-- This turn · KV cache -->
					<div class="details-group">
						<div class="section-label">THIS TURN · KV CACHE</div>
						<div class="breakdown-row">
							<span class="breakdown-label">Prompt</span>
							<span class="breakdown-val">{details.thisTurnPrompt} tok</span>
						</div>
						<div class="breakdown-subtext">
							{details.thisTurnFresh} fresh + {details.thisTurnCached} cached
						</div>
						<div class="breakdown-row">
							<span class="breakdown-label">Generated</span>
							<span class="breakdown-val">{details.thisTurnGenerated} tok</span>
						</div>
						<div class="breakdown-row highlight-row">
							<span class="breakdown-label font-medium text-[var(--text-primary)]">KV cache total</span>
							<span class="breakdown-val font-semibold text-[var(--text-primary)]">{details.kvCacheTotal} tok</span>
						</div>
					</div>

					<div class="context-separator"></div>

					<!-- Speed -->
					<div class="breakdown-row speed-row">
						<span class="breakdown-label">Avg speed</span>
						<span class="breakdown-val">{details.avgSpeed}</span>
					</div>
				</div>
			{/if}
		</div>
	{/if}
</div>

<style>
	.context-indicator-wrapper {
		position: relative;
		display: inline-flex;
		align-items: center;
	}

	.mobile-backdrop {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.5);
		-webkit-backdrop-filter: blur(2px);
		backdrop-filter: blur(2px);
		z-index: 55;
	}

	.context-trigger-btn {
		width: 32px;
		height: 32px;
		padding: 2px;
		border-radius: 50%;
		background: transparent;
		border: 1px solid transparent;
		display: flex;
		align-items: center;
		justify-content: center;
		cursor: pointer;
		color: var(--text-muted);
		transition: all 0.15s ease;
	}

	.context-trigger-btn:hover,
	.context-trigger-btn.active {
		background: color-mix(in srgb, var(--text-primary) 8%, transparent);
		border-color: var(--border-color);
		color: var(--text-primary);
	}

	.circular-progress {
		width: 24px;
		height: 24px;
		transform: rotate(-90deg);
	}

	.circle-bg {
		fill: none;
		stroke: color-mix(in srgb, var(--text-primary) 22%, transparent);
		stroke-width: 5;
	}

	.circle-progress {
		fill: none;
		stroke: var(--primary);
		stroke-width: 5;
		stroke-linecap: round;
		transition: stroke-dasharray 0.3s ease, stroke 0.3s ease;
	}

	/* Completely opaque, responsive popover */
	.context-popover {
		position: absolute;
		bottom: calc(100% + 0.65rem);
		left: 50%;
		transform: translateX(-50%);
		right: auto;
		background-color: var(--bg-surface);
		border: 1px solid var(--border-color);
		border-radius: 0.875rem;
		padding: 0.85rem 1rem;
		width: 280px;
		max-height: 80vh;
		overflow-y: auto;
		box-shadow: 0 16px 40px rgba(0, 0, 0, 0.55);
		z-index: 60;
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		color: var(--text-primary);
		box-sizing: border-box;
		animation: popoverFadeIn 0.15s ease-out;
	}

	@media (max-width: 639px) {
		.context-popover {
			position: fixed !important;
			bottom: calc(env(safe-area-inset-bottom, 0px) + 5.25rem) !important;
			left: 1rem !important;
			right: 1rem !important;
			width: auto !important;
			max-width: calc(100vw - 2rem) !important;
			transform: none !important;
			border-radius: 1rem;
			box-shadow: 0 20px 50px rgba(0, 0, 0, 0.7);
			max-height: 65vh;
			z-index: 60;
		}
	}

	@keyframes popoverFadeIn {
		from {
			opacity: 0;
			transform: translateY(4px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}

	.context-header {
		display: flex;
		align-items: center;
		gap: 0.35rem;
		font-size: 0.825rem;
	}

	.context-title {
		font-weight: 600;
		color: var(--text-primary);
	}

	.context-divider {
		color: var(--text-muted);
	}

	.context-numbers {
		color: var(--text-secondary);
		font-variant-numeric: tabular-nums;
	}

	.progress-bar-track {
		width: 100%;
		height: 4px;
		background: color-mix(in srgb, var(--text-primary) 12%, transparent);
		border-radius: 9999px;
		overflow: hidden;
	}

	.progress-bar-fill {
		height: 100%;
		background-color: var(--primary);
		border-radius: 9999px;
		transition: width 0.3s ease;
	}

	.context-stats-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		font-size: 0.725rem;
		color: var(--text-muted);
		font-variant-numeric: tabular-nums;
	}

	.stat-used {
		font-weight: 500;
	}

	.stat-remaining {
		color: var(--text-secondary);
	}

	.context-separator {
		height: 1px;
		background: var(--border-color);
		margin: 0.2rem 0;
		opacity: 0.7;
	}

	.details-toggle-btn {
		background: transparent;
		border: none;
		color: var(--text-muted);
		font-size: 0.75rem;
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 0.2rem 0;
		cursor: pointer;
		transition: color 0.15s ease;
	}

	.details-toggle-btn:hover {
		color: var(--text-primary);
	}

	.details-breakdown {
		display: flex;
		flex-direction: column;
		gap: 0.65rem;
		padding-top: 0.25rem;
		font-size: 0.75rem;
	}

	.details-group {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
	}

	.section-label {
		font-size: 0.6875rem;
		font-weight: 700;
		color: var(--primary);
		letter-spacing: 0.06em;
		margin-bottom: 0.15rem;
	}

	.breakdown-row {
		display: flex;
		justify-content: space-between;
		align-items: baseline;
		color: var(--text-secondary);
		font-size: 0.75rem;
	}

	.breakdown-label {
		color: var(--text-muted);
	}

	.breakdown-subtext {
		font-size: 0.68rem;
		color: var(--text-muted);
		opacity: 0.8;
		padding-left: 0.25rem;
		margin-top: -0.15rem;
		margin-bottom: 0.15rem;
	}

	.breakdown-val {
		color: var(--text-secondary);
		font-weight: 500;
		font-variant-numeric: tabular-nums;
	}

	.highlight-row {
		margin-top: 0.15rem;
	}

	.highlight-row .breakdown-label,
	.highlight-row .breakdown-val {
		color: var(--text-primary);
		font-weight: 600;
	}

	.speed-row {
		margin-top: -0.15rem;
	}
</style>
