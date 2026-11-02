<script lang="ts">
	import { Alert, Button } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import { convertBlobToWav } from '$lib/utils/audio';
	import { Mic, Square, Trash2 } from '@lucide/svelte';
	import { onDestroy } from 'svelte';

	interface AudioRecorderProps {
		disabled?: boolean;
		onRecordingComplete: (blob: Blob, durationSecs: number) => void;
	}

	let { disabled = false, onRecordingComplete }: AudioRecorderProps = $props();

	let isRecording = $state(false);
	let recordingElapsed = $state(0);
	let errorMessage = $state<string | null>(null);

	let mediaRecorder: MediaRecorder | null = null;
	let audioChunks: Blob[] = [];
	let timerInterval: ReturnType<typeof setInterval> | null = null;
	let mediaStream: MediaStream | null = null;

	async function startRecording() {
		errorMessage = null;
		if (typeof navigator === 'undefined' || !navigator.mediaDevices?.getUserMedia) {
			errorMessage = t('audio.micNotSupported');
			return;
		}

		try {
			const stream = await navigator.mediaDevices.getUserMedia({
				audio: {
					echoCancellation: true,
					noiseSuppression: true,
					sampleRate: 16000
				}
			});
			mediaStream = stream;

			// Determine supported mimeType
			const mimeType =
				['audio/webm;codecs=opus', 'audio/webm', 'audio/ogg;codecs=opus', 'audio/wav'].find(
					(type) => MediaRecorder.isTypeSupported(type)
				) || '';

			mediaRecorder = mimeType
				? new MediaRecorder(stream, { mimeType })
				: new MediaRecorder(stream);
			audioChunks = [];

			mediaRecorder.ondataavailable = (e) => {
				if (e.data && e.data.size > 0) {
					audioChunks.push(e.data);
				}
			};

			mediaRecorder.onstop = async () => {
				const finalType = mediaRecorder?.mimeType || 'audio/webm';
				const rawBlob = new Blob(audioChunks, { type: finalType });
				try {
					const { blob: wavBlob, duration } = await convertBlobToWav(rawBlob);
					onRecordingComplete(wavBlob, duration || recordingElapsed);
				} catch (convErr) {
					console.warn('WAV conversion fallback:', convErr);
					onRecordingComplete(rawBlob, recordingElapsed);
				} finally {
					cleanupStream();
				}
			};

			mediaRecorder.start(250); // Slice chunks every 250ms
			isRecording = true;
			recordingElapsed = 0;

			const start = Date.now();
			timerInterval = setInterval(() => {
				recordingElapsed = Math.floor((Date.now() - start) / 1000);
			}, 500);
		} catch (err: unknown) {
			console.error('Microphone access error:', err);
			const errStr = err instanceof Error ? err.message : String(err);
			errorMessage = t('audio.micAccessDenied') + ` (${errStr})`;
			cleanupStream();
		}
	}

	function stopRecording() {
		if (mediaRecorder && isRecording) {
			if (timerInterval) {
				clearInterval(timerInterval);
				timerInterval = null;
			}
			isRecording = false;
			mediaRecorder.stop();
		}
	}

	function cancelRecording() {
		if (timerInterval) {
			clearInterval(timerInterval);
			timerInterval = null;
		}
		if (mediaRecorder && isRecording) {
			mediaRecorder.ondataavailable = null;
			mediaRecorder.onstop = null;
			mediaRecorder.stop();
		}
		isRecording = false;
		recordingElapsed = 0;
		audioChunks = [];
		cleanupStream();
	}

	function cleanupStream() {
		if (mediaStream) {
			mediaStream.getTracks().forEach((track) => track.stop());
			mediaStream = null;
		}
	}

	function formatTime(secs: number): string {
		const m = Math.floor(secs / 60);
		const s = secs % 60;
		return `${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
	}

	onDestroy(() => {
		cancelRecording();
	});
</script>

<div class="flex flex-col gap-2.5">
	{#if errorMessage}
		<Alert variant="error" message={errorMessage} />
	{/if}

	<div
		class="p-4 sm:p-5 rounded-xl flex items-center justify-between gap-4 border border-dashed {isRecording
			? 'border-[var(--danger)] bg-[var(--danger-light)]/20'
			: 'border-[var(--border-color)] bg-[var(--bg-hover)]/30'}"
	>
		<div class="flex items-center gap-3">
			<div
				class="w-9 h-9 rounded-full flex items-center justify-center shrink-0 {isRecording
					? 'bg-[var(--danger-light)] text-[var(--danger-text)] animate-pulse'
					: 'bg-[var(--bg-hover)] text-[var(--text-secondary)]'}"
			>
				<Mic size={18} class="shrink-0" />
			</div>

			<div class="flex flex-col">
				<span class="text-sm font-semibold text-[var(--text-primary)]">
					{isRecording ? t('audio.recordingInProgress') : t('audio.recordFromMic')}
				</span>
				<span class="text-xs text-[var(--text-muted)] font-mono">
					{isRecording ? formatTime(recordingElapsed) : t('audio.clickToStartRecording')}
				</span>
			</div>
		</div>

		<div class="flex items-center gap-2">
			{#if isRecording}
				<Button
					variant="ghost"
					size="icon-sm"
					onclick={cancelRecording}
					title={t('common.cancel')}
					ariaLabel={t('common.cancel')}
				>
					<Trash2 size={15} class="shrink-0 text-[var(--danger-text)]" />
				</Button>
				<Button
					variant="danger"
					size="sm"
					onclick={stopRecording}
					class="gap-1.5 px-3"
					ariaLabel={t('audio.stopRecording')}
				>
					<Square size={14} class="shrink-0 fill-current" />
					<span>{t('audio.stopRecording')}</span>
				</Button>
			{:else}
				<Button
					variant="primary"
					size="sm"
					{disabled}
					onclick={startRecording}
					class="gap-1.5 px-3"
					ariaLabel={t('audio.startRecording')}
				>
					<Mic size={15} class="shrink-0" />
					<span>{t('audio.record')}</span>
				</Button>
			{/if}
		</div>
	</div>
</div>
