<script lang="ts">
	import { page } from '$app/state';
	import ChatView from '$lib/components/ChatView.svelte';
	import { FeatureGate } from '$lib/components/ds';

	let chatId = $derived(page.params.id);
</script>

<FeatureGate features={['chat']}>
	<ChatView {chatId} />
</FeatureGate>
