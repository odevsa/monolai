use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Modality {
    #[serde(rename = "text-to-text")]
    TextToText,
    #[serde(rename = "text-to-image")]
    TextToImage,
    #[serde(rename = "image-to-image")]
    ImageToImage,
    #[serde(rename = "text-to-speech")]
    TextToSpeech,
    #[serde(rename = "automatic-speech-recognition")]
    AutomaticSpeechRecognition,
    #[serde(rename = "embeddings")]
    Embeddings,
}

impl Modality {
    pub fn as_str(&self) -> &'static str {
        match self {
            Modality::TextToText => "text-to-text",
            Modality::TextToImage => "text-to-image",
            Modality::ImageToImage => "image-to-image",
            Modality::TextToSpeech => "text-to-speech",
            Modality::AutomaticSpeechRecognition => "automatic-speech-recognition",
            Modality::Embeddings => "embeddings",
        }
    }

    pub fn from_str_loose(s: &str) -> Option<Self> {
        let trimmed = s.trim().to_lowercase();
        match trimmed.as_str() {
            "text-to-text" | "text_to_text" | "text-generation" | "text2text-generation" | "chat" | "llm" => {
                Some(Modality::TextToText)
            }
            "text-to-image" | "text_to_image" | "image-generation" | "diffusion" => {
                Some(Modality::TextToImage)
            }
            "image-to-image" | "image_to_image" => Some(Modality::ImageToImage),
            "text-to-speech" | "text_to_speech" | "tts" => Some(Modality::TextToSpeech),
            "automatic-speech-recognition" | "speech-to-text" | "speech_to_text" | "asr" | "stt" => {
                Some(Modality::AutomaticSpeechRecognition)
            }
            "embeddings" | "embedding" | "feature-extraction" | "sentence-similarity" => {
                Some(Modality::Embeddings)
            }
            _ => None,
        }
    }
}

impl std::fmt::Display for Modality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClassificationConfidence {
    ExplicitMetadata,
    TagMatch,
    NameRegex,
    SiblingPattern,
    BinaryHeader,
    DirectoryHint,
    Fallback,
}

#[derive(Debug, Clone, Default)]
pub struct ClassificationContext<'a> {
    pub raw_pipeline_tag: Option<&'a str>,
    pub tags: &'a [String],
    pub model_id: &'a str,
    pub sibling_filenames: &'a [String],
    pub architecture: Option<&'a str>,
    pub directory_hint: Option<&'a str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClassificationResult {
    pub modality: Modality,
    pub confidence: ClassificationConfidence,
    pub matched_rule: Option<String>,
    pub detected_architecture: Option<String>,
}

static REGEX_IMAGE_NAME: OnceLock<Regex> = OnceLock::new();
static REGEX_ASR_NAME: OnceLock<Regex> = OnceLock::new();
static REGEX_TTS_NAME: OnceLock<Regex> = OnceLock::new();
static REGEX_EMBEDDINGS_NAME: OnceLock<Regex> = OnceLock::new();

fn get_image_name_regex() -> &'static Regex {
    REGEX_IMAGE_NAME.get_or_init(|| {
        Regex::new(r"(?i)(sd[-_]?turbo|sdxl|sd[1-3](\.5)?|stable[-_]?diffusion|flux(\.1)?|pony[-_]?(diffusion|realism)?|wan2(\.1)?|hunyuan[-_]?(video|image)?|photomaker|controlnet)").expect("Valid regex")
    })
}

fn get_asr_name_regex() -> &'static Regex {
    REGEX_ASR_NAME.get_or_init(|| {
        Regex::new(r"(?i)(whisper|sensevoice|paraformer|conformer|zipformer|speech[-_]?to[-_]?text|stt)").expect("Valid regex")
    })
}

fn get_tts_name_regex() -> &'static Regex {
    REGEX_TTS_NAME.get_or_init(|| {
        Regex::new(r"(?i)(kokoro|bark|xtts([-_]?v?2)?|chattts|f5[-_]?tts|e2[-_]?tts|fish[-_]?speech|cosyvoice|spark[-_]?tts|parler[-_]?tts|text[-_]?to[-_]?speech|\btts\b)").expect("Valid regex")
    })
}

fn get_embeddings_name_regex() -> &'static Regex {
    REGEX_EMBEDDINGS_NAME.get_or_init(|| {
        Regex::new(r"(?i)(bge[-_]|nomic[-_]?embed|gte[-_]|jina[-_]?embeddings?|sentence[-_]?transformers?|all[-_]?minilm)").expect("Valid regex")
    })
}

pub struct ModalityClassifier;

impl ModalityClassifier {
    pub fn classify(ctx: &ClassificationContext) -> ClassificationResult {
        // Tier 1: Explicit Pipeline Tag / Provider category
        if let Some(tag) = ctx.raw_pipeline_tag.map(str::trim).filter(|t| !t.is_empty()) {
            match tag.to_lowercase().as_str() {
                "text-generation" | "text2text-generation" => {
                    return ClassificationResult {
                        modality: Modality::TextToText,
                        confidence: ClassificationConfidence::ExplicitMetadata,
                        matched_rule: Some(format!("pipeline_tag:{}", tag)),
                        detected_architecture: None,
                    };
                }
                "text-to-image" => {
                    return ClassificationResult {
                        modality: Modality::TextToImage,
                        confidence: ClassificationConfidence::ExplicitMetadata,
                        matched_rule: Some(format!("pipeline_tag:{}", tag)),
                        detected_architecture: None,
                    };
                }
                "image-to-image" => {
                    return ClassificationResult {
                        modality: Modality::ImageToImage,
                        confidence: ClassificationConfidence::ExplicitMetadata,
                        matched_rule: Some(format!("pipeline_tag:{}", tag)),
                        detected_architecture: None,
                    };
                }
                "text-to-speech" => {
                    return ClassificationResult {
                        modality: Modality::TextToSpeech,
                        confidence: ClassificationConfidence::ExplicitMetadata,
                        matched_rule: Some(format!("pipeline_tag:{}", tag)),
                        detected_architecture: None,
                    };
                }
                "automatic-speech-recognition" => {
                    return ClassificationResult {
                        modality: Modality::AutomaticSpeechRecognition,
                        confidence: ClassificationConfidence::ExplicitMetadata,
                        matched_rule: Some(format!("pipeline_tag:{}", tag)),
                        detected_architecture: None,
                    };
                }
                "feature-extraction" | "sentence-similarity" => {
                    return ClassificationResult {
                        modality: Modality::Embeddings,
                        confidence: ClassificationConfidence::ExplicitMetadata,
                        matched_rule: Some(format!("pipeline_tag:{}", tag)),
                        detected_architecture: None,
                    };
                }
                _ => {}
            }
        }

        // Tier 2: Community Tags & Labels inspection
        for raw_tag in ctx.tags {
            let t = raw_tag.trim().to_lowercase();
            match t.as_str() {
                "text-to-image"
                | "image-to-image"
                | "diffusers"
                | "stable-diffusion"
                | "stable-diffusion-xl"
                | "sdxl"
                | "sd-turbo"
                | "flux"
                | "flux.1"
                | "sd3"
                | "sd3.5"
                | "pony" => {
                    return ClassificationResult {
                        modality: Modality::TextToImage,
                        confidence: ClassificationConfidence::TagMatch,
                        matched_rule: Some(format!("tag:{}", raw_tag)),
                        detected_architecture: None,
                    };
                }
                "text-to-speech"
                | "tts"
                | "speech-synthesis"
                | "kokoro"
                | "bark"
                | "xtts"
                | "chattts"
                | "f5-tts"
                | "cosyvoice"
                | "fish-speech" => {
                    return ClassificationResult {
                        modality: Modality::TextToSpeech,
                        confidence: ClassificationConfidence::TagMatch,
                        matched_rule: Some(format!("tag:{}", raw_tag)),
                        detected_architecture: None,
                    };
                }
                "automatic-speech-recognition"
                | "speech-to-text"
                | "asr"
                | "stt"
                | "speech-recognition"
                | "whisper"
                | "sensevoice"
                | "paraformer" => {
                    return ClassificationResult {
                        modality: Modality::AutomaticSpeechRecognition,
                        confidence: ClassificationConfidence::TagMatch,
                        matched_rule: Some(format!("tag:{}", raw_tag)),
                        detected_architecture: None,
                    };
                }
                "feature-extraction"
                | "sentence-similarity"
                | "sentence-transformers"
                | "embeddings" => {
                    return ClassificationResult {
                        modality: Modality::Embeddings,
                        confidence: ClassificationConfidence::TagMatch,
                        matched_rule: Some(format!("tag:{}", raw_tag)),
                        detected_architecture: None,
                    };
                }
                _ => {}
            }
        }

        // Tier 3: Model ID & Repository Name Regex Heuristics
        let model_id = ctx.model_id;
        if get_image_name_regex().is_match(model_id) {
            return ClassificationResult {
                modality: Modality::TextToImage,
                confidence: ClassificationConfidence::NameRegex,
                matched_rule: Some(format!("name_regex:image on '{}'", model_id)),
                detected_architecture: None,
            };
        }
        if get_asr_name_regex().is_match(model_id) {
            return ClassificationResult {
                modality: Modality::AutomaticSpeechRecognition,
                confidence: ClassificationConfidence::NameRegex,
                matched_rule: Some(format!("name_regex:asr on '{}'", model_id)),
                detected_architecture: None,
            };
        }
        if get_tts_name_regex().is_match(model_id) {
            return ClassificationResult {
                modality: Modality::TextToSpeech,
                confidence: ClassificationConfidence::NameRegex,
                matched_rule: Some(format!("name_regex:tts on '{}'", model_id)),
                detected_architecture: None,
            };
        }
        if get_embeddings_name_regex().is_match(model_id) {
            return ClassificationResult {
                modality: Modality::Embeddings,
                confidence: ClassificationConfidence::NameRegex,
                matched_rule: Some(format!("name_regex:embeddings on '{}'", model_id)),
                detected_architecture: None,
            };
        }

        // Tier 4: Sibling File Patterns (e.g. vae, clip, unet, sd_turbo, voices.bin)
        for fname in ctx.sibling_filenames {
            let lower = fname.to_lowercase();
            if lower.contains("sd_turbo")
                || lower.contains("sdxl")
                || lower.contains("flux")
                || lower.contains("vae.safetensors")
                || lower.contains("clip_l.safetensors")
                || lower.contains("diffusion_pytorch_model")
            {
                return ClassificationResult {
                    modality: Modality::TextToImage,
                    confidence: ClassificationConfidence::SiblingPattern,
                    matched_rule: Some(format!("sibling_pattern:{}", fname)),
                    detected_architecture: None,
                };
            }
            if lower.contains("voices.bin") || lower.contains("kokoro") {
                return ClassificationResult {
                    modality: Modality::TextToSpeech,
                    confidence: ClassificationConfidence::SiblingPattern,
                    matched_rule: Some(format!("sibling_pattern:{}", fname)),
                    detected_architecture: None,
                };
            }
            if lower.contains("whisper") {
                return ClassificationResult {
                    modality: Modality::AutomaticSpeechRecognition,
                    confidence: ClassificationConfidence::SiblingPattern,
                    matched_rule: Some(format!("sibling_pattern:{}", fname)),
                    detected_architecture: None,
                };
            }
        }

        // Tier 5: Binary / Header Architecture Metadata
        if let Some(arch) = ctx.architecture.map(str::trim).filter(|a| !a.is_empty()) {
            let lower = arch.to_lowercase();
            let trimmed = lower.trim();
            match trimmed {
                "flux" | "sd1" | "sd2" | "sdxl" | "sd3" | "sd3.5" | "stable-diffusion"
                | "stable_diffusion" | "diffusion" | "pony" => {
                    return ClassificationResult {
                        modality: Modality::TextToImage,
                        confidence: ClassificationConfidence::BinaryHeader,
                        matched_rule: Some(format!("arch:{}", arch)),
                        detected_architecture: Some(arch.to_string()),
                    };
                }
                "whisper" | "sensevoice" | "funasr" | "paraformer" | "conformer" | "zipformer" => {
                    return ClassificationResult {
                        modality: Modality::AutomaticSpeechRecognition,
                        confidence: ClassificationConfidence::BinaryHeader,
                        matched_rule: Some(format!("arch:{}", arch)),
                        detected_architecture: Some(arch.to_string()),
                    };
                }
                "kokoro" | "kokoro_tts" | "kokoro-tts" | "bark" | "xtts" | "xtts2" | "chattts"
                | "cosyvoice" | "cosyvoice2" | "f5_tts" | "f5tts" | "e2_tts" | "fish_speech"
                | "spark_tts" | "megatts" | "metavoice" | "valle" | "vall_e" | "tortoise"
                | "parler_tts" | "vits" | "fastspeech" | "fastspeech2" | "glow_tts" | "tacotron"
                | "tacotron2" => {
                    return ClassificationResult {
                        modality: Modality::TextToSpeech,
                        confidence: ClassificationConfidence::BinaryHeader,
                        matched_rule: Some(format!("arch:{}", arch)),
                        detected_architecture: Some(arch.to_string()),
                    };
                }
                "bert" | "nomic_bert" | "bge" | "jina" => {
                    return ClassificationResult {
                        modality: Modality::Embeddings,
                        confidence: ClassificationConfidence::BinaryHeader,
                        matched_rule: Some(format!("arch:{}", arch)),
                        detected_architecture: Some(arch.to_string()),
                    };
                }
                _ => {}
            }

            if trimmed.contains("diffusion") || trimmed.contains("flux") {
                return ClassificationResult {
                    modality: Modality::TextToImage,
                    confidence: ClassificationConfidence::BinaryHeader,
                    matched_rule: Some(format!("arch:{}", arch)),
                    detected_architecture: Some(arch.to_string()),
                };
            }
            if trimmed.contains("tts") || trimmed.contains("voice") {
                return ClassificationResult {
                    modality: Modality::TextToSpeech,
                    confidence: ClassificationConfidence::BinaryHeader,
                    matched_rule: Some(format!("arch:{}", arch)),
                    detected_architecture: Some(arch.to_string()),
                };
            }
            if trimmed.contains("asr") || trimmed.contains("stt") || trimmed.contains("whisper") {
                return ClassificationResult {
                    modality: Modality::AutomaticSpeechRecognition,
                    confidence: ClassificationConfidence::BinaryHeader,
                    matched_rule: Some(format!("arch:{}", arch)),
                    detected_architecture: Some(arch.to_string()),
                };
            }
        }

        // Tier 6: Directory Path Hint
        if let Some(hint) = ctx.directory_hint {
            if let Some(parsed) = Modality::from_str_loose(hint) {
                if parsed != Modality::TextToText {
                    return ClassificationResult {
                        modality: parsed,
                        confidence: ClassificationConfidence::DirectoryHint,
                        matched_rule: Some(format!("directory_hint:{}", hint)),
                        detected_architecture: None,
                    };
                }
            }
        }

        // Fallback default: TextToText
        ClassificationResult {
            modality: Modality::TextToText,
            confidence: ClassificationConfidence::Fallback,
            matched_rule: Some("default_fallback".to_string()),
            detected_architecture: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sd_turbo_gguf_classified_as_text_to_image() {
        // Reproduces the exact user bug: Green-Sky/SD-Turbo-GGUF with no pipeline_tag!
        let ctx = ClassificationContext {
            raw_pipeline_tag: None,
            tags: &[],
            model_id: "Green-Sky/SD-Turbo-GGUF",
            sibling_filenames: &["sd_turbo-f16.gguf".to_string()],
            architecture: None,
            directory_hint: None,
        };

        let result = ModalityClassifier::classify(&ctx);
        assert_eq!(result.modality, Modality::TextToImage);
        assert_eq!(result.confidence, ClassificationConfidence::NameRegex);
    }

    #[test]
    fn test_flux_classified_as_text_to_image_via_name() {
        let ctx = ClassificationContext {
            raw_pipeline_tag: None,
            tags: &[],
            model_id: "city96/FLUX.1-dev-gguf",
            sibling_filenames: &["flux1-dev-q4_0.gguf".to_string()],
            architecture: None,
            directory_hint: None,
        };

        let result = ModalityClassifier::classify(&ctx);
        assert_eq!(result.modality, Modality::TextToImage);
        assert_eq!(result.confidence, ClassificationConfidence::NameRegex);
    }

    #[test]
    fn test_whisper_classified_as_asr() {
        let ctx = ClassificationContext {
            raw_pipeline_tag: None,
            tags: &["whisper".to_string()],
            model_id: "ggerganov/whisper.cpp",
            sibling_filenames: &["ggml-base.en.bin".to_string()],
            architecture: None,
            directory_hint: None,
        };

        let result = ModalityClassifier::classify(&ctx);
        assert_eq!(result.modality, Modality::AutomaticSpeechRecognition);
        assert_eq!(result.confidence, ClassificationConfidence::TagMatch);
    }

    #[test]
    fn test_kokoro_classified_as_tts() {
        let ctx = ClassificationContext {
            raw_pipeline_tag: None,
            tags: &[],
            model_id: "hexgrad/Kokoro-82M",
            sibling_filenames: &["kokoro-v0_19.gguf".to_string()],
            architecture: None,
            directory_hint: None,
        };

        let result = ModalityClassifier::classify(&ctx);
        assert_eq!(result.modality, Modality::TextToSpeech);
        assert_eq!(result.confidence, ClassificationConfidence::NameRegex);
    }

    #[test]
    fn test_bge_embeddings() {
        let ctx = ClassificationContext {
            raw_pipeline_tag: None,
            tags: &["embeddings".to_string()],
            model_id: "BAAI/bge-large-en-v1.5",
            sibling_filenames: &[],
            architecture: None,
            directory_hint: None,
        };

        let result = ModalityClassifier::classify(&ctx);
        assert_eq!(result.modality, Modality::Embeddings);
        assert_eq!(result.confidence, ClassificationConfidence::TagMatch);
    }

    #[test]
    fn test_standard_llm_defaults_text_to_text() {
        let ctx = ClassificationContext {
            raw_pipeline_tag: Some("text-generation"),
            tags: &[],
            model_id: "Qwen/Qwen2.5-7B-Instruct-GGUF",
            sibling_filenames: &["qwen2.5-7b-instruct-q4_k_m.gguf".to_string()],
            architecture: None,
            directory_hint: None,
        };

        let result = ModalityClassifier::classify(&ctx);
        assert_eq!(result.modality, Modality::TextToText);
        assert_eq!(result.confidence, ClassificationConfidence::ExplicitMetadata);
    }
}

