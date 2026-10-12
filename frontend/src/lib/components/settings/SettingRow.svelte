<script lang="ts">
	import { ChevronRight, ExternalLink } from '@lucide/svelte';
	import type { Component, Snippet } from 'svelte';

	let {
		icon = undefined,
		iconSnippet = undefined,
		title,
		subtitle = undefined,
		href = undefined,
		onclick = undefined,
		vertical = false,
		class: className = '',
		children,
		actionSnippet = undefined
	}: {
		icon?: Component<{ size?: number; class?: string }>;
		iconSnippet?: Snippet;
		title: string;
		subtitle?: string;
		href?: string;
		onclick?: () => void;
		vertical?: boolean;
		class?: string;
		children?: Snippet;
		actionSnippet?: Snippet;
	} = $props();

	const IconComponent = $derived(icon);
</script>

<div class="bg-[var(--bg-hover)] first:rounded-t-2xl last:rounded-b-2xl">
	{#if href}
		<a
			{href}
			target="_blank"
			rel="noopener noreferrer"
			class="flex items-center justify-between gap-3 px-3.5 py-3 sm:px-4 sm:py-3.5 min-h-[48px] first:rounded-t-2xl last:rounded-b-2xl transition-colors hover:bg-[var(--bg-surface-hover)]/40 no-underline text-[var(--text-primary)] group {className}"
		>
			<div class="flex items-center gap-2.5 min-w-0 flex-1">
				{#if iconSnippet}
					{@render iconSnippet()}
				{:else if IconComponent}
					<IconComponent size={17} class="text-[var(--primary)] shrink-0" />
				{/if}
				<div class="flex flex-col min-w-0">
					<span class="text-xs sm:text-sm font-medium text-[var(--text-primary)] leading-tight">
						{title}
					</span>
					{#if subtitle}
						<span class="text-[11px] text-[var(--text-muted)] leading-tight mt-0.5 truncate">
							{subtitle}
						</span>
					{/if}
				</div>
			</div>

			<div class="flex items-center gap-2 shrink-0">
				{#if children}
					{@render children()}
				{:else}
					<ExternalLink size={17} class="text-[var(--text-muted)] shrink-0" />
				{/if}
			</div>
		</a>
	{:else if onclick}
		<button
			type="button"
			{onclick}
			class="w-full text-left flex items-center justify-between gap-3 px-3.5 py-3 sm:px-4 sm:py-3.5 min-h-[48px] first:rounded-t-2xl last:rounded-b-2xl transition-colors hover:bg-[var(--bg-surface-hover)]/40 bg-transparent border-0 cursor-pointer group {className}"
		>
			<div class="flex items-center gap-2.5 min-w-0 flex-1">
				{#if iconSnippet}
					{@render iconSnippet()}
				{:else if IconComponent}
					<IconComponent size={17} class="text-[var(--primary)] shrink-0" />
				{/if}
				<div class="flex flex-col min-w-0">
					<span class="text-xs sm:text-sm font-medium text-[var(--text-primary)] leading-tight">
						{title}
					</span>
					{#if subtitle}
						<span class="text-[11px] text-[var(--text-muted)] leading-tight mt-0.5 truncate">
							{subtitle}
						</span>
					{/if}
				</div>
			</div>

			<div class="flex items-center gap-2 shrink-0">
				{#if children}
					{@render children()}
				{:else}
					<ChevronRight size={17} class="text-[var(--text-muted)] shrink-0" />
				{/if}
			</div>
		</button>
	{:else if vertical}
		<div
			class="flex flex-col items-start justify-between gap-3 px-3.5 py-2.5 sm:px-4 sm:py-3 min-h-[48px] {className}"
		>
			<div class="flex items-center justify-between w-full">
				<div class="flex items-center gap-2.5 min-w-0 flex-1">
					{#if iconSnippet}
						{@render iconSnippet()}
					{:else if IconComponent}
						<IconComponent size={17} class="text-[var(--primary)] shrink-0" />
					{/if}
					<div class="flex flex-col min-w-0">
						<span class="text-xs sm:text-sm font-medium text-[var(--text-primary)] leading-tight">
							{title}
						</span>
						{#if subtitle}
							<span class="text-[11px] text-[var(--text-muted)] leading-tight mt-0.5 truncate">
								{subtitle}
							</span>
						{/if}
					</div>
				</div>

				<div class="flex items-center gap-2 shrink-0">
					{#if actionSnippet}
						{@render actionSnippet()}
					{/if}
				</div>
			</div>
			{#if children}
				<div class="w-full">
					{@render children()}
				</div>
			{/if}
		</div>
	{:else}
		<div
			class="flex items-center justify-between gap-3 px-3.5 py-2.5 sm:px-4 sm:py-3 min-h-[48px] {className}"
		>
			<div class="flex items-center gap-2.5 min-w-0 flex-1">
				{#if iconSnippet}
					{@render iconSnippet()}
				{:else if IconComponent}
					<IconComponent size={17} class="text-[var(--primary)] shrink-0" />
				{/if}
				<div class="flex flex-col min-w-0">
					<span class="text-xs sm:text-sm font-medium text-[var(--text-primary)] leading-tight">
						{title}
					</span>
					{#if subtitle}
						<span class="text-[11px] text-[var(--text-muted)] leading-tight mt-0.5 truncate">
							{subtitle}
						</span>
					{/if}
				</div>
			</div>

			<div class="flex items-center gap-2 shrink-0 max-w-[55%] sm:max-w-[60%] justify-end">
				{#if children}
					{@render children()}
				{/if}
			</div>
		</div>
	{/if}
</div>
