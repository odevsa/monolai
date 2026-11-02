<script lang="ts">
	import { Badge, Button, Card } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import { formatElapsedTime } from '$lib/utils/format';
	import { Download, Pause, Play, RotateCcw, Volume2, VolumeX } from '@lucide/svelte';
	import { onDestroy } from 'svelte';

	interface AudioPlayerCardProps {
		src?: string | null;
		title?: string;
		model?: string;
		durationSecs?: number;
		onDownload?: () => void;
	}

	let {
		src = null,
		title = '',
		model = '',
		durationSecs = 0,
		onDownload = undefined
	}: AudioPlayerCardProps = $props();

	let audioEl = $state<HTMLAudioElement | null>(null);
	let isPlaying = $state(false);
	let currentTime = $state(0);
	let totalDuration = $state(0);
	let isMuted = $state(false);
	let volume = $state(1);

	let effectiveDuration = $derived(totalDuration > 0 ? totalDuration : durationSecs);
	let progressPercent = $derived(
		effectiveDuration > 0 ? Math.min(100, (currentTime / effectiveDuration) * 100) : 0
	);

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
		a.download = `audio-${Date.now()}.wav`;
		document.body.appendChild(a);
		a.click();
		document.body.removeChild(a);
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

<Card class="p-4 sm:p-5 flex flex-col gap-3.5">
	<!-- Header: Title and Model Badge -->
	<div class="flex items-center justify-between gap-3">
		<div class="flex items-center gap-2 min-w-0">
			<div
				class="w-2.5 h-2.5 rounded-full shrink-0 {isPlaying
					? 'bg-[var(--success)] animate-pulse'
					: 'bg-[var(--text-muted)]'}"
			></div>
			<span class="text-sm font-semibold text-[var(--text-primary)] truncate">
				{title || t('audio.audioOutput')}
			</span>
		</div>
		<div class="flex items-center gap-2 shrink-0">
			{#if model}
				<Badge variant="muted" class="text-xs">{model}</Badge>
			{/if}
			{#if src}
				<Button
					variant="ghost"
					size="icon-sm"
					onclick={handleDownload}
					title={t('common.download')}
					ariaLabel={t('common.download')}
				>
					<Download size={15} />
				</Button>
			{/if}
		</div>
	</div>

	<!-- Seek Bar & Timing -->
	<div class="flex flex-col gap-1.5">
		<div class="relative w-full flex items-center">
			<input
				type="range"
				min="0"
				max={effectiveDuration || 100}
				step="0.05"
				value={currentTime}
				oninput={handleSeek}
				disabled={!src}
				aria-label={t('audio.playbackProgress')}
				class="w-full h-1.5 rounded-lg appearance-none cursor-pointer bg-[var(--bg-hover)] accent-[var(--primary)] focus:outline-hidden disabled:opacity-40"
			/>
		</div>
		<div class="flex items-center justify-between text-[11px] font-mono text-[var(--text-muted)]">
			<span>{formatTime(currentTime)}</span>
			<span>{formatTime(effectiveDuration)}</span>
		</div>
	</div>

	<!-- Controls Bar -->
	<div class="flex items-center justify-between pt-1">
		<div class="flex items-center gap-2">
			<!-- Play/Pause -->
			<Button
				variant="primary"
				size="sm"
				disabled={!src}
				onclick={togglePlay}
				class="gap-1.5 px-3"
				ariaLabel={isPlaying ? t('audio.pause') : t('audio.play')}
			>
				{#if isPlaying}
					<Pause size={15} />
					<span>{t('audio.pause')}</span>
				{:else}
					<Play size={15} />
					<span>{t('audio.play')}</span>
				{/if}
			</Button>

			<!-- Restart -->
			<Button
				variant="ghost"
				size="icon-sm"
				disabled={!src}
				onclick={handleRestart}
				title={t('audio.restart')}
				ariaLabel={t('audio.restart')}
			>
				<RotateCcw size={14} />
			</Button>
		</div>

		<!-- Volume Control -->
		<div class="flex items-center gap-1.5">
			<Button
				variant="ghost"
				size="icon-sm"
				onclick={toggleMute}
				disabled={!src}
				title={isMuted ? t('audio.unmute') : t('audio.mute')}
				ariaLabel={isMuted ? t('audio.unmute') : t('audio.mute')}
			>
				{#if isMuted || volume === 0}
					<VolumeX size={15} class="shrink-0" />
				{:else}
					<Volume2 size={15} class="shrink-0" />
				{/if}
			</Button>
			<input
				type="range"
				min="0"
				max="1"
				step="0.05"
				value={isMuted ? 0 : volume}
				oninput={handleVolumeChange}
				disabled={!src}
				aria-label={t('audio.volume')}
				class="w-16 sm:w-20 h-1 rounded-lg appearance-none cursor-pointer bg-[var(--bg-hover)] accent-[var(--primary)] focus:outline-hidden disabled:opacity-40"
			/>
		</div>
	</div>
</Card>
