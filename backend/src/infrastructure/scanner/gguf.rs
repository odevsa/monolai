use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct ModelFileInfo {
    pub format: String,
    pub modality: String,
    pub architecture: Option<String>,
    pub model_name: Option<String>,
}

#[derive(Debug)]
#[allow(dead_code)]
enum GgufValue {
    Uint8(u8),
    Int8(i8),
    Uint16(u16),
    Int16(i16),
    Uint32(u32),
    Int32(i32),
    Float32(f32),
    Bool(bool),
    String(String),
    Array(Vec<GgufValue>),
    Uint64(u64),
    Int64(i64),
    Float64(f64),
    Other,
}

fn read_exact<R: Read>(reader: &mut R, buf: &mut [u8]) -> Option<()> {
    reader.read_exact(buf).ok()
}

fn read_u8<R: Read>(reader: &mut R) -> Option<u8> {
    let mut b = [0u8; 1];
    read_exact(reader, &mut b)?;
    Some(b[0])
}

fn read_u32<R: Read>(reader: &mut R) -> Option<u32> {
    let mut b = [0u8; 4];
    read_exact(reader, &mut b)?;
    Some(u32::from_le_bytes(b))
}

fn read_u64<R: Read>(reader: &mut R) -> Option<u64> {
    let mut b = [0u8; 8];
    read_exact(reader, &mut b)?;
    Some(u64::from_le_bytes(b))
}

fn read_string<R: Read>(reader: &mut R, version: u32) -> Option<String> {
    let len = if version >= 2 {
        read_u64(reader)? as usize
    } else {
        read_u32(reader)? as usize
    };

    if len > 1024 * 1024 {
        // Sanity limit: metadata strings shouldn't be > 1MB
        return None;
    }

    let mut buf = vec![0u8; len];
    read_exact(reader, &mut buf)?;
    String::from_utf8(buf).ok()
}

fn read_value<R: Read + Seek>(reader: &mut R, value_type: u32, version: u32) -> Option<GgufValue> {
    match value_type {
        0 => Some(GgufValue::Uint8(read_u8(reader)?)),
        1 => {
            let mut b = [0u8; 1];
            read_exact(reader, &mut b)?;
            Some(GgufValue::Int8(b[0] as i8))
        }
        2 => {
            let mut b = [0u8; 2];
            read_exact(reader, &mut b)?;
            Some(GgufValue::Uint16(u16::from_le_bytes(b)))
        }
        3 => {
            let mut b = [0u8; 2];
            read_exact(reader, &mut b)?;
            Some(GgufValue::Int16(i16::from_le_bytes(b)))
        }
        4 => Some(GgufValue::Uint32(read_u32(reader)?)),
        5 => {
            let mut b = [0u8; 4];
            read_exact(reader, &mut b)?;
            Some(GgufValue::Int32(i32::from_le_bytes(b)))
        }
        6 => {
            let mut b = [0u8; 4];
            read_exact(reader, &mut b)?;
            Some(GgufValue::Float32(f32::from_le_bytes(b)))
        }
        7 => {
            let b = read_u8(reader)?;
            Some(GgufValue::Bool(b != 0))
        }
        8 => Some(GgufValue::String(read_string(reader, version)?)),
        9 => {
            let elem_type = read_u32(reader)?;
            let count = if version >= 2 {
                read_u64(reader)? as usize
            } else {
                read_u32(reader)? as usize
            };

            // Avoid unbounded allocation for large arrays (e.g. token lists)
            if elem_type == 8 && count > 1000 {
                // Large string array (e.g. tokenizer tokens) -> skip elements
                for _ in 0..count {
                    let s_len = if version >= 2 {
                        read_u64(reader)?
                    } else {
                        read_u32(reader)? as u64
                    };
                    reader.seek(SeekFrom::Current(s_len as i64)).ok()?;
                }
                return Some(GgufValue::Other);
            }

            let mut items = Vec::new();
            for _ in 0..count.min(100) {
                if let Some(val) = read_value(reader, elem_type, version) {
                    items.push(val);
                } else {
                    return None;
                }
            }
            // Skip remaining if count > 100
            if count > 100 {
                for _ in 100..count {
                    let _ = read_value(reader, elem_type, version);
                }
            }
            Some(GgufValue::Array(items))
        }
        10 => Some(GgufValue::Uint64(read_u64(reader)?)),
        11 => {
            let mut b = [0u8; 8];
            read_exact(reader, &mut b)?;
            Some(GgufValue::Int64(i64::from_le_bytes(b)))
        }
        12 => {
            let mut b = [0u8; 8];
            read_exact(reader, &mut b)?;
            Some(GgufValue::Float64(f64::from_le_bytes(b)))
        }
        _ => None,
    }
}

pub fn parse_gguf_metadata<P: AsRef<Path>>(path: P) -> Option<HashMap<String, String>> {
    let file = File::open(path).ok()?;
    let mut reader = BufReader::new(file);

    let mut magic = [0u8; 4];
    read_exact(&mut reader, &mut magic)?;
    if &magic != b"GGUF" {
        return None;
    }

    let version = read_u32(&mut reader)?;
    if version == 0 || version > 10 {
        return None;
    }

    let _tensor_count = if version >= 2 {
        read_u64(&mut reader)?
    } else {
        read_u32(&mut reader)? as u64
    };

    let kv_count = if version >= 2 {
        read_u64(&mut reader)?
    } else {
        read_u32(&mut reader)? as u64
    };

    let mut metadata = HashMap::new();

    for _ in 0..kv_count {
        let key = read_string(&mut reader, version)?;
        let val_type = read_u32(&mut reader)?;
        let val = read_value(&mut reader, val_type, version)?;

        match val {
            GgufValue::String(s) => {
                metadata.insert(key, s);
            }
            GgufValue::Uint32(u) => {
                metadata.insert(key, u.to_string());
            }
            GgufValue::Uint64(u) => {
                metadata.insert(key, u.to_string());
            }
            GgufValue::Int32(i) => {
                metadata.insert(key, i.to_string());
            }
            _ => {}
        }

        // Early return if we already have the primary metadata keys
        if metadata.contains_key("general.architecture") && metadata.contains_key("general.name") {
            // Keep going briefly for extra fields, but can break if needed
        }
    }

    Some(metadata)
}

pub fn parse_safetensors_metadata<P: AsRef<Path>>(path: P) -> Option<serde_json::Value> {
    let mut file = File::open(path).ok()?;
    let mut len_bytes = [0u8; 8];
    file.read_exact(&mut len_bytes).ok()?;
    let header_len = u64::from_le_bytes(len_bytes);

    // Sanity check: header shouldn't be larger than 50MB
    if header_len == 0 || header_len > 50 * 1024 * 1024 {
        return None;
    }

    let mut buf = vec![0u8; header_len as usize];
    file.read_exact(&mut buf).ok()?;
    serde_json::from_slice(&buf).ok()
}

pub fn map_architecture_to_modality(arch: &str) -> &'static str {
    let lower = arch.to_lowercase();
    match lower.as_str() {
        // Text-to-text architectures
        "llama" | "qwen" | "qwen2" | "qwen2.5" | "mistral" | "mixtral" | "phi" | "phi3"
        | "gemma" | "gemma2" | "deepseek" | "deepseek2" | "starcoder" | "starcoder2"
        | "command-r" | "bloom" | "falcon" | "mpt" | "baichuan" | "gpt2" | "gptj" | "gptneox"
        | "chatglm" | "internlm" | "nemotron" | "olmo" => "text-to-text",

        // Text-to-image / Diffusion architectures
        "flux" | "sd1" | "sd2" | "sdxl" | "sd3" | "sd3.5" | "stable-diffusion"
        | "stable_diffusion" | "diffusion" | "pony" => "text-to-image",

        // Speech-to-text
        "whisper" => "speech-to-text",

        // Text-to-speech
        "bark" | "xtts" | "kokoro" => "text-to-speech",

        // Embeddings
        "bert" | "nomic_bert" | "bge" | "jina" => "embeddings",

        _ => "text-to-text",
    }
}

pub fn inspect_model_file<P: AsRef<Path>>(path: P) -> ModelFileInfo {
    let path_ref = path.as_ref();
    let ext = path_ref
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_lowercase();

    let mut info = ModelFileInfo {
        format: ext.clone(),
        modality: "text-to-text".to_string(),
        architecture: None,
        model_name: None,
    };

    if ext == "gguf" {
        if let Some(meta) = parse_gguf_metadata(path_ref) {
            if let Some(arch) = meta.get("general.architecture") {
                info.architecture = Some(arch.clone());
                info.modality = map_architecture_to_modality(arch).to_string();
            }
            if let Some(name) = meta.get("general.name") {
                info.model_name = Some(name.clone());
            }
        }
    } else if ext == "safetensors" {
        if let Some(json) = parse_safetensors_metadata(path_ref) {
            let mut detected_modality: Option<&'static str> = None;
            if let Some(meta) = json.get("__metadata__").and_then(|m| m.as_object()) {
                for (k, v) in meta {
                    let k_lower = k.to_lowercase();
                    let v_str = v.as_str().unwrap_or("").to_lowercase();
                    if k_lower.contains("architecture") || k_lower.contains("format") {
                        if v_str.contains("flux")
                            || v_str.contains("sdxl")
                            || v_str.contains("stable-diffusion")
                        {
                            detected_modality = Some("text-to-image");
                            info.architecture = Some(v_str.clone());
                            break;
                        }
                    }
                }
            }
            // Check top-level keys in safetensors for diffusion UNet / checkpoint markers
            if detected_modality.is_none() {
                if let Some(obj) = json.as_object() {
                    let has_unet = obj.keys().any(|k| {
                        k.starts_with("model.diffusion_model")
                            || k.starts_with("first_stage_model")
                            || k.starts_with("cond_stage_model")
                            || k.starts_with("double_blocks.")
                            || k.contains("diffusion_model")
                    });
                    if has_unet {
                        detected_modality = Some("text-to-image");
                        info.architecture = Some("diffusion".to_string());
                    }
                }
            }
            if let Some(m) = detected_modality {
                info.modality = m.to_string();
            }
        }
    }

    // Single Source of Truth fallback: If modality wasn't specified in binary metadata,
    // inspect canonical directory structure component (e.g. text-to-image, text-to-text)
    if info.modality == "text-to-text" {
        for comp in path_ref.components() {
            if let Some(s) = comp.as_os_str().to_str() {
                match s {
                    "text-to-image" | "text-to-text" | "text-to-speech" | "speech-to-text"
                    | "embeddings" | "miscellaneous" => {
                        info.modality = s.to_string();
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    info
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_architecture_to_modality() {
        assert_eq!(map_architecture_to_modality("llama"), "text-to-text");
        assert_eq!(map_architecture_to_modality("qwen2"), "text-to-text");
        assert_eq!(map_architecture_to_modality("flux"), "text-to-image");
        assert_eq!(map_architecture_to_modality("whisper"), "speech-to-text");
        assert_eq!(map_architecture_to_modality("bert"), "embeddings");
    }
}
