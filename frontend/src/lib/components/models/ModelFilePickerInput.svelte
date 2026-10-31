<script lang="ts">
	import { Button, Input } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import type { ModelFileItem } from '$lib/types/models';
	import type { ModelPickerFilter } from '$lib/utils/modelTree';
	import { FolderSearch, X } from '@lucide/svelte';
	import ModelFilePickerModal from './ModelFilePickerModal.svelte';

	let {
		value = $bindable(''),
		placeholder = '',
		filterMode = 'all',
		returnType = 'relative',
		disabled = false,
		id = '',
		class: className = '',
		onchange
	}: {
		value?: string;
		placeholder?: string;
		filterMode?: ModelPickerFilter;
		returnType?: 'relative' | 'absolute';
		disabled?: boolean;
		id?: string;
		class?: string;
		onchange?: (val: string) => void;
	} = $props();

	let isModalOpen = $state(false);

	function openModal() {
		if (disabled) return;
		isModalOpen = true;
	}

	function closeModal() {
		isModalOpen = false;
	}

	function handleSelect(newPath: string, _file: ModelFileItem) {
		value = newPath;
		if (onchange) {
			onchange(newPath);
		}
	}

	function handleClear(e: MouseEvent) {
		e.stopPropagation();
		value = '';
		if (onchange) {
			onchange('');
		}
	}
</script>

<div class="relative flex items-center w-full {className}" title={value || placeholder}>
	<div class="relative w-full">
		<Input
			{id}
			type="text"
			readonly
			{value}
			placeholder={placeholder || t('models.selectModelFile')}
			title={value || placeholder}
			class="pr-16 cursor-pointer select-none h-[34px]"
			inputClass="truncate font-mono text-xs"
			onclick={openModal}
			{disabled}
		/>

		<div class="absolute right-1 top-1/2 -translate-y-1/2 flex items-center gap-1">
			{#if value && !disabled}
				<button
					type="button"
					class="p-1 rounded text-[var(--text-muted)] hover:text-[var(--danger-text)] hover:bg-[var(--bg-surface-hover)] transition-colors cursor-pointer border-0 bg-transparent flex items-center justify-center"
					onclick={handleClear}
					title={t('common.clear')}
					aria-label={t('common.clear')}
				>
					<X size={12} />
				</button>
			{/if}

			<Button
				variant="secondary"
				size="icon-xs"
				onclick={openModal}
				{disabled}
				title={t('models.picker.title')}
				ariaLabel={t('models.picker.title')}
			>
				<FolderSearch size={13} class="text-[var(--primary)]" />
			</Button>
		</div>
	</div>

	<!-- Standalone Picker Modal triggered on demand -->
	<ModelFilePickerModal
		open={isModalOpen}
		{filterMode}
		{returnType}
		selectedPath={value}
		onSelect={handleSelect}
		onClose={closeModal}
	/>
</div>
