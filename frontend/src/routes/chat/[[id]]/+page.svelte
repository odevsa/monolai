<script lang="ts">
	import { page } from '$app/state';
	import ChatView from '$lib/components/ChatView.svelte';

	let chatId = $derived(page.params.id);
</script>

<ChatView {chatId} />
