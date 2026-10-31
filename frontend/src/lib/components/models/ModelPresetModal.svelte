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
	import FlagRow from './FlagRow.svelte';

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

	let name = $state('default');
	let selectedRuntime = $state('');
	let isDefault = $state(true);
	let flagsList = $state<Array<{ key: string; value: string }>>([]);

	let compatibleRuntimes = $state<RuntimeManifest[]>([]);
	let isLoadingRuntimes = $state(true);
	let isSaving = $state(false);
	let errorMessage = $state<string | null>(null);

	$effect(() => {
		if (preset) {
			name = preset.name;
			selectedRuntime = preset.runtime_id;
			isDefault = preset.is_default;
		} else {
			name = 'default';
			isDefault = true;
		}
	});

	onMount(async () => {
		if (preset?.flags) {
			if (typeof preset.flags === 'object' && preset.flags !== null) {
				flagsList = Object.entries(preset.flags).map(([key, value]) => ({
					key,
					value: String(value)
				}));
			} else if (typeof preset.flags === 'string') {
				try {
					const parsed = JSON.parse(preset.flags);
					flagsList = Object.entries(parsed).map(([key, value]) => ({
						key,
						value: String(value)
					}));
				} catch {
					flagsList = [{ key: 'flags', value: preset.flags }];
				}
			}
		} else {
			flagsList = [
				{ key: '--ctx-size', value: '4096' },
				{ key: '--n-gpu-layers', value: '99' }
			];
		}

		try {
			compatibleRuntimes = await modelsApi.getCompatibleRuntimes(model.modality, model.format);
			if (!selectedRuntime && compatibleRuntimes.length > 0) {
				selectedRuntime = compatibleRuntimes[0].id;
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
		flagsList = [...flagsList, { key: '', value: '' }];
	}

	function removeFlag(index: number) {
		flagsList = flagsList.filter((_, i) => i !== index);
	}

	async function handleSave() {
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
			const k = item.key.trim();
			if (k) {
				flagsObj[k] = item.value.trim();
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
</script>

<Modal
	{open}
	title={preset ? t('models.presets.editPreset') : t('models.presets.addPreset')}
	{onclose}
>
	<div class="space-y-4">
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
				bind:value={name}
				placeholder="e.g. default, low-vram, long-context"
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
				<Select bind:value={selectedRuntime} options={runtimeOptions} />
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
				<Button variant="ghost" size="xs" onclick={addFlag}>
					<Plus size={12} />
					<span>{t('common.add')}</span>
				</Button>
			</div>

			<div class="space-y-1.5">
				{#each flagsList as item, index}
					<FlagRow
						bind:key={item.key}
						bind:value={item.value}
						{manifestFlags}
						onRemove={() => removeFlag(index)}
					/>
				{/each}
			</div>
		</div>

		<!-- Modal Actions -->
		<div class="flex items-center justify-end gap-2 pt-3 border-t border-[var(--border-color)]">
			<Button variant="secondary" onclick={onclose}>
				{t('common.cancel')}
			</Button>
			<Button
				variant="primary"
				onclick={handleSave}
				disabled={isSaving || !name.trim() || !selectedRuntime}
			>
				{#if isSaving}
					<Spinner size={14} />
				{/if}
				<span>{t('common.save')}</span>
			</Button>
		</div>
	</div>
</Modal>
