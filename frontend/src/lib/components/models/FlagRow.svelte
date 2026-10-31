<script lang="ts">
	import { Button, Input, Select, Toggle } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import type { SelectOption } from '$lib/components/ui/molecules/Select.svelte';
	import type { ManifestFlag } from '$lib/types/runtimes';
	import { portal } from '$lib/utils/portal';
	import {
		Box,
		Check,
		ChevronDown,
		Files,
		Hash,
		Layers,
		ToggleLeft,
		Trash,
		Type
	} from '@lucide/svelte';
	import { onMount } from 'svelte';
	import ModelFilePickerInput from './ModelFilePickerInput.svelte';

	export type FlagValueType = 'primary' | 'auxiliary' | 'file' | 'text' | 'number' | 'bool';

	let {
		key = $bindable(''),
		value = $bindable(''),
		manifestFlags = [],
		onRemove
	}: {
		key: string;
		value: string;
		manifestFlags?: ManifestFlag[];
		onRemove: () => void;
	} = $props();

	let customType = $state<FlagValueType | null>(null);
	let typeMenuOpen = $state(false);
	let triggerRef = $state<HTMLButtonElement | null>(null);
	let menuRef = $state<HTMLDivElement | null>(null);
	let menuCoords = $state({ top: 0, left: 0 });

	let flagOptions: SelectOption[] = $derived(
		manifestFlags.map((f) => ({
			value: f.flag,
			label: `${f.flag} (${f.name || f.flag})`,
			sublabel: f.description
		}))
	);

	let flagDef = $derived(manifestFlags.find((f) => f.flag === key));

	function detectMode(flagKey: string, flags: ManifestFlag[]): FlagValueType {
		const lower = flagKey.toLowerCase().trim();
		const def = flags.find((f) => f.flag.toLowerCase().trim() === lower);

		// Prioritize auxiliary assets (from manifest type, known flags, or descriptions)
		if (
			def?.type === 'file-auxiliary' ||
			def?.type === 'auxiliary' ||
			lower === '--mmproj' ||
			lower === '--vae' ||
			lower.includes('clip') ||
			lower.includes('t5xxl') ||
			lower.includes('vision') ||
			def?.description.toLowerCase().includes('mmproj') ||
			def?.description.toLowerCase().includes('vae') ||
			def?.description.toLowerCase().includes('clip')
		) {
			return 'auxiliary';
		}

		if (
			def?.type === 'bool' ||
			lower === '--embedding' ||
			lower.startsWith('--enable') ||
			lower.startsWith('--disable')
		) {
			return 'bool';
		}

		if (
			def?.type === 'int' ||
			def?.type === 'number' ||
			lower === '--threads' ||
			lower === '--ctx-size' ||
			lower === '--n-gpu-layers' ||
			lower === '--batch-size' ||
			lower === '--ubatch-size' ||
			lower === '--port'
		) {
			return 'number';
		}

		if (def?.type === 'file' || lower === '--model') {
			return 'primary';
		}

		return 'text';
	}

	let activeType: FlagValueType = $derived(customType ?? detectMode(key, manifestFlags));

	let isEnumType = $derived(
		activeType === 'text' &&
			flagDef?.type === 'enum' &&
			Array.isArray(flagDef.options) &&
			flagDef.options.length > 0
	);

	function updateMenuCoords() {
		if (!triggerRef) return;
		const rect = triggerRef.getBoundingClientRect();
		const menuWidth = 190;
		const menuHeight = 240;
		const spaceBelow = window.innerHeight - rect.bottom;

		let top = rect.bottom + 4;
		if (spaceBelow < menuHeight && rect.top > spaceBelow) {
			top = Math.max(8, rect.top - menuHeight - 4);
		}

		let left = Math.min(window.innerWidth - menuWidth - 8, Math.max(8, rect.right - menuWidth));
		menuCoords = { top, left };
	}

	function toggleTypeMenu() {
		if (!typeMenuOpen) {
			updateMenuCoords();
			typeMenuOpen = true;
		} else {
			typeMenuOpen = false;
		}
	}

	function setType(type: FlagValueType) {
		customType = type;
		typeMenuOpen = false;
		if (type === 'bool' && value !== 'true' && value !== 'false') {
			value = 'true';
		}
	}

	function handleClickOutside(event: MouseEvent) {
		const target = event.target as Node;
		if (triggerRef && !triggerRef.contains(target) && menuRef && !menuRef.contains(target)) {
			typeMenuOpen = false;
		}
	}

	onMount(() => {
		document.addEventListener('click', handleClickOutside, true);
		return () => {
			document.removeEventListener('click', handleClickOutside, true);
		};
	});
</script>

<div class="flex items-center gap-2 w-full py-1">
	<!-- Flag Key Writable Select (Manifest SSoT + Free Typing) -->
	<div class="w-1/2 min-w-0">
		<Select
			writable
			bind:value={key}
			options={flagOptions}
			placeholder={t('models.presets.selectFlagPlaceholder')}
			class="w-full"
			onchange={() => {
				// Reset override to adapt to the newly selected flag type
				customType = null;
			}}
		/>
	</div>

	<!-- Flag Value Dynamic Input -->
	<div class="w-1/2 min-w-0 flex items-center gap-1.5">
		{#if activeType === 'primary'}
			<ModelFilePickerInput
				bind:value
				filterMode="primary"
				placeholder={flagDef?.description || t('models.presets.flagValuePlaceholder')}
			/>
		{:else if activeType === 'auxiliary'}
			<ModelFilePickerInput
				bind:value
				filterMode="auxiliary"
				placeholder={flagDef?.description || t('models.presets.flagValuePlaceholder')}
			/>
		{:else if activeType === 'file'}
			<ModelFilePickerInput
				bind:value
				filterMode="all"
				placeholder={flagDef?.description || t('models.presets.flagValuePlaceholder')}
			/>
		{:else if activeType === 'bool'}
			<div
				class="flex items-center justify-between w-full h-[34px] px-3 py-1.5 rounded-lg bg-[var(--bg-primary)] border border-[var(--border-color)]"
			>
				<span class="text-xs text-[var(--text-secondary)] font-mono">
					{value === 'true' ? 'true' : 'false'}
				</span>
				<Toggle
					checked={value === 'true'}
					onchange={(checked) => {
						value = checked ? 'true' : 'false';
					}}
				/>
			</div>
		{:else if activeType === 'number'}
			<Input
				type="number"
				bind:value
				title={value}
				placeholder={flagDef?.default_value || '0'}
				class="w-full h-[34px]"
				inputClass="font-mono text-xs"
			/>
		{:else if isEnumType && flagDef?.options}
			<Select
				bind:value
				options={flagDef.options.map((opt: string) => ({ value: opt, label: opt }))}
				placeholder={t('models.presets.flagValuePlaceholder')}
				class="w-full"
			/>
		{:else}
			<Input
				type="text"
				bind:value
				title={value}
				placeholder={flagDef?.default_value || t('models.presets.flagValuePlaceholder')}
				class="w-full h-[34px]"
				inputClass="truncate font-mono text-xs"
			/>
		{/if}

		<!-- Value Type Selector Popover Trigger -->
		<button
			type="button"
			bind:this={triggerRef}
			onclick={toggleTypeMenu}
			title={t('models.presets.changeValueType')}
			aria-label={t('models.presets.changeValueType')}
			class="h-[34px] w-[34px] rounded-lg border border-[var(--border-color)] bg-[var(--bg-primary)] hover:bg-[var(--bg-surface-hover)] text-[var(--text-muted)] hover:text-[var(--text-primary)] transition-colors shrink-0 flex items-center justify-center cursor-pointer"
		>
			{#if activeType === 'primary'}
				<Box size={13} class="text-[var(--primary)]" />
			{:else if activeType === 'auxiliary'}
				<Layers size={13} class="text-[var(--info-text)]" />
			{:else if activeType === 'file'}
				<Files size={13} />
			{:else if activeType === 'bool'}
				<ToggleLeft size={13} class="text-[var(--success-text)]" />
			{:else if activeType === 'number'}
				<Hash size={13} />
			{:else}
				<Type size={13} />
			{/if}
		</button>

		<!-- Floating Value Type Popover Menu -->
		{#if typeMenuOpen}
			<div
				use:portal
				bind:this={menuRef}
				style="position: fixed; top: {menuCoords.top}px; left: {menuCoords.left}px; width: 190px; z-index: 9999;"
				class="p-1.5 rounded-xl shadow-2xl bg-[var(--bg-surface)] border border-[var(--border-color)] space-y-0.5 backdrop-blur-xl animate-in fade-in zoom-in-95 duration-100"
			>
				<div
					class="px-2 py-1 text-[10px] font-semibold text-[var(--text-muted)] uppercase tracking-wider"
				>
					{t('models.presets.changeValueType')}
				</div>

				<!-- Primary Model -->
				<button
					type="button"
					class="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg text-xs transition-colors cursor-pointer border-0 {activeType ===
					'primary'
						? 'bg-[var(--primary-light)] text-[var(--primary)] font-semibold'
						: 'bg-transparent text-[var(--text-secondary)] hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)]'}"
					onclick={() => setType('primary')}
				>
					<div class="flex items-center gap-2">
						<Box size={13} class="shrink-0 text-[var(--primary)]" />
						<span>{t('models.presets.typePrimary')}</span>
					</div>
					{#if activeType === 'primary'}
						<Check size={12} class="text-[var(--primary)]" />
					{/if}
				</button>

				<!-- Auxiliary Model -->
				<button
					type="button"
					class="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg text-xs transition-colors cursor-pointer border-0 {activeType ===
					'auxiliary'
						? 'bg-[var(--primary-light)] text-[var(--primary)] font-semibold'
						: 'bg-transparent text-[var(--text-secondary)] hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)]'}"
					onclick={() => setType('auxiliary')}
				>
					<div class="flex items-center gap-2">
						<Layers size={13} class="shrink-0 text-[var(--info-text)]" />
						<span>{t('models.presets.typeAuxiliary')}</span>
					</div>
					{#if activeType === 'auxiliary'}
						<Check size={12} class="text-[var(--primary)]" />
					{/if}
				</button>

				<!-- Any Model File -->
				<button
					type="button"
					class="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg text-xs transition-colors cursor-pointer border-0 {activeType ===
					'file'
						? 'bg-[var(--primary-light)] text-[var(--primary)] font-semibold'
						: 'bg-transparent text-[var(--text-secondary)] hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)]'}"
					onclick={() => setType('file')}
				>
					<div class="flex items-center gap-2">
						<Files size={13} class="shrink-0" />
						<span>{t('models.presets.typeAnyFile')}</span>
					</div>
					{#if activeType === 'file'}
						<Check size={12} class="text-[var(--primary)]" />
					{/if}
				</button>

				<!-- Text -->
				<button
					type="button"
					class="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg text-xs transition-colors cursor-pointer border-0 {activeType ===
					'text'
						? 'bg-[var(--primary-light)] text-[var(--primary)] font-semibold'
						: 'bg-transparent text-[var(--text-secondary)] hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)]'}"
					onclick={() => setType('text')}
				>
					<div class="flex items-center gap-2">
						<Type size={13} class="shrink-0" />
						<span>{t('models.presets.typeText')}</span>
					</div>
					{#if activeType === 'text'}
						<Check size={12} class="text-[var(--primary)]" />
					{/if}
				</button>

				<!-- Number -->
				<button
					type="button"
					class="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg text-xs transition-colors cursor-pointer border-0 {activeType ===
					'number'
						? 'bg-[var(--primary-light)] text-[var(--primary)] font-semibold'
						: 'bg-transparent text-[var(--text-secondary)] hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)]'}"
					onclick={() => setType('number')}
				>
					<div class="flex items-center gap-2">
						<Hash size={13} class="shrink-0" />
						<span>{t('models.presets.typeNumber')}</span>
					</div>
					{#if activeType === 'number'}
						<Check size={12} class="text-[var(--primary)]" />
					{/if}
				</button>

				<!-- Boolean -->
				<button
					type="button"
					class="w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg text-xs transition-colors cursor-pointer border-0 {activeType ===
					'bool'
						? 'bg-[var(--primary-light)] text-[var(--primary)] font-semibold'
						: 'bg-transparent text-[var(--text-secondary)] hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)]'}"
					onclick={() => setType('bool')}
				>
					<div class="flex items-center gap-2">
						<ToggleLeft size={13} class="shrink-0 text-[var(--success-text)]" />
						<span>{t('models.presets.typeBool')}</span>
					</div>
					{#if activeType === 'bool'}
						<Check size={12} class="text-[var(--primary)]" />
					{/if}
				</button>
			</div>
		{/if}

		<!-- Delete Row Button -->
		<Button
			variant="danger"
			size="icon"
			onclick={onRemove}
			title={t('common.delete')}
			ariaLabel={t('common.delete')}
			class="h-[34px] w-[34px] shrink-0"
		>
			<Trash size={13} />
		</Button>
	</div>
</div>
