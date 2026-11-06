<script lang="ts">
	import { Button } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import type { ChatAttachment } from '$lib/types/chat';
	import { formatFileSize } from '$lib/utils/attachments';
	import { File, FileText, Mic, Video, X } from '@lucide/svelte';

	interface ChatAttachmentChipProps {
		attachment: ChatAttachment;
		onRemove: (id: string) => void;
	}

	let { attachment, onRemove }: ChatAttachmentChipProps = $props();
</script>

{#if attachment.type === 'image'}
	<div
		class="group/chip relative rounded-2xl overflow-hidden border border-[var(--border-color)] bg-[var(--bg-surface)] shadow-sm transition-all duration-150 hover:border-[var(--border-hover)] flex-1 min-w-[120px] max-w-[170px] h-[76px]"
	>
		<img
			src={attachment.thumbnailUrl || attachment.dataUrl}
			alt={attachment.name}
			class="w-full h-full object-cover block"
		/>
		<div class="absolute top-1.5 right-1.5">
			<Button
				variant="glass"
				size="icon-xs"
				class="!w-5 !h-5 !rounded-full !bg-black/60 hover:!bg-[var(--danger)] text-white hover:text-white border border-white/20 shadow-sm"
				onclick={(e) => {
					e.stopPropagation();
					onRemove(attachment.id);
				}}
				title={t('common.remove')}
				ariaLabel={t('common.remove')}
			>
				<X size={11} />
			</Button>
		</div>
	</div>
{:else}
	<div
		class="group/chip relative flex flex-col justify-between p-2.5 rounded-2xl bg-[var(--bg-surface)] border border-[var(--border-color)] text-[var(--text-primary)] shadow-sm transition-all duration-150 hover:border-[var(--border-hover)] flex-1 min-w-[130px] max-w-[200px] h-[76px]"
	>
		<div class="flex items-center justify-between w-full">
			<div
				class="w-6 h-6 rounded-lg bg-[var(--bg-surface-hover)] border border-[var(--border-color)]/60 flex items-center justify-center text-[var(--text-secondary)] shrink-0"
			>
				{#if attachment.type === 'audio'}
					<Mic size={14} />
				{:else if attachment.type === 'video'}
					<Video size={14} />
				{:else if attachment.type === 'text'}
					<FileText size={14} />
				{:else}
					<File size={14} />
				{/if}
			</div>

			<Button
				variant="ghost"
				size="icon-xs"
				class="!w-5 !h-5 !rounded-full !bg-[var(--bg-surface-hover)] hover:!bg-[var(--danger-light)] hover:!text-[var(--danger-text)] text-[var(--text-muted)] border border-[var(--border-color)]/50 transition-colors"
				onclick={(e) => {
					e.stopPropagation();
					onRemove(attachment.id);
				}}
				title={t('common.remove')}
				ariaLabel={t('common.remove')}
			>
				<X size={11} />
			</Button>
		</div>

		<div class="flex flex-col min-w-0 mt-0.5">
			<span
				class="truncate font-medium text-[0.78rem] leading-tight text-[var(--text-primary)]"
				title={attachment.name}
			>
				{attachment.name}
			</span>
			<span class="text-[0.68rem] text-[var(--text-muted)] leading-tight mt-0.5">
				{formatFileSize(attachment.size)}
			</span>
		</div>
	</div>
{/if}
