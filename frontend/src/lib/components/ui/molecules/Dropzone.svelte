<script lang="ts">
	import type { Component, Snippet } from 'svelte';
	import { Upload } from '@lucide/svelte';

	interface DropzoneProps {
		accept?: string;
		disabled?: boolean;
		multiple?: boolean;
		file?: File | null;
		title?: string;
		subtitle?: string;
		icon?: Component<{ size?: number; class?: string }>;
		ariaLabel?: string;
		class?: string;
		onFileSelect?: (file: File) => void;
		children?: Snippet;
	}

	let {
		accept = '*',
		disabled = false,
		multiple = false,
		file = null,
		title = '',
		subtitle = '',
		icon: IconComponent = Upload,
		ariaLabel = 'Upload file',
		class: className = '',
		onFileSelect,
		children
	}: DropzoneProps = $props();

	let isDragging = $state(false);
	let dragCounter = 0;
	let inputEl = $state<HTMLInputElement | null>(null);

	function handleFileSelected(e: Event) {
		const target = e.target as HTMLInputElement;
		if (target.files && target.files.length > 0) {
			const selected = target.files[0];
			if (onFileSelect) onFileSelect(selected);
		}
	}

	function handleDragEnter(e: DragEvent) {
		if (disabled) return;
		e.preventDefault();
		dragCounter++;
		if (e.dataTransfer) {
			isDragging = true;
		}
	}

	function handleDragOver(e: DragEvent) {
		if (disabled) return;
		e.preventDefault();
		if (e.dataTransfer) {
			e.dataTransfer.dropEffect = 'copy';
		}
		isDragging = true;
	}

	function handleDragLeave(e: DragEvent) {
		if (disabled) return;
		e.preventDefault();
		dragCounter--;
		if (dragCounter <= 0) {
			dragCounter = 0;
			isDragging = false;
		}
	}

	function handleDrop(e: DragEvent) {
		if (disabled) return;
		e.preventDefault();
		isDragging = false;
		dragCounter = 0;
		if (e.dataTransfer?.files && e.dataTransfer.files.length > 0) {
			const dropped = e.dataTransfer.files[0];
			if (onFileSelect) onFileSelect(dropped);
		}
	}

	function formatFileSize(bytes: number): string {
		if (bytes >= 1024 * 1024) {
			return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
		}
		return `${(bytes / 1024).toFixed(1)} KB`;
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	ondragenter={handleDragEnter}
	ondragover={handleDragOver}
	ondragleave={handleDragLeave}
	ondrop={handleDrop}
	class="group flex flex-col items-center justify-center p-6 border-2 border-dashed rounded-xl transition-all cursor-pointer text-center relative select-none {disabled
		? 'opacity-60 cursor-not-allowed border-[var(--border-color)] bg-[var(--bg-surface)]'
		: isDragging
			? 'border-[var(--primary)] bg-[var(--primary-light)]/20 shadow-sm'
			: 'border-[var(--border-color)] hover:border-[var(--primary)] bg-[var(--bg-hover)]/30 hover:bg-[var(--primary-light)]/20'} {className}"
>
	<input
		bind:this={inputEl}
		type="file"
		{accept}
		{multiple}
		{disabled}
		onchange={handleFileSelected}
		class="absolute inset-0 opacity-0 cursor-pointer w-full h-full disabled:cursor-not-allowed"
		aria-label={ariaLabel}
	/>

	{#if children}
		{@render children()}
	{:else}
		<IconComponent
			size={28}
			class="mb-2 shrink-0 transition-transform duration-150 {isDragging
				? 'text-[var(--primary)] scale-110'
				: 'text-[var(--text-muted)] group-hover:text-[var(--primary)] group-hover:scale-110'}"
		/>

		{#if file}
			<span class="text-xs font-semibold text-[var(--primary)] truncate max-w-xs">
				{file.name}
			</span>
			<span class="text-[11px] text-[var(--text-muted)] mt-0.5">
				{formatFileSize(file.size)}
			</span>
		{:else}
			{#if title}
				<span
					class="text-xs font-semibold transition-colors {isDragging
						? 'text-[var(--primary)]'
						: 'text-[var(--text-primary)] group-hover:text-[var(--primary)]'}"
				>
					{title}
				</span>
			{/if}
			{#if subtitle}
				<span class="text-[11px] text-[var(--text-muted)] mt-1">
					{subtitle}
				</span>
			{/if}
		{/if}
	{/if}
</div>
