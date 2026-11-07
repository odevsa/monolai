<script lang="ts">
	import { AttachmentChip } from '$lib/components/ui';
	import type { ChatAttachment } from '$lib/types/chat';

	interface ChatMessageAttachmentsProps {
		attachments: ChatAttachment[];
	}

	let { attachments = [] }: ChatMessageAttachmentsProps = $props();
</script>

{#if attachments && attachments.length > 0}
	<div class="flex flex-wrap items-center justify-end gap-2 my-1 max-w-full">
		{#each attachments as att (att.id)}
			<AttachmentChip attachment={att} variant="history" />
		{/each}
	</div>
{/if}
