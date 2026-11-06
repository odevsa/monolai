<script lang="ts">
	import { modelsApi } from '$lib/api/models';
	import { Alert, Button, Input, Modal, Select, Spinner, Toggle } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import type {
		CreatePresetPayload,
		ModelPreset,
		ModelRecord,
		UpdatePresetPayload
	} from '$lib/types/models';
	import type { RuntimeManifest } from '$lib/types/runtimes';
	import { Plus, SlidersVertical } from '@lucide/svelte';
	import { onMount } from 'svelte';
	import { generateUUID } from '$lib/utils/common';
	import FlagRow from './FlagRow.svelte';

	interface FlagItem {
		id: string;
		key: string;
		value: string;
		disabled?: boolean;
	}

	let {
		open = false,
		model,
		preset = null,
		onclose,
		onSaved
	}: {
		open: boolean;
		model: ModelRecord;
		preset?: ModelPreset | null;
		onclose: () => void;
		onSaved: () => void;
	} = $props();

	let name = $state('');
	let nameInputRef = $state<HTMLInputElement | null>(null);
	let selectedRuntime = $state('');
	let previousRuntime = $state('');
	let isDefault = $state(true);
	let flagsList = $state<FlagItem[]>([]);

	$effect(() => {
		if (open) {
			requestAnimationFrame(() => {
				nameInputRef?.focus();
			});
		}
	});

	let compatibleRuntimes = $state<RuntimeManifest[]>([]);
	let isLoadingRuntimes = $state(true);
	let isSaving = $state(false);
	let errorMessage = $state<string | null>(null);

	function getModelFlagKey(
		runtimeId: string,
		runtimesList: RuntimeManifest[] = compatibleRuntimes
	): string {
		const manifest = runtimesList.find((r) => r.id === runtimeId);
		const modelFlag =
			manifest?.flags?.find((f) => f.model) ||
			manifest?.flags?.find(
				(f) => f.type === 'file' || f.flag === '--model' || f.flag === '--checkpoint'
			);
		return modelFlag?.flag || '--model';
	}

	function createModelFlagItem(
		runtimeId: string,
		runtimesList: RuntimeManifest[] = compatibleRuntimes
	): FlagItem {
		const key = getModelFlagKey(runtimeId, runtimesList);
		const val = model.relative_path || model.filename || '';
		return {
			id: 'model-flag',
			key,
			value: val,
			disabled: true
		};
	}

	function getImportantFlagsForRuntime(
		runtimeId: string,
		runtimesList: RuntimeManifest[] = compatibleRuntimes
	): FlagItem[] {
		const modelItem = createModelFlagItem(runtimeId, runtimesList);
		const manifest = runtimesList.find((r) => r.id === runtimeId);
		if (!manifest) return [modelItem];

		const important = manifest.flags?.filter((f) => f.important && f.flag !== modelItem.key) || [];
		const otherItems = important.map((f) => ({
			id: generateUUID(),
			key: f.flag,
			value: f.default_value ?? ''
		}));

		return [modelItem, ...otherItems];
	}

	function parsePresetFlags(
		presetFlags: unknown,
		runtimeId: string,
		runtimesList: RuntimeManifest[] = compatibleRuntimes
	): FlagItem[] {
		const modelItem = createModelFlagItem(runtimeId, runtimesList);
		let rawEntries: [string, string][] = [];

		if (presetFlags && typeof presetFlags === 'object') {
			rawEntries = Object.entries(presetFlags as Record<string, unknown>).map(([k, v]) => [
				k,
				String(v)
			]);
		} else if (typeof presetFlags === 'string') {
			try {
				const parsed = JSON.parse(presetFlags);
				rawEntries = Object.entries(parsed).map(([k, v]) => [k, String(v)]);
			} catch {
				rawEntries = [['flags', presetFlags]];
			}
		}

		const otherEntries = rawEntries.filter(
			([k]) => k !== modelItem.key && k !== '--model' && k !== '--checkpoint' && k !== '-m'
		);

		const otherItems = otherEntries.map(([k, v]) => ({
			id: generateUUID(),
			key: k,
			value: v
		}));

		return [modelItem, ...otherItems];
	}

	function onRuntimeChange(newRuntimeId: string) {
		if (newRuntimeId === previousRuntime) return;
		previousRuntime = newRuntimeId;
		selectedRuntime = newRuntimeId;
		flagsList = getImportantFlagsForRuntime(newRuntimeId);
	}

	onMount(async () => {
		if (preset) {
			name = preset.name;
			selectedRuntime = preset.runtime_id;
			previousRuntime = preset.runtime_id;
			isDefault = preset.is_default;
			flagsList = parsePresetFlags(preset.flags, selectedRuntime, compatibleRuntimes);
		} else {
			name = '';
			const hasDefault = model.presets?.some((p) => p.is_default);
			isDefault = !hasDefault;
		}

		try {
			compatibleRuntimes = await modelsApi.getCompatibleRuntimes(model.modality, model.format);
			if (!selectedRuntime && compatibleRuntimes.length > 0) {
				selectedRuntime = compatibleRuntimes[0].id;
				previousRuntime = compatibleRuntimes[0].id;
			}

			if (preset) {
				flagsList = parsePresetFlags(preset.flags, selectedRuntime, compatibleRuntimes);
			} else if (selectedRuntime) {
				flagsList = getImportantFlagsForRuntime(selectedRuntime, compatibleRuntimes);
			}
		} catch (err: unknown) {
			console.error('Failed to load compatible runtimes:', err);
		} finally {
			isLoadingRuntimes = false;
		}
	});

	let runtimeOptions = $derived(
		compatibleRuntimes.map((r) => ({
			value: r.id,
			label: `${r.name} (${r.id})`
		}))
	);

	let manifestFlags = $derived(
		compatibleRuntimes.find((r) => r.id === selectedRuntime)?.flags || []
	);

	function addFlag() {
		flagsList = [...flagsList, { id: generateUUID(), key: '', value: '' }];
	}

	function removeFlag(index: number) {
		if (flagsList[index]?.disabled) return;
		flagsList = flagsList.filter((_, i) => i !== index);
	}

	async function handleSave() {
		if (isSaving) return;
		if (!name.trim()) {
			errorMessage = t('models.presets.nameRequired');
			return;
		}
		if (!selectedRuntime) {
			errorMessage = t('models.presets.runtimeRequired');
			return;
		}

		isSaving = true;
		errorMessage = null;

		const flagsObj: Record<string, string> = {};
		for (const item of flagsList) {
			const k = String(item.key || '').trim();
			if (k) {
				const val = item.value != null ? String(item.value).trim() : '';
				flagsObj[k] = val;
			}
		}

		try {
			if (preset) {
				const payload: UpdatePresetPayload = {
					name: name.trim(),
					runtime_id: selectedRuntime,
					flags: flagsObj,
					is_default: isDefault
				};
				await modelsApi.updatePreset(preset.id, payload);
			} else {
				const payload: CreatePresetPayload = {
					name: name.trim(),
					runtime_id: selectedRuntime,
					flags: flagsObj,
					is_default: isDefault
				};
				await modelsApi.createPreset(model.id, payload);
			}
			onSaved();
		} catch (err: unknown) {
			const msg = err instanceof Error ? err.message : String(err);
			errorMessage = msg;
		} finally {
			isSaving = false;
		}
	}

	function handleInputKeyDown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			e.preventDefault();
			handleSave();
		}
	}
</script>

<Modal
	{open}
	title={preset ? t('models.presets.editPreset') : t('models.presets.addPreset')}
	{onclose}
>
	<form
		onsubmit={(e) => {
			e.preventDefault();
			handleSave();
		}}
		class="space-y-4"
	>
		{#if errorMessage}
			<Alert variant="error" title={t('common.error')}>
				{errorMessage}
			</Alert>
		{/if}

		<div>
			<label for="preset-name" class="text-xs font-medium text-[var(--text-secondary)] block mb-1">
				{t('models.presets.presetName')}
			</label>
			<Input
				id="preset-name"
				bind:ref={nameInputRef}
				bind:value={name}
				placeholder="e.g. default, low-vram, long-context"
				autofocus
				onkeydown={handleInputKeyDown}
			/>
		</div>

		<div>
			<label
				for="preset-runtime"
				class="text-xs font-medium text-[var(--text-secondary)] block mb-1"
			>
				{t('models.presets.runtime')}
			</label>
			{#if isLoadingRuntimes}
				<div class="flex items-center gap-2 text-xs text-[var(--text-muted)] py-2">
					<Spinner size={14} />
					<span>{t('common.loading')}</span>
				</div>
			{:else if compatibleRuntimes.length === 0}
				<div class="text-xs text-[var(--warning-text)] bg-[var(--warning-light)] p-2.5 rounded-lg">
					{t('models.presets.noCompatibleRuntimes', {
						modality: model.modality,
						format: model.format
					})}
				</div>
			{:else}
				<Select bind:value={selectedRuntime} options={runtimeOptions} onchange={onRuntimeChange} />
			{/if}
		</div>

		<div class="flex items-center justify-between py-1">
			<span class="text-xs font-semibold text-[var(--text-primary)]">
				{t('models.presets.isDefault')}
			</span>
			<Toggle bind:checked={isDefault} />
		</div>

		<!-- Flags editor -->
		<div class="space-y-2">
			<div class="flex items-center justify-between">
				<label class="text-xs font-semibold text-[var(--text-primary)] flex items-center gap-1.5">
					<SlidersVertical size={13} />
					<span>{t('models.presets.flags')}</span>
				</label>
			</div>

			<div class="space-y-1.5">
				{#each flagsList as item, index (item.id)}
					<FlagRow
						bind:key={item.key}
						bind:value={item.value}
						{manifestFlags}
						disabled={item.disabled}
						onRemove={() => removeFlag(index)}
					/>
				{/each}
			</div>

			<Button variant="primary" size="xs" onclick={addFlag}>
				<Plus size={12} />
				<span>{t('common.add')}</span>
			</Button>
		</div>

		<!-- Modal Actions -->
		<div class="flex items-center justify-end gap-2 pt-3 border-t border-[var(--border-color)]">
			<Button variant="secondary" onclick={onclose}>
				{t('common.cancel')}
			</Button>
			<Button
				type="submit"
				variant="primary"
				disabled={isSaving || !name.trim() || !selectedRuntime}
			>
				{#if isSaving}
					<Spinner size={14} />
				{/if}
				<span>{t('common.save')}</span>
			</Button>
		</div>
	</form>
</Modal>
