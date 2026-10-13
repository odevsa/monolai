<script lang="ts">
	import type { FeatureModelItem } from '$lib/api/models';
	import { ModelSelect } from '$lib/components/ui';
	import { t } from '$lib/i18n';

	interface ChatModelSelectorProps {
		selectedModel: string;
		models: FeatureModelItem[];
		onSelectModel: (modelId: string) => void;
	}

	let {
		selectedModel = $bindable(''),
		models = [],
		onSelectModel
	}: ChatModelSelectorProps = $props();
</script>

<ModelSelect
	bind:value={selectedModel}
	{models}
	placement="top"
	align="right"
	variant="pill"
	class="shrink min-w-0 max-w-[130px] sm:max-w-[260px] md:max-w-[360px]"
	onchange={onSelectModel}
	placeholder={t('common.selectModel')}
	label={t('common.selectActiveModel')}
	emptyText={t('common.noModelsRegistered')}
/>
