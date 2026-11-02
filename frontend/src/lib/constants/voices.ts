import type { SelectOption } from '$lib/components/ui';

export interface VoiceMetadata {
	id: string;
	name: string;
	language: string;
	flag: string;
	gender: 'female' | 'male';
}

/**
 * Full canonical catalogue of all 54 Kokoro Multilingual voices (v0.19 / v1.0).
 */
export const KOKORO_VOICES: VoiceMetadata[] = [
	// 🇧🇷 Portuguese (pt-BR)
	{ id: 'pf_dora', name: 'Dora', language: 'pt-BR', flag: '🇧🇷', gender: 'female' },
	{ id: 'pm_alex', name: 'Alex', language: 'pt-BR', flag: '🇧🇷', gender: 'male' },
	{ id: 'pm_santa', name: 'Santa', language: 'pt-BR', flag: '🇧🇷', gender: 'male' },

	// 🇪🇸 Spanish (es)
	{ id: 'ef_dora', name: 'Dora', language: 'es', flag: '🇪🇸', gender: 'female' },
	{ id: 'em_alex', name: 'Alex', language: 'es', flag: '🇪🇸', gender: 'male' },
	{ id: 'em_santa', name: 'Santa', language: 'es', flag: '🇪🇸', gender: 'male' },

	// 🇫🇷 French (fr)
	{ id: 'ff_siwis', name: 'Siwis', language: 'fr', flag: '🇫🇷', gender: 'female' },

	// 🇮🇹 Italian (it)
	{ id: 'if_sara', name: 'Sara', language: 'it', flag: '🇮🇹', gender: 'female' },
	{ id: 'im_nicola', name: 'Nicola', language: 'it', flag: '🇮🇹', gender: 'male' },

	// 🇯🇵 Japanese (ja)
	{ id: 'jf_alpha', name: 'Alpha', language: 'ja', flag: '🇯🇵', gender: 'female' },
	{ id: 'jf_gongitsune', name: 'Gongitsune', language: 'ja', flag: '🇯🇵', gender: 'female' },
	{ id: 'jf_nezumi', name: 'Nezumi', language: 'ja', flag: '🇯🇵', gender: 'female' },
	{ id: 'jf_tebukuro', name: 'Tebukuro', language: 'ja', flag: '🇯🇵', gender: 'female' },
	{ id: 'jm_kumo', name: 'Kumo', language: 'ja', flag: '🇯🇵', gender: 'male' },

	// 🇨🇳 Chinese (zh)
	{ id: 'zf_xiaobei', name: 'Xiaobei', language: 'zh', flag: '🇨🇳', gender: 'female' },
	{ id: 'zf_xiaoni', name: 'Xiaoni', language: 'zh', flag: '🇨🇳', gender: 'female' },
	{ id: 'zf_xiaoxiao', name: 'Xiaoxiao', language: 'zh', flag: '🇨🇳', gender: 'female' },
	{ id: 'zf_xiaoyi', name: 'Xiaoyi', language: 'zh', flag: '🇨🇳', gender: 'female' },
	{ id: 'zm_yunjian', name: 'Yunjian', language: 'zh', flag: '🇨🇳', gender: 'male' },
	{ id: 'zm_yunxi', name: 'Yunxi', language: 'zh', flag: '🇨🇳', gender: 'male' },
	{ id: 'zm_yunxia', name: 'Yunxia', language: 'zh', flag: '🇨🇳', gender: 'male' },
	{ id: 'zm_yunyang', name: 'Yunyang', language: 'zh', flag: '🇨🇳', gender: 'male' },

	// 🇮🇳 Hindi (hi)
	{ id: 'hf_alpha', name: 'Alpha', language: 'hi', flag: '🇮🇳', gender: 'female' },
	{ id: 'hf_beta', name: 'Beta', language: 'hi', flag: '🇮🇳', gender: 'female' },
	{ id: 'hm_omega', name: 'Omega', language: 'hi', flag: '🇮🇳', gender: 'male' },
	{ id: 'hm_psi', name: 'Psi', language: 'hi', flag: '🇮🇳', gender: 'male' },

	// 🇺🇸 American English (en-US)
	{ id: 'af_heart', name: 'Heart', language: 'en-US', flag: '🇺🇸', gender: 'female' },
	{ id: 'af_bella', name: 'Bella', language: 'en-US', flag: '🇺🇸', gender: 'female' },
	{ id: 'af_nicole', name: 'Nicole', language: 'en-US', flag: '🇺🇸', gender: 'female' },
	{ id: 'af_sarah', name: 'Sarah', language: 'en-US', flag: '🇺🇸', gender: 'female' },
	{ id: 'af_sky', name: 'Sky', language: 'en-US', flag: '🇺🇸', gender: 'female' },
	{ id: 'af_alloy', name: 'Alloy', language: 'en-US', flag: '🇺🇸', gender: 'female' },
	{ id: 'af_aoede', name: 'Aoede', language: 'en-US', flag: '🇺🇸', gender: 'female' },
	{ id: 'af_jessica', name: 'Jessica', language: 'en-US', flag: '🇺🇸', gender: 'female' },
	{ id: 'af_kore', name: 'Kore', language: 'en-US', flag: '🇺🇸', gender: 'female' },
	{ id: 'af_nova', name: 'Nova', language: 'en-US', flag: '🇺🇸', gender: 'female' },
	{ id: 'af_river', name: 'River', language: 'en-US', flag: '🇺🇸', gender: 'female' },
	{ id: 'am_adam', name: 'Adam', language: 'en-US', flag: '🇺🇸', gender: 'male' },
	{ id: 'am_michael', name: 'Michael', language: 'en-US', flag: '🇺🇸', gender: 'male' },
	{ id: 'am_echo', name: 'Echo', language: 'en-US', flag: '🇺🇸', gender: 'male' },
	{ id: 'am_eric', name: 'Eric', language: 'en-US', flag: '🇺🇸', gender: 'male' },
	{ id: 'am_fenrir', name: 'Fenrir', language: 'en-US', flag: '🇺🇸', gender: 'male' },
	{ id: 'am_liam', name: 'Liam', language: 'en-US', flag: '🇺🇸', gender: 'male' },
	{ id: 'am_onyx', name: 'Onyx', language: 'en-US', flag: '🇺🇸', gender: 'male' },
	{ id: 'am_puck', name: 'Puck', language: 'en-US', flag: '🇺🇸', gender: 'male' },
	{ id: 'am_santa', name: 'Santa', language: 'en-US', flag: '🇺🇸', gender: 'male' },

	// 🇬🇧 British English (en-GB)
	{ id: 'bf_emma', name: 'Emma', language: 'en-GB', flag: '🇬🇧', gender: 'female' },
	{ id: 'bf_isabella', name: 'Isabella', language: 'en-GB', flag: '🇬🇧', gender: 'female' },
	{ id: 'bf_alice', name: 'Alice', language: 'en-GB', flag: '🇬🇧', gender: 'female' },
	{ id: 'bf_lily', name: 'Lily', language: 'en-GB', flag: '🇬🇧', gender: 'female' },
	{ id: 'bm_george', name: 'George', language: 'en-GB', flag: '🇬🇧', gender: 'male' },
	{ id: 'bm_lewis', name: 'Lewis', language: 'en-GB', flag: '🇬🇧', gender: 'male' },
	{ id: 'bm_daniel', name: 'Daniel', language: 'en-GB', flag: '🇬🇧', gender: 'male' },
	{ id: 'bm_fable', name: 'Fable', language: 'en-GB', flag: '🇬🇧', gender: 'male' }
];

export const OPENAI_STANDARD_VOICES: SelectOption[] = [
	{ value: 'alloy', label: 'Alloy (OpenAI Standard)' },
	{ value: 'echo', label: 'Echo (OpenAI Standard)' },
	{ value: 'fable', label: 'Fable (OpenAI Standard)' },
	{ value: 'onyx', label: 'Onyx (OpenAI Standard)' },
	{ value: 'nova', label: 'Nova (OpenAI Standard)' },
	{ value: 'shimmer', label: 'Shimmer (OpenAI Standard)' },
	{ value: 'default', label: 'Default' }
];

const KOKORO_MAP = new Map<string, VoiceMetadata>(KOKORO_VOICES.map((v) => [v.id, v]));

export function formatVoiceOption(voiceId: string): SelectOption {
	const kokoro = KOKORO_MAP.get(voiceId);
	if (kokoro) {
		const genderLabel = kokoro.gender === 'female' ? 'Female' : 'Male';
		return {
			value: kokoro.id,
			label: `${kokoro.flag} ${kokoro.name} (${genderLabel}, ${kokoro.language})`,
			sublabel: kokoro.id
		};
	}

	const openai = OPENAI_STANDARD_VOICES.find((o) => o.value === voiceId);
	if (openai) {
		return openai;
	}

	// Unknown or custom speaker ID (e.g. zero-shot or user-defined)
	return {
		value: voiceId,
		label: voiceId,
		sublabel: 'Custom Voice'
	};
}

export const KOKORO_VOICE_OPTIONS: SelectOption[] = KOKORO_VOICES.map((v) =>
	formatVoiceOption(v.id)
);

/**
 * Resolves appropriate voice options given detected backend voices and model family.
 */
export function resolveVoiceOptions(
	detectedVoices?: string[],
	family?: string,
	modelId?: string
): SelectOption[] {
	if (detectedVoices && detectedVoices.length > 0) {
		return detectedVoices.map((v) => formatVoiceOption(v));
	}

	const normalized = `${family || ''} ${modelId || ''}`.toLowerCase();
	if (normalized.includes('kokoro')) {
		return KOKORO_VOICE_OPTIONS;
	}

	return OPENAI_STANDARD_VOICES;
}

