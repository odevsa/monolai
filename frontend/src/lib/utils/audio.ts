/**
 * Audio Utilities
 * Standardized audio processing, WAV PCM encoding, and format conversion.
 */

/**
 * Formats a duration in seconds into MM:SS format.
 */
export function formatAudioDuration(secs?: number | null): string {
	if (secs === undefined || secs === null || isNaN(secs) || secs < 0) return '00:00';
	const m = Math.floor(secs / 60);
	const s = Math.floor(secs % 60);
	return `${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
}

/**
 * Encodes Float32 PCM channel samples into a 16-bit PCM RIFF WAVE Blob.
 * Generates a valid standard RIFF header with accurate data chunk size and sample rate.
 */
export function encodePcmWav(channelData: Float32Array[], sampleRate: number): Blob {
	const numChannels = channelData.length;
	const numSamples = channelData[0].length;
	const bytesPerSample = 2; // 16-bit PCM
	const blockAlign = numChannels * bytesPerSample;
	const byteRate = sampleRate * blockAlign;
	const dataSize = numSamples * blockAlign;
	const buffer = new ArrayBuffer(44 + dataSize);
	const view = new DataView(buffer);

	// Helper to write ASCII strings
	function writeString(offset: number, str: string) {
		for (let i = 0; i < str.length; i++) {
			view.setUint8(offset + i, str.charCodeAt(i));
		}
	}

	// 1. RIFF chunk descriptor
	writeString(0, 'RIFF');
	view.setUint32(4, 36 + dataSize, true); // ChunkSize (total size - 8)
	writeString(8, 'WAVE');

	// 2. "fmt " sub-chunk
	writeString(12, 'fmt ');
	view.setUint32(16, 16, true); // Subchunk1Size (16 for PCM)
	view.setUint16(20, 1, true); // AudioFormat (1 for PCM)
	view.setUint16(22, numChannels, true); // NumChannels
	view.setUint32(24, sampleRate, true); // SampleRate
	view.setUint32(28, byteRate, true); // ByteRate
	view.setUint16(32, blockAlign, true); // BlockAlign
	view.setUint16(34, 16, true); // BitsPerSample

	// 3. "data" sub-chunk
	writeString(36, 'data');
	view.setUint32(40, dataSize, true); // Subchunk2Size

	// 4. Interleave & write 16-bit PCM samples with clipping
	let offset = 44;
	for (let i = 0; i < numSamples; i++) {
		for (let channel = 0; channel < numChannels; channel++) {
			let sample = channelData[channel][i];
			// Clamp to [-1.0, 1.0]
			sample = Math.max(-1, Math.min(1, sample));
			// Scale to 16-bit signed integer
			const int16 = sample < 0 ? sample * 0x8000 : sample * 0x7fff;
			view.setInt16(offset, int16, true);
			offset += 2;
		}
	}

	return new Blob([buffer], { type: 'audio/wav' });
}

/**
 * Resamples an AudioBuffer to a target sample rate (default: 16000Hz, ideal for speech models)
 * and downmixes multi-channel audio to mono.
 */
export async function resampleAndDownmixToMono(
	audioBuffer: AudioBuffer,
	targetSampleRate = 16000
): Promise<{ channelData: Float32Array; sampleRate: number; duration: number }> {
	const duration = audioBuffer.duration;

	// Use OfflineAudioContext for fast, hardware-accurate native resampling
	const offlineCtx = new (
		window.OfflineAudioContext ||
		(window as unknown as { webkitOfflineAudioContext: typeof OfflineAudioContext })
			.webkitOfflineAudioContext
	)(
		1, // mono
		Math.ceil(duration * targetSampleRate),
		targetSampleRate
	);

	const source = offlineCtx.createBufferSource();
	source.buffer = audioBuffer;
	source.connect(offlineCtx.destination);
	source.start(0);

	const renderedBuffer = await offlineCtx.startRendering();
	return {
		channelData: renderedBuffer.getChannelData(0),
		sampleRate: targetSampleRate,
		duration
	};
}

/**
 * Decodes any audio Blob/File (WebM, OGG, MP3, etc.) and converts it to a standard
 * 16-bit 16kHz mono PCM WAV Blob, returning the WAV Blob and exact duration in seconds.
 */
export async function convertBlobToWav(
	blob: Blob,
	targetSampleRate = 16000
): Promise<{ blob: Blob; duration: number }> {
	const arrayBuffer = await blob.arrayBuffer();
	const audioCtx = new (
		window.AudioContext ||
		(window as unknown as { webkitAudioContext: typeof AudioContext }).webkitAudioContext
	)();

	try {
		const decodedBuffer = await audioCtx.decodeAudioData(arrayBuffer);
		const { channelData, sampleRate, duration } = await resampleAndDownmixToMono(
			decodedBuffer,
			targetSampleRate
		);
		const wavBlob = encodePcmWav([channelData], sampleRate);
		return { blob: wavBlob, duration };
	} finally {
		if (audioCtx.state !== 'closed') {
			await audioCtx.close().catch(() => {});
		}
	}
}
