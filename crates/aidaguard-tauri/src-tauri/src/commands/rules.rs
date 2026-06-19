use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::info;

use aidaguard_core::DetectionEngine;
use aidaguard_core::detector::{Match, RuleDef, RuleFile};
use aidaguard_core::replacer;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleWithCategory {
    #[serde(flatten)]
    pub def: RuleDef,
    pub category: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetRulesResponse {
    pub rules: Vec<RuleWithCategory>,
    pub files: Vec<String>,
    pub rules_dir: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestRuleResult {
    pub matches: Vec<MatchInfo>,
    pub sanitized_text: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchInfo {
    pub rule_id: String,
    pub start: usize,
    pub end: usize,
    pub text: String,
    pub priority: u32,
    pub strategy: String,
    pub mode: String,
}

impl From<&Match> for MatchInfo {
    fn from(m: &Match) -> Self {
        MatchInfo {
            rule_id: m.rule_id.clone(),
            start: m.start,
            end: m.end,
            text: m.text.clone(),
            priority: m.priority,
            strategy: format!("{:?}", m.strategy).to_lowercase(),
            mode: format!("{:?}", m.mode).to_lowercase(),
        }
    }
}

fn rules_dir(state: &AppState) -> PathBuf {
    let d = state.rules_dir.try_read().map(|d| d.clone()).unwrap_or_else(|_| "./rules".into());
    PathBuf::from(d)
}

fn read_rule_files(dir: &std::path::Path) -> Result<Vec<(String, RuleFile)>, String> {
    let mut results = Vec::new();
    read_rule_files_recursive(dir, dir, &mut results)?;
    Ok(results)
}

fn read_rule_files_recursive(dir: &std::path::Path, base_dir: &std::path::Path, results: &mut Vec<(String, RuleFile)>) -> Result<(), String> {
    let entries = std::fs::read_dir(dir)
        .map_err(|e| format!("Failed to read rules directory: {}", e))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();
        if path.is_dir() {
            read_rule_files_recursive(&path, base_dir, results)?;
        } else if path.extension().map_or(false, |ext| ext == "yaml" || ext == "yml") {
            let content = std::fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read file {}: {}", path.display(), e))?;
            let file: RuleFile = serde_yaml::from_str(&content)
                .map_err(|e| format!("YAML parse failed {}: {}", path.display(), e))?;
            let rel_path = path.strip_prefix(base_dir).unwrap_or(&path);
            let cat = rel_path.with_extension("").to_string_lossy().to_string();
            results.push((cat, file));
        }
    }
    Ok(())
}

fn write_rule_file(dir: &std::path::Path, category: &str, file: &RuleFile) -> Result<(), String> {
    let path = dir.join(format!("{}.yaml", category));
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    let content = serde_yaml::to_string(file)
        .map_err(|e| format!("YAML serialization failed: {}", e))?;
    std::fs::write(&path, content)
        .map_err(|e| format!("Failed to write file {}: {}", path.display(), e))?;
    Ok(())
}

#[tauri::command]
pub async fn get_rules(
    state: tauri::State<'_, AppState>,
) -> Result<GetRulesResponse, String> {
    let dir = rules_dir(&state);
    let files = read_rule_files(&dir)?;
    let file_names: Vec<String> = files.iter().map(|(c, _)| c.clone()).collect();
    let mut rules = Vec::new();
    for (cat, file) in files {
        for def in file.rules {
            rules.push(RuleWithCategory { def, category: cat.clone() });
        }
    }
    Ok(GetRulesResponse {
        rules,
        files: file_names,
        rules_dir: dir.to_string_lossy().to_string(),
    })
}

#[tauri::command]
pub async fn save_rule(
    state: tauri::State<'_, AppState>,
    rule: RuleDef,
    category: String,
) -> Result<(), String> {
    let dir = rules_dir(&state);
    let cat_key = category.clone();
    let mut files = read_rule_files(&dir)?;

    // Find or create the category file
    let (cat, file) = if let Some(idx) = files.iter().position(|(c, _)| c == &cat_key) {
        files.remove(idx)
    } else {
        (cat_key.clone(), RuleFile {
            version: "1".into(),
            name: cat_key.clone(),
            description: String::new(),
            rules: Vec::new(),
        })
    };

    let rule_id = rule.id.clone();
    let mut file = file;
    // Update existing or add new
    if let Some(idx) = file.rules.iter().position(|r| r.id == rule_id) {
        file.rules[idx] = rule;
    } else {
        file.rules.push(rule);
    }

    write_rule_file(&dir, &cat, &file)?;

    // Reload rules into engine
    let presets = state.config.read().await.rule_presets();
    let mut engine = state.detector.write().await;
    engine.reload_presets(&dir, &presets).map_err(|e| format!("Failed to reload rules: {}", e))?;
    info!("Rule saved: {} -> {}", rule_id, cat);

    Ok(())
}

#[tauri::command]
pub async fn delete_rule(
    state: tauri::State<'_, AppState>,
    rule_id: String,
    category: String,
) -> Result<(), String> {
    let dir = rules_dir(&state);
    let mut files = read_rule_files(&dir)?;

    let idx = files.iter().position(|(c, _)| c == &category)
        .ok_or_else(|| format!("Category {} does not exist", category))?;
    let (cat, mut file) = files.remove(idx);

    let before = file.rules.len();
    file.rules.retain(|r| r.id != rule_id);
    if file.rules.len() == before {
        return Err(format!("Rule {} does not exist in category {}", rule_id, category));
    }

    write_rule_file(&dir, &cat, &file)?;

    let presets = state.config.read().await.rule_presets();
    let mut engine = state.detector.write().await;
    engine.reload_presets(&dir, &presets).map_err(|e| format!("Failed to reload rules: {}", e))?;
    info!("Rule deleted: {}", rule_id);

    Ok(())
}

#[tauri::command]
pub async fn toggle_rule(
    state: tauri::State<'_, AppState>,
    rule_id: String,
    enabled: bool,
) -> Result<(), String> {
    let dir = rules_dir(&state);
    let mut files = read_rule_files(&dir)?;

    let mut found = false;
    for (cat, file) in &mut files {
        if let Some(rule) = file.rules.iter_mut().find(|r| r.id == rule_id) {
            rule.enabled = enabled;
            write_rule_file(&dir, cat, file)?;
            found = true;
            break;
        }
    }

    if !found {
        return Err(format!("Rule {} does not exist", rule_id));
    }

    let presets = state.config.read().await.rule_presets();
    let mut engine = state.detector.write().await;
    engine.reload_presets(&dir, &presets).map_err(|e| format!("Failed to reload rules: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn test_rule(
    _state: tauri::State<'_, AppState>,
    pattern: String,
    test_text: String,
) -> Result<TestRuleResult, String> {
    use regex::RegexBuilder;
    use aidaguard_core::detector::Strategy;

    if pattern.len() > 2000 {
        return Err("Pattern too long (max 2000 characters)".into());
    }

    if test_text.len() > 100_000 {
        return Err("Test text too long (max 100,000 characters)".into());
    }

    // Compile regex (with size_limit to prevent ReDoS)
    let regex = RegexBuilder::new(&pattern)
        .size_limit(1 << 20)
        .build()
        .map_err(|e| format!("Regex compilation failed: {}", e))?;

    // Find matches
    let mut raw_matches: Vec<Match> = Vec::new();
    for m in regex.find_iter(&test_text) {
        raw_matches.push(Match {
            rule_id: "test".into(),
            start: m.start(),
            end: m.end(),
            text: m.as_str().to_string(),
            priority: 100,
            strategy: Strategy::Placeholder,
            mode: aidaguard_core::detector::Mode::Filter,
            confidence: None,
        });
    }

    let match_infos: Vec<MatchInfo> = raw_matches.iter().map(MatchInfo::from).collect();

    // Replace
    let (sanitized_text, _) = replacer::replace(&test_text, &raw_matches);

    Ok(TestRuleResult {
        matches: match_infos,
        sanitized_text,
    })
}

#[tauri::command]
pub async fn reload_rules(
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let dir = rules_dir(&state);
    let presets = state.config.read().await.rule_presets();
    let mut engine = state.detector.write().await;
    let count = engine.reload_presets(&dir, &presets)
        .map_err(|e| format!("Failed to reload rules: {}", e))?;
    info!("Rules reloaded: {} rules", count);
    Ok(format!("Loaded {} rules", count))
}

#[tauri::command]
pub async fn get_rule_files(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let dir = rules_dir(&state);
    let files = read_rule_files(&dir)?;
    Ok(files.into_iter().map(|(c, _)| c).collect())
}

#[tauri::command]
pub async fn create_category(
    state: tauri::State<'_, AppState>,
    name: String,
) -> Result<(), String> {
    if name.is_empty() || name.len() > 64 {
        return Err("Category name must be 1-64 characters".into());
    }
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err("Category name may only contain letters, digits, underscores, and hyphens".into());
    }

    let dir = rules_dir(&state);
    let path = dir.join(format!("{}.yaml", name));
    if path.exists() {
        return Err(format!("Category {} already exists", name));
    }

    let file = RuleFile {
        version: "1".into(),
        name: name.clone(),
        description: String::new(),
        rules: Vec::new(),
    };
    write_rule_file(&dir, &name, &file)?;
    info!("Category created: {}", name);
    Ok(())
}

#[tauri::command]
pub async fn delete_category(
    state: tauri::State<'_, AppState>,
    name: String,
) -> Result<(), String> {
    let dir = rules_dir(&state);
    let path = dir.join(format!("{}.yaml", name));
    if !path.exists() {
        return Err(format!("Category {} does not exist", name));
    }
    std::fs::remove_file(&path)
        .map_err(|e| format!("Failed to delete category file: {}", e))?;

    let presets = state.config.read().await.rule_presets();
    let mut engine = state.detector.write().await;
    let _ = engine.reload_presets(&dir, &presets);
    info!("Category deleted: {}", name);
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedRule {
    pub name: String,
    pub pattern: String,
    pub strategy: String,
    pub mode: String,
    pub priority: u32,
}

#[tauri::command]
pub async fn generate_rule(
    state: tauri::State<'_, AppState>,
    sample_text: String,
) -> Result<GeneratedRule, String> {
    if sample_text.trim().is_empty() {
        return Err("Test sample cannot be empty".into());
    }
    if sample_text.len() > 50_000 {
        return Err("Test sample too long (max 50,000 characters)".into());
    }

    // Get default upstream config
    let config = state.config.read().await;
    let upstream = config.upstreams.iter()
        .find(|u| u.default)
        .or_else(|| config.upstreams.first())
        .ok_or("No upstream LLM configured. Please add one in \"LLM Upstreams\".")?;

    let api_key = upstream.api_key.as_deref()
        .or_else(|| if config.api_key.is_empty() { None } else { Some(config.api_key.as_str()) })
        .ok_or("API Key not set")?;

    let is_anthropic = upstream.protocol == aidaguard_core::config::UpstreamProtocol::Anthropic;

    let url = if is_anthropic {
        if upstream.url.ends_with('/') {
            format!("{}messages", upstream.url)
        } else {
            format!("{}/messages", upstream.url)
        }
    } else {
        if upstream.url.ends_with('/') {
            format!("{}chat/completions", upstream.url)
        } else {
            format!("{}/chat/completions", upstream.url)
        }
    };

    // Build prompt
    let system_prompt = r#"You are a regex expert. Based on the user's test sample, generate a sensitive data detection rule.

Requirements:
1. Carefully analyze which part of the sample is sensitive data that needs detection
2. Write a precise regex pattern that avoids false positives on normal text
3. Choose an appropriate strategy: placeholder (replace with placeholder) or mask (partial masking)
4. Choose an appropriate mode: detect (record only) or filter (detect and replace)

Return strictly in the following JSON format, with no additional text:
{
  "name": "Rule name",
  "pattern": "regex pattern",
  "strategy": "placeholder or mask",
  "mode": "detect or filter",
  "priority": 100
}"#;

    let user_prompt = format!("Please generate a detection rule for the following test sample:\n\n{}", sample_text);

    let model = upstream.models.first().map(|s| s.as_str()).unwrap_or("gpt-4");

    // Call LLM API
    let client = reqwest::Client::builder()
        .use_rustls_tls()
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let timeout = std::time::Duration::from_secs(upstream.timeout_secs.max(30));

    let resp = if is_anthropic {
        let request_body = serde_json::json!({
            "model": model,
            "max_tokens": 1024,
            "system": system_prompt,
            "messages": [
                {"role": "user", "content": user_prompt}
            ]
        });

        client
            .post(&url)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request_body)
            .timeout(timeout)
            .send()
            .await
            .map_err(|e| format!("LLM call failed: {}", e))?
    } else {
        let auth_value = if api_key.starts_with("Bearer ") {
            api_key.to_string()
        } else {
            format!("Bearer {}", api_key)
        };

        let request_body = serde_json::json!({
            "model": model,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_prompt}
            ],
            "temperature": 0.3,
            "max_tokens": 1024,
            "response_format": {"type": "json_object"}
        });

        client
            .post(&url)
            .header("Authorization", &auth_value)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .timeout(timeout)
            .send()
            .await
            .map_err(|e| format!("LLM call failed: {}", e))?
    };

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("LLM returned error ({}): {}", status.as_u16(), body));
    }

    let body: serde_json::Value = resp.json().await
        .map_err(|e| format!("Failed to parse LLM response: {}", e))?;

    // Extract assistant reply
    let content = if is_anthropic {
        body
            .get("content")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|c| c.get("text"))
            .and_then(|t| t.as_str())
            .ok_or("Unexpected LLM response format")?
    } else {
        body
            .get("choices")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .ok_or("Unexpected LLM response format")?
    };

    // Parse JSON (try extracting ```json ... ``` fenced content)
    let json_str = if let Some(start) = content.find("```json") {
        let start = start + 7;
        let end = content[start..].find("```").map(|e| start + e).unwrap_or(content.len());
        &content[start..end]
    } else if let Some(start) = content.find('{') {
        &content[start..]
    } else {
        return Err(format!("Failed to extract JSON from LLM reply: {}", content));
    };

    let rule: GeneratedRule = serde_json::from_str(json_str.trim())
        .map_err(|e| format!("Failed to parse generated rule: {} — raw reply: {}", e, content))?;

    // Validate
    if rule.name.is_empty() || rule.pattern.is_empty() {
        return Err("LLM-generated rule is missing required fields".into());
    }

    // Compile regex to verify validity
    let _ = regex::RegexBuilder::new(&rule.pattern)
        .size_limit(1 << 20)
        .build()
        .map_err(|e| format!("LLM-generated regex is invalid: {} — pattern: {}", e, rule.pattern))?;

    info!("LLM generated rule: {} — pattern: {}", rule.name, rule.pattern);
    Ok(rule)
}

#[tauri::command]
pub async fn rename_category(
    state: tauri::State<'_, AppState>,
    old_name: String,
    new_name: String,
) -> Result<(), String> {
    if new_name.is_empty() || new_name.len() > 64 {
        return Err("Category name must be 1-64 characters".into());
    }
    if !new_name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err("Category name may only contain letters, digits, underscores, and hyphens".into());
    }

    let dir = rules_dir(&state);
    let old_path = dir.join(format!("{}.yaml", old_name));
    let new_path = dir.join(format!("{}.yaml", new_name));

    if !old_path.exists() {
        return Err(format!("Category {} does not exist", old_name));
    }
    if new_path.exists() {
        return Err(format!("Category {} already exists", new_name));
    }

    std::fs::rename(&old_path, &new_path)
        .map_err(|e| format!("Failed to rename category: {}", e))?;

    let presets = state.config.read().await.rule_presets();
    let mut engine = state.detector.write().await;
    let _ = engine.reload_presets(&dir, &presets);
    info!("Category renamed: {} -> {}", old_name, new_name);
    Ok(())
}
