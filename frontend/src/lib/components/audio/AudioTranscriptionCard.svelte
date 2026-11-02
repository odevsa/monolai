<script lang="ts">
	import { Badge, Button, EmptyState, Loader } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import { copyToClipboard } from '$lib/utils/clipboard';
	import { Check, Copy, Download, Mic, Sparkles } from '@lucide/svelte';

	interface AudioTranscriptionCardProps {
		transcript?: string;
		model?: string;
		isTranscribing?: boolean;
		onDownload?: () => void;
	}

	let {
		transcript = '',
		model = '',
		isTranscribing = false,
		onDownload = undefined
	}: AudioTranscriptionCardProps = $props();

	let copied = $state(false);

	let wordCount = $derived(
		transcript && transcript.trim() ? transcript.trim().split(/\s+/).length : 0
	);
	let charCount = $derived(transcript ? transcript.length : 0);

	async function handleCopy() {
		if (!transcript) return;
		const success = await copyToClipboard(transcript);
		if (success) {
			copied = true;
			setTimeout(() => (copied = false), 2000);
		}
	}

	function handleDownload() {
		if (onDownload) {
			onDownload();
			return;
		}
		if (!transcript) return;
		const blob = new Blob([transcript], { type: 'text/plain;charset=utf-8' });
		const url = URL.createObjectURL(blob);
		const a = document.createElement('a');
		a.href = url;
		a.download = `transcript-${Date.now()}.txt`;
		document.body.appendChild(a);
		a.click();
		document.body.removeChild(a);
		URL.revokeObjectURL(url);
	}
</script>

<div class="flex flex-col gap-4">
	<!-- Header with Action Toolbar (Mirroring ImagePreviewCard) -->
	<div class="flex items-center justify-between">
		<div class="flex items-center gap-2">
			<Sparkles size={16} class="text-[var(--primary)] shrink-0" />
			<h3 class="m-0 text-sm font-bold text-[var(--text-primary)]">
				{t('audio.transcriptionResult')}
			</h3>
		</div>

		{#if transcript}
			<div class="flex items-center gap-1.5 ml-auto">
				<Button
					variant="secondary"
					size="icon-sm"
					onclick={handleDownload}
					title={t('common.download')}
					ariaLabel={t('common.download')}
				>
					<Download size={13} />
				</Button>

				<Button
					variant="secondary"
					size="icon-sm"
					onclick={handleCopy}
					title={t('common.copy')}
					ariaLabel={t('common.copy')}
				>
					{#if copied}
						<Check size={13} class="text-[var(--success-text)]" />
					{:else}
						<Copy size={13} />
					{/if}
				</Button>
			</div>
		{/if}
	</div>

	<!-- Main Transcription Display Canvas (Mirroring ImagePreviewDisplay) -->
	<div
		class="w-full min-h-[380px] max-h-[580px] rounded-lg bg-[var(--bg-primary)] border border-[var(--border-color)] flex flex-col items-center justify-center relative overflow-hidden group p-6"
	>
		{#if isTranscribing}
			<!-- Loading State -->
			<div class="flex flex-col items-center justify-center gap-3 p-8 text-center">
				<div class="relative w-16 h-16 flex items-center justify-center">
					<Loader size={64} />
				</div>
				<span class="text-xs text-[var(--text-muted)] animate-pulse">
					{t('audio.transcribingAudio')}
				</span>
			</div>
		{:else if transcript}
			<!-- Transcript Text Content Area -->
			<div
				class="w-full h-full min-h-[260px] max-h-[460px] overflow-y-auto text-sm text-[var(--text-primary)] leading-relaxed whitespace-pre-wrap select-text font-sans p-4 pb-16 rounded-lg bg-[var(--bg-surface)]/60 border border-[var(--border-color)]/60"
			>
				{transcript}
			</div>

			<!-- Meta Overlay Pill at Bottom -->
			<div
				class="absolute bottom-3 left-3 right-3 p-3 rounded-lg bg-[var(--bg-surface)]/90 backdrop-blur-md border border-[var(--border-color)]/80 flex items-center justify-between gap-3 text-xs pointer-events-auto"
			>
				<div class="flex items-center gap-2 text-[var(--text-muted)]">
					<span>{wordCount} {t('audio.words')}</span>
					<span>•</span>
					<span>{charCount} {t('audio.characters')}</span>
				</div>
				{#if model}
					<Badge variant="muted" rounded="rounded-md">{model}</Badge>
				{/if}
			</div>
		{:else}
			<!-- Empty State Placeholder -->
			<EmptyState
				title={t('audio.noTranscriptionYet')}
				description={t('audio.sttEmptyGuide')}
				icon={Mic}
				class="border-0 bg-transparent max-w-sm"
			/>
		{/if}
	</div>
</div>
