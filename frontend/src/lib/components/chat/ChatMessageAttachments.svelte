<script lang="ts">
	import { Modal } from '$lib/components/ui';
	import type { ChatAttachment } from '$lib/types/chat';
	import { formatFileSize } from '$lib/utils/attachments';
	import { File, FileText, Image, Mic, Video } from '@lucide/svelte';

	interface ChatMessageAttachmentsProps {
		attachments: ChatAttachment[];
	}

	let { attachments = [] }: ChatMessageAttachmentsProps = $props();

	let lightboxSrc = $state<string | null>(null);
	let lightboxPrompt = $state<string>('');

	function openLightbox(src: string, name: string) {
		lightboxSrc = src;
		lightboxPrompt = name;
	}

	function closeLightbox() {
		lightboxSrc = null;
	}
</script>

{#if attachments && attachments.length > 0}
	<div class="flex flex-wrap items-center justify-end gap-2 my-1 max-w-full">
		{#each attachments as att (att.id)}
			{#if att.type === 'image'}
				<!-- svelte-ignore a11y_click_events_have_key_events -->
				<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
				<div
					class="relative rounded-2xl overflow-hidden border border-[var(--border-color)] bg-[var(--bg-surface)] shadow-md group/img cursor-pointer transition-all duration-200 hover:scale-[1.02] hover:border-[var(--border-hover)]"
					onclick={() => openLightbox(att.dataUrl || att.thumbnailUrl || '', att.name)}
					role="button"
					tabindex="0"
					aria-label={att.name}
				>
					<img
						src={att.thumbnailUrl || att.dataUrl}
						alt={att.name}
						class="max-w-[220px] max-h-[160px] sm:max-w-[280px] sm:max-h-[200px] w-auto h-auto object-cover block"
						loading="lazy"
					/>
					<div
						class="absolute inset-0 bg-black/40 opacity-0 group-hover/img:opacity-100 transition-opacity duration-150 flex items-end p-2"
					>
						<span class="text-white text-[0.7rem] truncate font-medium">
							{att.name}
						</span>
					</div>
				</div>
			{:else if att.type === 'video'}
				<div
					class="rounded-2xl overflow-hidden border border-[var(--border-color)] bg-[var(--bg-surface)] p-1 max-w-[320px]"
				>
					<!-- svelte-ignore a11y_media_has_caption -->
					<video
						src={att.dataUrl}
						controls
						preload="metadata"
						class="max-w-full max-h-[200px] rounded-xl block"
					></video>
				</div>
			{:else if att.type === 'audio'}
				<div
					class="rounded-xl border border-[var(--border-color)] bg-[var(--bg-surface)] py-2 px-3 flex items-center gap-2.5"
				>
					<div
						class="w-7 h-7 rounded-lg bg-[var(--bg-surface-hover)] flex items-center justify-center text-[var(--text-secondary)]"
					>
						<Mic size={15} />
					</div>
					<div class="flex flex-col min-w-0 max-w-[180px]">
						<span class="truncate text-xs font-medium text-[var(--text-primary)]" title={att.name}>
							{att.name}
						</span>
						<span class="text-[0.68rem] text-[var(--text-muted)]">
							{formatFileSize(att.size)}
						</span>
					</div>
					<!-- svelte-ignore a11y_media_has_caption -->
					<audio src={att.dataUrl} controls class="h-7 w-44"></audio>
				</div>
			{:else}
				<div
					class="rounded-xl border border-[var(--border-color)] bg-[var(--bg-surface)] py-2 px-3 flex items-center gap-2.5 shadow-sm"
				>
					<div
						class="w-7 h-7 rounded-lg bg-[var(--bg-surface-hover)] flex items-center justify-center text-[var(--text-secondary)]"
					>
						{#if att.type === 'text'}
							<FileText size={15} />
						{:else}
							<File size={15} />
						{/if}
					</div>
					<div class="flex flex-col min-w-0 max-w-[200px]">
						<span class="truncate text-xs font-medium text-[var(--text-primary)]" title={att.name}>
							{att.name}
						</span>
						<span class="text-[0.68rem] text-[var(--text-muted)]">
							{formatFileSize(att.size)}
						</span>
					</div>
				</div>
			{/if}
		{/each}
	</div>
{/if}

<Modal open={!!lightboxSrc} title={lightboxPrompt} maxWidth="xl" onclose={closeLightbox}>
	{#if lightboxSrc}
		<div class="flex items-center justify-center p-2 max-h-[75vh] overflow-hidden">
			<img
				src={lightboxSrc}
				alt={lightboxPrompt}
				class="max-w-full max-h-[70vh] object-contain rounded-xl"
			/>
		</div>
	{/if}
</Modal>
