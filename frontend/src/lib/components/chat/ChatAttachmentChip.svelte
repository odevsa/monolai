<script lang="ts">
	import { AttachmentChip } from '$lib/components/ui';
	import type { ChatAttachment } from '$lib/types/chat';

	interface ChatAttachmentChipProps {
		attachment: ChatAttachment;
		onRemove: (id: string) => void;
		disabled?: boolean;
	}

	let { attachment, onRemove, disabled = false }: ChatAttachmentChipProps = $props();
</script>

<AttachmentChip {attachment} variant="prompt" {disabled} {onRemove} />
