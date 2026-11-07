<script lang="ts">
	import { Button } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import type { ChatAttachment } from '$lib/types/chat';
	import { formatFileSize } from '$lib/utils/attachments';
	import { File, FileCode, FileText, Mic, Video, X } from '@lucide/svelte';
	import AudioPlayer from './AudioPlayer.svelte';
	import CodeBlock from './CodeBlock.svelte';
	import Modal from '../organisms/Modal.svelte';

	interface AttachmentChipProps {
		attachment: ChatAttachment;
		variant?: 'history' | 'prompt';
		disabled?: boolean;
		onRemove?: (id: string) => void;
		class?: string;
	}

	let {
		attachment,
		variant = 'history',
		disabled = false,
		onRemove,
		class: className = ''
	}: AttachmentChipProps = $props();

	let isPreviewOpen = $state(false);

	function detectFileType(att: ChatAttachment): 'image' | 'text' | 'audio' | 'video' | 'file' {
		if (att.type && att.type !== 'file') return att.type;
		const mime = (att.mimeType || '').toLowerCase();
		const name = (att.name || '').toLowerCase();
		if (mime.startsWith('image/') || /\.(png|jpe?g|webp|gif|svg|bmp|ico)$/i.test(name))
			return 'image';
		if (mime.startsWith('audio/') || /\.(mp3|wav|ogg|m4a|aac|flac)$/i.test(name)) return 'audio';
		if (mime.startsWith('video/') || /\.(mp4|webm|mkv|mov|avi)$/i.test(name)) return 'video';
		if (
			mime.startsWith('text/') ||
			att.textContent !== undefined ||
			/\.(txt|md|rs|py|js|ts|jsx|tsx|json|csv|log|yaml|yml|toml|html|css|scss|c|cpp|h|hpp|go|sh|bash|sql|xml|env|ini)$/i.test(
				name
			)
		) {
			return 'text';
		}
		return 'file';
	}

	let fileType = $derived(detectFileType(attachment));

	function getCodeLang(filename: string): string {
		const ext = filename.split('.').pop()?.toLowerCase() || '';
		const map: Record<string, string> = {
			rs: 'rust',
			py: 'python',
			js: 'javascript',
			ts: 'typescript',
			jsx: 'jsx',
			tsx: 'tsx',
			json: 'json',
			md: 'markdown',
			html: 'html',
			css: 'css',
			scss: 'scss',
			sh: 'bash',
			bash: 'bash',
			yaml: 'yaml',
			yml: 'yaml',
			toml: 'toml',
			sql: 'sql',
			c: 'c',
			cpp: 'cpp',
			h: 'c',
			hpp: 'cpp',
			go: 'go',
			xml: 'xml'
		};
		return map[ext] || ext || 'text';
	}

	function decodeDataUrl(dataUrl: string): string {
		try {
			const base64 = dataUrl.split(',')[1];
			if (!base64) return '';
			const bin = atob(base64);
			const bytes = Uint8Array.from(bin, (c) => c.charCodeAt(0));
			return new TextDecoder().decode(bytes);
		} catch {
			return '';
		}
	}

	let resolvedTextContent = $derived.by(() => {
		if (attachment.textContent) return attachment.textContent;
		if (fileType === 'text' && attachment.dataUrl?.startsWith('data:')) {
			return decodeDataUrl(attachment.dataUrl);
		}
		return '';
	});

	let modalMaxWidth = $derived<'md' | 'xl' | '2xl'>(
		fileType === 'text' ? '2xl' : fileType === 'audio' ? 'md' : 'xl'
	);

	function handleOpenPreview() {
		isPreviewOpen = true;
	}

	function handleRemoveClick(e: MouseEvent) {
		e.stopPropagation();
		if (onRemove) {
			onRemove(attachment.id);
		}
	}
</script>

<!-- Attachment Card Chip (Exact Same Size across all types & variants: 170px x 64px) -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
	role="button"
	tabindex="0"
	onclick={handleOpenPreview}
	class="group/chip relative w-[170px] h-[64px] shrink-0 rounded-2xl overflow-hidden border border-[var(--border-color)] bg-[var(--bg-surface)] shadow-sm transition-all duration-150 hover:border-[var(--border-hover)] hover:shadow-md cursor-pointer select-none {disabled
		? 'opacity-60 pointer-events-none'
		: ''} {className}"
	title={attachment.name}
>
	{#if fileType === 'image'}
		<img
			src={attachment.thumbnailUrl || attachment.dataUrl}
			alt={attachment.name}
			class="w-full h-full object-cover block"
			loading="lazy"
		/>

		<!-- Gradient Overlay for subtle title visibility on hover -->
		<div
			class="absolute inset-0 bg-gradient-to-t from-black/60 via-transparent to-transparent opacity-0 group-hover/chip:opacity-100 transition-opacity duration-150 flex items-end p-2 pointer-events-none"
		>
			<span class="text-white text-[0.72rem] truncate font-medium drop-shadow-sm">
				{attachment.name}
			</span>
		</div>

		<!-- Remove button (prompt variant) -->
		{#if variant === 'prompt'}
			<div class="absolute top-1.5 right-1.5 z-10">
				<Button
					variant="glass"
					size="icon-xs"
					class="!w-5 !h-5 !rounded-full !bg-black/60 hover:!bg-[var(--danger)] text-white hover:text-white border border-white/20 shadow-sm"
					onclick={handleRemoveClick}
					{disabled}
					title={t('common.remove')}
					ariaLabel={t('common.remove')}
				>
					<X size={11} />
				</Button>
			</div>
		{/if}
	{:else}
		<!-- Non-image layout (Text, Audio, Video, Generic File) -->
		<div class="flex items-center gap-2.5 h-full px-3 py-2">
			<!-- Icon positioned to the left of the filename -->
			<div
				class="w-8 h-8 rounded-xl bg-[var(--bg-surface-hover)] border border-[var(--border-color)]/60 flex items-center justify-center text-[var(--text-secondary)] shrink-0"
			>
				{#if fileType === 'audio'}
					<Mic size={15} class="text-[var(--primary)]" />
				{:else if fileType === 'video'}
					<Video size={15} class="text-[var(--info-text)]" />
				{:else if fileType === 'text'}
					<FileCode size={15} class="text-[var(--success-text)]" />
				{:else}
					<File size={15} />
				{/if}
			</div>

			<!-- Filename & size stacked on the right side of the icon -->
			<div class="flex flex-col min-w-0 flex-1 justify-center">
				<span
					class="truncate font-medium text-[0.78rem] leading-tight text-[var(--text-primary)]"
					title={attachment.name}
				>
					{attachment.name}
				</span>
				<span class="text-[0.68rem] text-[var(--text-muted)] mt-0.5">
					{formatFileSize(attachment.size)}
				</span>
			</div>

			<!-- Remove button (prompt variant) -->
			{#if variant === 'prompt'}
				<Button
					variant="ghost"
					size="icon-xs"
					class="!w-5 !h-5 !rounded-full !bg-[var(--bg-surface-hover)] hover:!bg-[var(--danger-light)] hover:!text-[var(--danger-text)] text-[var(--text-muted)] border border-[var(--border-color)]/50 transition-colors shrink-0"
					onclick={handleRemoveClick}
					{disabled}
					title={t('common.remove')}
					ariaLabel={t('common.remove')}
				>
					<X size={11} />
				</Button>
			{/if}
		</div>
	{/if}
</div>

<!-- Modal Preview Dialog (Screenshot 2) -->
<Modal
	bind:open={isPreviewOpen}
	title={attachment.name}
	maxWidth={modalMaxWidth}
	onclose={() => (isPreviewOpen = false)}
>
	<div class="w-full flex flex-col items-center justify-center min-h-[140px]">
		{#if fileType === 'image'}
			<div class="w-full flex items-center justify-center p-1">
				<img
					src={attachment.dataUrl || attachment.thumbnailUrl}
					alt={attachment.name}
					class="max-h-[75vh] w-auto max-w-full object-contain rounded-xl block shadow-lg border border-[var(--border-color)]"
				/>
			</div>
		{:else if fileType === 'text'}
			<div class="w-full max-h-[70vh] overflow-y-auto rounded-xl">
				<CodeBlock code={resolvedTextContent} lang={getCodeLang(attachment.name)} />
			</div>
		{:else if fileType === 'audio'}
			<div class="w-full py-6 px-4 flex flex-col items-center justify-center">
				<AudioPlayer src={attachment.dataUrl} variant="default" class="w-full max-w-[520px]" />
				<div class="text-xs text-[var(--text-muted)] mt-3">
					{attachment.name} ({formatFileSize(attachment.size)})
				</div>
			</div>
		{:else if fileType === 'video'}
			<div class="w-full flex items-center justify-center p-1">
				<!-- svelte-ignore a11y_media_has_caption -->
				<video
					src={attachment.dataUrl}
					controls
					class="max-h-[75vh] max-w-full rounded-xl block shadow-lg border border-[var(--border-color)]"
				></video>
			</div>
		{:else}
			<!-- Generic file download fallback -->
			<div class="flex flex-col items-center justify-center py-8 gap-3">
				<div
					class="w-12 h-12 rounded-2xl bg-[var(--bg-surface-hover)] border border-[var(--border-color)] flex items-center justify-center text-[var(--text-secondary)]"
				>
					<File size={24} />
				</div>
				<div class="text-sm font-medium text-[var(--text-primary)]">
					{attachment.name}
				</div>
				<div class="text-xs text-[var(--text-muted)]">
					{formatFileSize(attachment.size)}
				</div>
				{#if attachment.dataUrl}
					<a
						href={attachment.dataUrl}
						download={attachment.name}
						class="mt-2 px-4 py-2 rounded-xl text-xs font-medium bg-[var(--primary)] text-white hover:bg-[var(--primary-hover)] transition-colors inline-flex items-center gap-1.5"
					>
						{t('common.download')}
					</a>
				{/if}
			</div>
		{/if}
	</div>
</Modal>
