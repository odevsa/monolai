<script lang="ts">
	import { Badge, Button, EmptyState, Loader } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import { copyToClipboard } from '$lib/utils/clipboard';
	import { formatElapsedTime } from '$lib/utils/format';
	import {
		Check,
		Copy,
		Download,
		Headphones,
		Pause,
		Play,
		RotateCcw,
		Volume2,
		VolumeX
	} from '@lucide/svelte';
	import { onDestroy } from 'svelte';

	interface AudioPreviewCardProps {
		src?: string | null;
		title?: string;
		model?: string;
		durationSecs?: number;
		isGenerating?: boolean;
		onDownload?: () => void;
	}

	let {
		src = null,
		title = '',
		model = '',
		durationSecs = 0,
		isGenerating = false,
		onDownload = undefined
	}: AudioPreviewCardProps = $props();

	let audioEl = $state<HTMLAudioElement | null>(null);
	let isPlaying = $state(false);
	let currentTime = $state(0);
	let totalDuration = $state(0);
	let isMuted = $state(false);
	let volume = $state(1);
	let copied = $state(false);

	let effectiveDuration = $derived(totalDuration > 0 ? totalDuration : durationSecs);

	function togglePlay() {
		if (!audioEl || !src) return;
		if (isPlaying) {
			audioEl.pause();
		} else {
			audioEl.play().catch((err) => console.error('Audio play error:', err));
		}
	}

	function handleRestart() {
		if (!audioEl) return;
		audioEl.currentTime = 0;
		currentTime = 0;
		audioEl.play().catch((err) => console.error('Audio play error:', err));
	}

	function handleSeek(e: Event) {
		const target = e.target as HTMLInputElement;
		const val = parseFloat(target.value);
		if (audioEl && !isNaN(val)) {
			audioEl.currentTime = val;
			currentTime = val;
		}
	}

	function toggleMute() {
		if (!audioEl) return;
		isMuted = !isMuted;
		audioEl.muted = isMuted;
	}

	function handleVolumeChange(e: Event) {
		const target = e.target as HTMLInputElement;
		const val = parseFloat(target.value);
		if (audioEl && !isNaN(val)) {
			volume = val;
			audioEl.volume = val;
			if (val === 0) {
				isMuted = true;
				audioEl.muted = true;
			} else if (isMuted) {
				isMuted = false;
				audioEl.muted = false;
			}
		}
	}

	function formatTime(secs: number): string {
		if (!secs || isNaN(secs) || secs < 0) return '00:00';
		const m = Math.floor(secs / 60);
		const s = Math.floor(secs % 60);
		return `${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
	}

	function handleDownload() {
		if (onDownload) {
			onDownload();
			return;
		}
		if (!src) return;
		const a = document.createElement('a');
		a.href = src;
		a.download = `monolai-audio-${Date.now()}.wav`;
		document.body.appendChild(a);
		a.click();
		document.body.removeChild(a);
	}

	async function handleCopy() {
		if (!title) return;
		const success = await copyToClipboard(title);
		if (success) {
			copied = true;
			setTimeout(() => (copied = false), 2000);
		}
	}

	onDestroy(() => {
		if (audioEl) {
			audioEl.pause();
			audioEl.src = '';
		}
	});
</script>

{#if src}
	<audio
		bind:this={audioEl}
		{src}
		preload="metadata"
		onplay={() => (isPlaying = true)}
		onpause={() => (isPlaying = false)}
		onended={() => {
			isPlaying = false;
			currentTime = 0;
		}}
		ontimeupdate={() => {
			if (audioEl) currentTime = audioEl.currentTime;
		}}
		onloadedmetadata={() => {
			if (audioEl && !isNaN(audioEl.duration) && audioEl.duration !== Infinity) {
				totalDuration = audioEl.duration;
			}
		}}
	></audio>
{/if}

<div class="flex flex-col gap-4">
	<!-- Header with Action Toolbar (Mirroring ImagePreviewCard) -->
	<div class="flex items-center justify-between">
		<div class="flex items-center gap-2">
			<Headphones size={16} class="text-[var(--primary)] shrink-0" />
			<h3 class="m-0 text-sm font-bold text-[var(--text-primary)]">
				{t('audio.audioOutput')}
			</h3>
		</div>

		{#if src}
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

	<!-- Main Audio Display Canvas (Mirroring ImagePreviewDisplay) -->
	<div
		class="w-full min-h-[380px] max-h-[580px] rounded-lg bg-[var(--bg-primary)] border border-[var(--border-color)] flex flex-col items-center justify-center relative overflow-hidden group select-none p-6"
	>
		{#if isGenerating}
			<!-- Loading State -->
			<div class="flex flex-col items-center justify-center gap-3 p-8 text-center">
				<div class="relative w-16 h-16 flex items-center justify-center">
					<Loader size={64} />
				</div>
				<span class="text-xs text-[var(--text-muted)] animate-pulse">
					{t('audio.generatingAudio')}
				</span>
			</div>
		{:else if src}
			<!-- Center Play/Pause Control -->
			<div class="flex flex-col items-center justify-center gap-6 w-full max-w-md my-auto">
				<button
					type="button"
					onclick={togglePlay}
					class="w-20 h-20 rounded-full bg-[var(--primary)] text-[var(--bg-primary)] flex items-center justify-center hover:scale-105 active:scale-95 transition-all cursor-pointer shadow-lg focus:outline-hidden"
					aria-label={isPlaying ? t('audio.pause') : t('audio.play')}
				>
					{#if isPlaying}
						<Pause size={32} class="fill-current" />
					{:else}
						<Play size={32} class="fill-current translate-x-0.5" />
					{/if}
				</button>

				<!-- Seek Bar and Time Display -->
				<div class="w-full flex flex-col gap-1.5">
					<div class="relative w-full flex items-center">
						<input
							type="range"
							min="0"
							max={effectiveDuration || 100}
							step="0.05"
							value={currentTime}
							oninput={handleSeek}
							aria-label={t('audio.playbackProgress')}
							class="w-full h-1.5 rounded-lg appearance-none cursor-pointer bg-[var(--bg-hover)] accent-[var(--primary)] focus:outline-hidden"
						/>
					</div>
					<div
						class="flex items-center justify-between text-[11px] font-mono text-[var(--text-muted)]"
					>
						<span>{formatTime(currentTime)}</span>
						<span>{formatTime(effectiveDuration)}</span>
					</div>
				</div>

				<!-- Secondary Controls: Restart & Volume -->
				<div class="flex items-center justify-between w-full pt-1">
					<Button
						variant="ghost"
						size="sm"
						onclick={handleRestart}
						title={t('audio.restart')}
						ariaLabel={t('audio.restart')}
						class="gap-1.5 text-xs text-[var(--text-muted)] hover:text-[var(--text-primary)]"
					>
						<RotateCcw size={14} />
						<span>{t('audio.restart')}</span>
					</Button>

					<div class="flex items-center gap-1.5">
						<Button
							variant="ghost"
							size="icon-sm"
							onclick={toggleMute}
							title={isMuted ? t('audio.unmute') : t('audio.mute')}
							ariaLabel={isMuted ? t('audio.unmute') : t('audio.mute')}
						>
							{#if isMuted || volume === 0}
								<VolumeX size={15} class="shrink-0 text-[var(--text-muted)]" />
							{:else}
								<Volume2 size={15} class="shrink-0 text-[var(--text-muted)]" />
							{/if}
						</Button>
						<input
							type="range"
							min="0"
							max="1"
							step="0.05"
							value={isMuted ? 0 : volume}
							oninput={handleVolumeChange}
							aria-label={t('audio.volume')}
							class="w-16 sm:w-20 h-1 rounded-lg appearance-none cursor-pointer bg-[var(--bg-hover)] accent-[var(--primary)] focus:outline-hidden"
						/>
					</div>
				</div>
			</div>

			<!-- Meta Overlay Pill at Bottom -->
			<div
				class="absolute bottom-3 left-3 right-3 p-3 rounded-lg bg-[var(--bg-surface)]/90 backdrop-blur-md border border-[var(--border-color)]/80 flex items-center justify-between gap-3 text-xs pointer-events-auto"
			>
				<span class="truncate text-[var(--text-primary)] font-medium">
					"{title}"
				</span>
				<div class="flex items-center gap-2 shrink-0">
					{#if model}
						<Badge variant="muted" rounded="rounded-md">{model}</Badge>
					{/if}
					{#if effectiveDuration}
						<Badge variant="muted" rounded="rounded-md">
							{formatElapsedTime(effectiveDuration)}
						</Badge>
					{/if}
				</div>
			</div>
		{:else}
			<!-- Empty State Placeholder -->
			<EmptyState
				title={t('audio.noAudioGeneratedYet')}
				description={t('audio.ttsEmptyGuide')}
				icon={Headphones}
				class="border-0 bg-transparent max-w-sm"
			/>
		{/if}
	</div>
</div>
