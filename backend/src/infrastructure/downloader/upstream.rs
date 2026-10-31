use crate::core::error::{AppError, AppResult};
use crate::domain::runtimes::{UpstreamConfig, UpstreamRuntimeEntry};
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const ROOT_METADATA_FILENAME: &str = ".metadata.json";

/// Load the root upstream metadata cache from `{runtimes_dir}/.metadata.json`.
pub fn load_root_upstream_metadata(runtimes_dir: &Path) -> HashMap<String, UpstreamRuntimeEntry> {
    let path = runtimes_dir.join(ROOT_METADATA_FILENAME);
    if let Ok(content) = fs::read_to_string(&path) {
        if let Ok(metadata) = serde_json::from_str::<HashMap<String, UpstreamRuntimeEntry>>(&content) {
            return metadata;
        }
    }
    HashMap::new()
}

/// Atomically save the root upstream metadata cache to `{runtimes_dir}/.metadata.json`.
pub fn save_root_upstream_metadata(
    runtimes_dir: &Path,
    metadata: &HashMap<String, UpstreamRuntimeEntry>,
) -> io::Result<()> {
    if !runtimes_dir.exists() {
        fs::create_dir_all(runtimes_dir)?;
    }

    let tmp_path = runtimes_dir.join(format!("{}.tmp", ROOT_METADATA_FILENAME));
    let final_path = runtimes_dir.join(ROOT_METADATA_FILENAME);

    let json_bytes = serde_json::to_vec_pretty(metadata)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    fs::write(&tmp_path, json_bytes)?;
    fs::rename(&tmp_path, &final_path)?;

    Ok(())
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Fetch the latest upstream release data for a given runtime upstream configuration.
///
/// Returns:
/// - `Ok(Some(entry))` if a fresh release was resolved and updated.
/// - `Ok(None)` if the remote server returned 304 Not Modified (cached entry is still fresh).
/// - `Err(e)` on network or parsing failure.
pub async fn fetch_upstream(
    client: &reqwest::Client,
    config: &UpstreamConfig,
    cached: Option<&UpstreamRuntimeEntry>,
) -> AppResult<Option<UpstreamRuntimeEntry>> {
    match config {
        UpstreamConfig::GitHubRelease {
            repository,
            include_prereleases,
            tag_regex,
        } => {
            fetch_github_release(client, repository, *include_prereleases, tag_regex.as_deref(), cached).await
        }
        UpstreamConfig::GitLabRelease {
            host,
            project_id,
            tag_regex,
        } => {
            fetch_gitlab_release(client, host, project_id, tag_regex.as_deref(), cached).await
        }
        UpstreamConfig::HttpJson { url, variables } => {
            fetch_http_json(client, url, variables, cached).await
        }
        UpstreamConfig::PlainText { url, regex } => {
            fetch_plain_text(client, url, regex, cached).await
        }
    }
}

async fn fetch_github_release(
    client: &reqwest::Client,
    repository: &str,
    include_prereleases: bool,
    tag_regex_str: Option<&str>,
    cached: Option<&UpstreamRuntimeEntry>,
) -> AppResult<Option<UpstreamRuntimeEntry>> {
    let tag_regex = if let Some(pattern) = tag_regex_str {
        Some(Regex::new(pattern).map_err(|e| {
            AppError::internal(format!("Invalid tag_regex for GitHub upstream: {}", e))
        })?)
    } else {
        None
    };

    // When tag_regex or prereleases are needed, we query the releases list to find the matching one.
    // Otherwise, we query /releases/latest.
    let is_list_endpoint = include_prereleases || tag_regex.is_some();
    let url = if is_list_endpoint {
        format!("https://api.github.com/repos/{}/releases?per_page=15", repository)
    } else {
        format!("https://api.github.com/repos/{}/releases/latest", repository)
    };

    let mut req = client
        .get(&url)
        .header("Accept", "application/vnd.github.v3+json");

    if let Some(cached_entry) = cached {
        if let Some(etag) = &cached_entry.etag {
            req = req.header("If-None-Match", etag);
        }
    }

    let resp = req.send().await.map_err(|e| {
        AppError::internal(format!("Failed to connect to GitHub for {}: {}", repository, e))
    })?;

    if resp.status() == reqwest::StatusCode::NOT_MODIFIED {
        tracing::debug!(
            "GitHub release for {} returned 304 Not Modified; using cached metadata",
            repository
        );
        return Ok(None);
    }

    if !resp.status().is_success() {
        return Err(AppError::internal(format!(
            "GitHub API error for {} (status {}): {}",
            repository,
            resp.status(),
            resp.text().await.unwrap_or_default()
        )));
    }

    let etag = resp
        .headers()
        .get(reqwest::header::ETAG)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let body: Value = resp.json().await.map_err(|e| {
        AppError::internal(format!("Failed to parse GitHub JSON response: {}", e))
    })?;

    let (tag_name, html_url) = if is_list_endpoint {
        let releases = body.as_array().ok_or_else(|| {
            AppError::internal(format!(
                "Expected array of releases from GitHub API for {}",
                repository
            ))
        })?;

        let mut matched: Option<(&str, Option<&str>)> = None;

        for rel in releases {
            let is_draft = rel.get("draft").and_then(Value::as_bool).unwrap_or(false);
            if is_draft {
                continue;
            }

            let is_prerelease = rel.get("prerelease").and_then(Value::as_bool).unwrap_or(false);
            if is_prerelease && !include_prereleases {
                continue;
            }

            let tag = match rel.get("tag_name").and_then(Value::as_str) {
                Some(t) => t,
                None => continue,
            };

            let hurl = rel.get("html_url").and_then(Value::as_str);

            if let Some(re) = &tag_regex {
                if re.is_match(tag) {
                    matched = Some((tag, hurl));
                    break;
                }
            } else {
                matched = Some((tag, hurl));
                break;
            }
        }

        let (tag, hurl) = matched.ok_or_else(|| {
            AppError::not_found(format!(
                "No matching release found on GitHub for {} (regex: {:?})",
                repository, tag_regex_str
            ))
        })?;

        (tag.to_string(), hurl.map(|s| s.to_string()))
    } else {
        let tag = body
            .get("tag_name")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                AppError::internal(format!(
                    "Missing 'tag_name' in GitHub release response for {}",
                    repository
                ))
            })?;
        let hurl = body
            .get("html_url")
            .and_then(Value::as_str)
            .map(|s| s.to_string());

        (tag.to_string(), hurl)
    };

    let mut variables = HashMap::new();
    variables.insert("version".to_string(), tag_name.clone());

    let mut canonical_version = tag_name.clone();

    if let Some(re) = &tag_regex {
        if let Some(caps) = re.captures(&tag_name) {
            for name in re.capture_names().flatten() {
                if let Some(matched_val) = caps.name(name) {
                    variables.insert(name.to_string(), matched_val.as_str().to_string());
                }
            }
            if let Some(v) = variables.get("version") {
                canonical_version = v.clone();
            }
        }
    }

    Ok(Some(UpstreamRuntimeEntry {
        version: canonical_version,
        variables,
        etag,
        last_checked_at: current_timestamp(),
        release_notes_url: html_url,
    }))
}

async fn fetch_gitlab_release(
    client: &reqwest::Client,
    host: &str,
    project_id: &str,
    tag_regex_str: Option<&str>,
    cached: Option<&UpstreamRuntimeEntry>,
) -> AppResult<Option<UpstreamRuntimeEntry>> {
    let clean_host = host.trim_end_matches('/');
    let url = format!("{}/api/v4/projects/{}/releases", clean_host, project_id);

    let mut req = client.get(&url);
    if let Some(cached_entry) = cached {
        if let Some(etag) = &cached_entry.etag {
            req = req.header("If-None-Match", etag);
        }
    }

    let resp = req.send().await.map_err(|e| {
        AppError::internal(format!("Failed to connect to GitLab for {}: {}", project_id, e))
    })?;

    if resp.status() == reqwest::StatusCode::NOT_MODIFIED {
        return Ok(None);
    }

    if !resp.status().is_success() {
        return Err(AppError::internal(format!(
            "GitLab API error for {} (status {}): {}",
            project_id,
            resp.status(),
            resp.text().await.unwrap_or_default()
        )));
    }

    let etag = resp
        .headers()
        .get(reqwest::header::ETAG)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let body: Value = resp.json().await.map_err(|e| {
        AppError::internal(format!("Failed to parse GitLab JSON response: {}", e))
    })?;

    let releases = body.as_array().ok_or_else(|| {
        AppError::internal(format!("Expected array of releases from GitLab API for {}", project_id))
    })?;

    let tag_regex = if let Some(pattern) = tag_regex_str {
        Some(Regex::new(pattern).map_err(|e| {
            AppError::internal(format!("Invalid tag_regex for GitLab upstream: {}", e))
        })?)
    } else {
        None
    };

    let mut matched: Option<(&str, Option<&str>)> = None;

    for rel in releases {
        let tag = match rel.get("tag_name").and_then(Value::as_str) {
            Some(t) => t,
            None => continue,
        };

        let link = rel
            .get("_links")
            .and_then(|l| l.get("self"))
            .and_then(Value::as_str);

        if let Some(re) = &tag_regex {
            if re.is_match(tag) {
                matched = Some((tag, link));
                break;
            }
        } else {
            matched = Some((tag, link));
            break;
        }
    }

    let (tag_name, release_notes_url) = matched.ok_or_else(|| {
        AppError::not_found(format!(
            "No matching release found on GitLab for {}",
            project_id
        ))
    })?;

    let mut variables = HashMap::new();
    variables.insert("version".to_string(), tag_name.to_string());
    let mut canonical_version = tag_name.to_string();

    if let Some(re) = &tag_regex {
        if let Some(caps) = re.captures(tag_name) {
            for name in re.capture_names().flatten() {
                if let Some(matched_val) = caps.name(name) {
                    variables.insert(name.to_string(), matched_val.as_str().to_string());
                }
            }
            if let Some(v) = variables.get("version") {
                canonical_version = v.clone();
            }
        }
    }

    Ok(Some(UpstreamRuntimeEntry {
        version: canonical_version,
        variables,
        etag,
        last_checked_at: current_timestamp(),
        release_notes_url: release_notes_url.map(|s| s.to_string()),
    }))
}

async fn fetch_http_json(
    client: &reqwest::Client,
    url: &str,
    variables_map: &HashMap<String, String>,
    cached: Option<&UpstreamRuntimeEntry>,
) -> AppResult<Option<UpstreamRuntimeEntry>> {
    let mut req = client.get(url);
    if let Some(cached_entry) = cached {
        if let Some(etag) = &cached_entry.etag {
            req = req.header("If-None-Match", etag);
        }
    }

    let resp = req.send().await.map_err(|e| {
        AppError::internal(format!("Failed to connect to JSON upstream {}: {}", url, e))
    })?;

    if resp.status() == reqwest::StatusCode::NOT_MODIFIED {
        return Ok(None);
    }

    if !resp.status().is_success() {
        return Err(AppError::internal(format!(
            "HTTP JSON upstream error for {} (status {}): {}",
            url,
            resp.status(),
            resp.text().await.unwrap_or_default()
        )));
    }

    let etag = resp
        .headers()
        .get(reqwest::header::ETAG)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let body: Value = resp.json().await.map_err(|e| {
        AppError::internal(format!("Failed to parse upstream JSON response: {}", e))
    })?;

    let mut variables = HashMap::new();

    for (var_name, pointer) in variables_map {
        let clean_pointer = if pointer.starts_with('/') {
            pointer.clone()
        } else {
            format!("/{}", pointer)
        };

        if let Some(val) = body.pointer(&clean_pointer) {
            let extracted = match val {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                _ => val.to_string(),
            };
            variables.insert(var_name.clone(), extracted);
        }
    }

    let canonical_version = variables
        .get("version")
        .cloned()
        .ok_or_else(|| AppError::internal(format!("Upstream JSON from {} did not provide a 'version' variable", url)))?;

    Ok(Some(UpstreamRuntimeEntry {
        version: canonical_version,
        variables,
        etag,
        last_checked_at: current_timestamp(),
        release_notes_url: None,
    }))
}

async fn fetch_plain_text(
    client: &reqwest::Client,
    url: &str,
    regex_str: &str,
    cached: Option<&UpstreamRuntimeEntry>,
) -> AppResult<Option<UpstreamRuntimeEntry>> {
    let re = Regex::new(regex_str).map_err(|e| {
        AppError::internal(format!("Invalid regex for plain text upstream: {}", e))
    })?;

    let mut req = client.get(url);
    if let Some(cached_entry) = cached {
        if let Some(etag) = &cached_entry.etag {
            req = req.header("If-None-Match", etag);
        }
    }

    let resp = req.send().await.map_err(|e| {
        AppError::internal(format!("Failed to connect to plain text upstream {}: {}", url, e))
    })?;

    if resp.status() == reqwest::StatusCode::NOT_MODIFIED {
        return Ok(None);
    }

    if !resp.status().is_success() {
        return Err(AppError::internal(format!(
            "Plain text upstream error for {} (status {}): {}",
            url,
            resp.status(),
            resp.text().await.unwrap_or_default()
        )));
    }

    let etag = resp
        .headers()
        .get(reqwest::header::ETAG)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let text = resp.text().await.map_err(|e| {
        AppError::internal(format!("Failed to read plain text upstream response: {}", e))
    })?;

    let caps = re.captures(&text).ok_or_else(|| {
        AppError::internal(format!(
            "Regex '{}' did not match content of upstream {}",
            regex_str, url
        ))
    })?;

    let mut variables = HashMap::new();
    for name in re.capture_names().flatten() {
        if let Some(matched_val) = caps.name(name) {
            variables.insert(name.to_string(), matched_val.as_str().to_string());
        }
    }

    let canonical_version = variables
        .get("version")
        .cloned()
        .or_else(|| caps.get(0).map(|m| m.as_str().to_string()))
        .unwrap_or_default();

    if canonical_version.is_empty() {
        return Err(AppError::internal(format!(
            "No version captured from plain text upstream {}",
            url
        )));
    }

    variables.entry("version".to_string()).or_insert_with(|| canonical_version.clone());

    Ok(Some(UpstreamRuntimeEntry {
        version: canonical_version,
        variables,
        etag,
        last_checked_at: current_timestamp(),
        release_notes_url: None,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_root_upstream_metadata_save_and_load() {
        let temp_dir = std::env::temp_dir().join(format!("monolai_test_{}", current_timestamp()));
        let _ = fs::create_dir_all(&temp_dir);

        let mut sample = HashMap::new();
        let mut vars = HashMap::new();
        vars.insert("version".to_string(), "master-1031-c6dfa9d".to_string());
        vars.insert("commit".to_string(), "c6dfa9d".to_string());

        sample.insert(
            "sd-cpp".to_string(),
            UpstreamRuntimeEntry {
                version: "master-1031-c6dfa9d".to_string(),
                variables: vars,
                etag: Some("W/\"12345\"".to_string()),
                last_checked_at: 1000,
                release_notes_url: Some("https://github.com/leejet/stable-diffusion.cpp".to_string()),
            },
        );

        save_root_upstream_metadata(&temp_dir, &sample).expect("Save should succeed");
        let loaded = load_root_upstream_metadata(&temp_dir);

        assert_eq!(loaded.len(), 1);
        let entry = loaded.get("sd-cpp").expect("sd-cpp must exist");
        assert_eq!(entry.version, "master-1031-c6dfa9d");
        assert_eq!(entry.variables.get("commit").map(|s| s.as_str()), Some("c6dfa9d"));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_sd_cpp_regex_captures_commit() {
        let pattern = "master-\\d+-(?P<commit>[a-f0-9]+)";
        let re = Regex::new(pattern).unwrap();
        let tag = "master-1031-c6dfa9d";

        assert!(re.is_match(tag));
        let caps = re.captures(tag).unwrap();
        assert_eq!(caps.name("commit").unwrap().as_str(), "c6dfa9d");
    }

    #[test]
    fn test_llama_cpp_regex_captures_tag() {
        let pattern = "^b\\d+$";
        let re = Regex::new(pattern).unwrap();
        assert!(re.is_match("b12024"));
        assert!(!re.is_match("v0.8.0"));
    }

    #[tokio::test]
    async fn test_fetch_real_github() {
        let client = reqwest::Client::builder()
            .user_agent("monolai/0.3.0")
            .build()
            .unwrap();

        let cfg = UpstreamConfig::GitHubRelease {
            repository: "ggml-org/llama.cpp".to_string(),
            include_prereleases: true,
            tag_regex: Some("^b\\d+$".to_string()),
        };

        let res = fetch_upstream(&client, &cfg, None).await;
        println!("Result llama: {:?}", res);
        assert!(res.is_ok());
        let entry = res.unwrap().unwrap();
        assert!(entry.version.starts_with('b'));

        let cfg_sd = UpstreamConfig::GitHubRelease {
            repository: "leejet/stable-diffusion.cpp".to_string(),
            include_prereleases: false,
            tag_regex: Some("master-\\d+-(?P<commit>[a-f0-9]+)".to_string()),
        };

        let res_sd = fetch_upstream(&client, &cfg_sd, None).await;
        println!("Result sd: {:?}", res_sd);
        assert!(res_sd.is_ok());
        let entry_sd = res_sd.unwrap().unwrap();
        assert!(entry_sd.version.starts_with("master-"));
        assert!(entry_sd.variables.contains_key("commit"));
        println!("SD commit captured: {:?}", entry_sd.variables.get("commit"));
    }
}

