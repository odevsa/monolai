<script lang="ts">
	import { Button } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import { Pause, Play, RotateCcw, Volume2, VolumeX } from '@lucide/svelte';
	import { onDestroy } from 'svelte';
	import ProgressBar from '../atoms/ProgressBar.svelte';

	interface AudioPlayerProps {
		src?: string | null;
		durationSecs?: number;
		variant?: 'default' | 'preview';
		class?: string;
	}

	let {
		src = null,
		durationSecs = 0,
		variant = 'default',
		class: className = ''
	}: AudioPlayerProps = $props();

	let audioEl = $state<HTMLAudioElement | null>(null);
	let isPlaying = $state(false);
	let currentTime = $state(0);
	let totalDuration = $state(0);
	let isMuted = $state(false);
	let volume = $state(1);

	// Continuous animation loop for progress bar
	let animFrameId: number | null = null;

	// Volume popup state for hover (desktop) and touch/hold (mobile)
	let isHoveringVolume = $state(false);
	let showVolumeSlider = $state(false);
	let volumeContainerEl = $state<HTMLElement | null>(null);
	let longPressTimer: ReturnType<typeof setTimeout> | null = null;
	let isLongPress = false;

	let effectiveDuration = $derived(totalDuration > 0 ? totalDuration : durationSecs);
	let progressPercent = $derived(
		effectiveDuration > 0 ? Math.min(100, (currentTime / effectiveDuration) * 100) : 0
	);

	function updateContinuousProgress() {
		if (audioEl && isPlaying) {
			currentTime = audioEl.currentTime;
			animFrameId = requestAnimationFrame(updateContinuousProgress);
		}
	}

	function startProgressLoop() {
		if (animFrameId) cancelAnimationFrame(animFrameId);
		animFrameId = requestAnimationFrame(updateContinuousProgress);
	}

	function stopProgressLoop() {
		if (animFrameId) {
			cancelAnimationFrame(animFrameId);
			animFrameId = null;
		}
		if (audioEl) {
			currentTime = audioEl.currentTime;
		}
	}

	$effect(() => {
		if (src) {
			currentTime = 0;
			isPlaying = false;
			totalDuration = 0;
			stopProgressLoop();
		}
	});

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

	function handleSeekClick(e: MouseEvent) {
		if (!audioEl || effectiveDuration <= 0) return;
		const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
		const clickX = e.clientX - rect.left;
		const ratio = Math.max(0, Math.min(1, clickX / rect.width));
		const newTime = ratio * effectiveDuration;
		audioEl.currentTime = newTime;
		currentTime = newTime;
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

	function handleTouchStart() {
		isLongPress = false;
		longPressTimer = setTimeout(() => {
			isLongPress = true;
			showVolumeSlider = true;
		}, 300);
	}

	function handleTouchEnd(e: TouchEvent) {
		if (longPressTimer) {
			clearTimeout(longPressTimer);
			longPressTimer = null;
		}
		if (isLongPress) {
			e.preventDefault();
		}
	}

	function handleVolumeClick() {
		if (isLongPress) {
			isLongPress = false;
			return;
		}
		toggleMute();
	}

	function handleWindowClick(e: MouseEvent) {
		if (showVolumeSlider && volumeContainerEl && !volumeContainerEl.contains(e.target as Node)) {
			showVolumeSlider = false;
		}
	}

	function formatTime(secs: number): string {
		if (!secs || isNaN(secs) || secs < 0) return '00:00';
		const m = Math.floor(secs / 60);
		const s = Math.floor(secs % 60);
		return `${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
	}

	onDestroy(() => {
		stopProgressLoop();
		if (longPressTimer) {
			clearTimeout(longPressTimer);
		}
		if (audioEl) {
			audioEl.pause();
			audioEl.src = '';
		}
	});
</script>

<svelte:window onclick={handleWindowClick} />

{#if src}
	<audio
		bind:this={audioEl}
		{src}
		preload="metadata"
		onplay={() => {
			isPlaying = true;
			startProgressLoop();
		}}
		onpause={() => {
			isPlaying = false;
			stopProgressLoop();
		}}
		onended={() => {
			isPlaying = false;
			stopProgressLoop();
			currentTime = 0;
		}}
		ontimeupdate={() => {
			if (audioEl && !isPlaying) {
				currentTime = audioEl.currentTime;
			}
		}}
		onloadedmetadata={() => {
			if (audioEl && !isNaN(audioEl.duration) && audioEl.duration !== Infinity) {
				totalDuration = audioEl.duration;
			}
		}}
	></audio>
{/if}

<div
	class="flex flex-col items-center justify-center w-full {variant === 'preview'
		? 'max-w-md mx-auto gap-6'
		: 'gap-1'} {className}"
>
	{#if variant === 'preview'}
		<!-- Large Center Play/Pause Button for Preview Variant -->
		<button
			type="button"
			disabled={!src}
			onclick={togglePlay}
			class="w-20 h-20 rounded-full bg-[var(--primary)] text-[var(--bg-primary)] flex items-center justify-center hover:scale-105 active:scale-95 transition-all cursor-pointer shadow-lg focus:outline-hidden disabled:opacity-50 disabled:cursor-not-allowed"
			aria-label={isPlaying ? t('audio.pause') : t('audio.play')}
		>
			{#if isPlaying}
				<Pause size={32} class="fill-current" />
			{:else}
				<Play size={32} class="fill-current translate-x-0.5" />
			{/if}
		</button>
	{/if}

	<!-- Seek Bar & Timing Controls -->
	<div class="flex flex-col gap-1 w-full">
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
		<div
			class="relative w-full flex items-center cursor-pointer py-1"
			onclick={handleSeekClick}
			role="slider"
			tabindex="0"
			aria-valuenow={currentTime}
			aria-valuemin="0"
			aria-valuemax={effectiveDuration}
			aria-label={t('audio.playbackProgress')}
		>
			<ProgressBar
				value={progressPercent}
				class={isPlaying ? '[&>div]:transition-none pointer-events-none' : 'pointer-events-none'}
			/>
		</div>

		<div
			class="flex items-center justify-between gap-2 text-[11px] font-mono text-[var(--text-muted)]"
		>
			<span>{formatTime(currentTime)}</span>

			<!-- Controls Bar -->
			<div class="flex justify-center gap-2 relative grow-1">
				{#if variant !== 'preview'}
					<!-- Standard Play/Pause Button for Default Variant -->
					<Button
						variant="ghost"
						size="icon-sm"
						disabled={!src}
						onclick={togglePlay}
						title={isPlaying ? t('audio.pause') : t('audio.play')}
						ariaLabel={isPlaying ? t('audio.pause') : t('audio.play')}
					>
						{#if isPlaying}
							<Pause size={15} />
						{:else}
							<Play size={15} />
						{/if}
					</Button>
				{/if}

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

				<!-- Volume with Hover Popover (Desktop) & Touch-hold/toggle (Mobile) -->
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<div
					bind:this={volumeContainerEl}
					class="relative flex items-center justify-center group/vol"
					onmouseenter={() => (isHoveringVolume = true)}
					onmouseleave={() => (isHoveringVolume = false)}
					ontouchstart={handleTouchStart}
					ontouchend={handleTouchEnd}
				>
					{#if isHoveringVolume || showVolumeSlider}
						<div
							class="absolute bottom-full w-10 left-1/2 -translate-x-1/2 p-2 rounded-xl bg-[var(--bg-surface)] border border-[var(--border-color)] shadow-xl z-50 flex flex-col items-center gap-2 animate-in fade-in zoom-in-95 duration-150 backdrop-blur-md"
						>
							<span class="text-[10px] font-mono text-[var(--text-muted)] font-bold select-none">
								{Math.round((isMuted ? 0 : volume) * 100)}%
							</span>
							<input
								type="range"
								min="0"
								max="1"
								step="0.01"
								value={isMuted ? 0 : volume}
								oninput={handleVolumeChange}
								style="writing-mode: vertical-lr; direction: rtl; -webkit-appearance: slider-vertical;"
								class="w-1.5 h-20 accent-[var(--primary)] cursor-pointer"
								aria-label={t('audio.volume')}
							/>
						</div>
					{/if}

					<Button
						variant="ghost"
						size="icon-sm"
						onclick={handleVolumeClick}
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
				</div>
			</div>

			<span>{formatTime(effectiveDuration)}</span>
		</div>
	</div>
</div>
