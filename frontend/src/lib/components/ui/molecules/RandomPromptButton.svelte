<script lang="ts">
	import { Button } from '$lib/components/ui/atoms';
	import { t } from '$lib/i18n';
	import { Dices } from '@lucide/svelte';

	let {
		onclick,
		disabled = false,
		title = t('image.randomPromptTitle'),
		label = t('common.random'),
		class: className = ''
	}: {
		onclick?: () => void;
		disabled?: boolean;
		title?: string;
		label?: string;
		class?: string;
	} = $props();
</script>

<Button
	variant="secondary"
	size="xs"
	{onclick}
	{disabled}
	{title}
	ariaLabel={title}
	class={className}
>
	<Dices size={13} class="text-[var(--primary)] shrink-0" />
	<span>{label}</span>
</Button>
