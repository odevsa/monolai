<script lang="ts">
	import ModelForm from '$lib/components/ModelForm.svelte';
	import { t } from '$lib/i18n';
	import type { ModelFileItem, ModelRecord } from '$lib/types/models';
	import type { Runtime, RuntimeManifest } from '$lib/types/runtimes';
	import { Box } from '@lucide/svelte';

	let {
		runtimes = [],
		availableFiles = [],
		runtimeManifests = [],
		editingModel = null,
		onSave,
		onCancel
	}: {
		runtimes: Runtime[];
		availableFiles: ModelFileItem[];
		runtimeManifests: RuntimeManifest[];
		editingModel: ModelRecord | null;
		onSave: (payload: {
			id: string;
			runtime: string;
			flags: Record<string, string>;
		}) => Promise<void>;
		onCancel: () => void;
	} = $props();
</script>

<div class="bg-[var(--bg-surface)] rounded-2xl p-5 sm:p-7 shadow-sm">
	<div class="flex items-center gap-2.5 pb-4 mb-5 border-b border-[var(--border-color)]">
		<div
			class="w-8 h-8 rounded-xl bg-[var(--primary-light)] text-[var(--primary)] flex items-center justify-center border-0"
		>
			<Box size={16} />
		</div>
		<div>
			<h3 class="m-0 text-base font-bold text-[var(--text-primary)]">
				{editingModel ? `${t('models.editModel')}: ${editingModel.id}` : t('models.addModel')}
			</h3>
			<p class="m-0 text-xs text-[var(--text-muted)] mt-0.5">
				{t('models.formSubtitle')}
			</p>
		</div>
	</div>

	<ModelForm {runtimes} {availableFiles} {runtimeManifests} {editingModel} {onSave} {onCancel} />
</div>
