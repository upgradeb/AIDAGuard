use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::info;

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
    let entries = std::fs::read_dir(dir)
        .map_err(|e| format!("无法读取规则目录: {}", e))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("读取目录条目失败: {}", e))?;
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "yaml" || ext == "yml") {
            let content = std::fs::read_to_string(&path)
                .map_err(|e| format!("无法读取文件 {}: {}", path.display(), e))?;
            let file: RuleFile = serde_yaml::from_str(&content)
                .map_err(|e| format!("YAML 解析失败 {}: {}", path.display(), e))?;
            let cat = path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();
            results.push((cat, file));
        }
    }
    Ok(results)
}

fn write_rule_file(dir: &std::path::Path, category: &str, file: &RuleFile) -> Result<(), String> {
    let path = dir.join(format!("{}.yaml", category));
    let content = serde_yaml::to_string(file)
        .map_err(|e| format!("YAML 序列化失败: {}", e))?;
    std::fs::write(&path, content)
        .map_err(|e| format!("写入文件失败 {}: {}", path.display(), e))?;
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

    // Reload rules into detector
    let mut detector = state.detector.write().await;
    detector.load_from_dir(&dir).map_err(|e| format!("重新加载规则失败: {}", e))?;
    info!("规则已保存: {} -> {}", rule_id, cat);

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
        .ok_or_else(|| format!("分类 {} 不存在", category))?;
    let (cat, mut file) = files.remove(idx);

    let before = file.rules.len();
    file.rules.retain(|r| r.id != rule_id);
    if file.rules.len() == before {
        return Err(format!("规则 {} 在分类 {} 中不存在", rule_id, category));
    }

    write_rule_file(&dir, &cat, &file)?;

    let mut detector = state.detector.write().await;
    detector.load_from_dir(&dir).map_err(|e| format!("重新加载规则失败: {}", e))?;
    info!("规则已删除: {}", rule_id);

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
        return Err(format!("规则 {} 不存在", rule_id));
    }

    let mut detector = state.detector.write().await;
    detector.load_from_dir(&dir).map_err(|e| format!("重新加载规则失败: {}", e))?;

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
        return Err("正则模式过长 (上限 2000 字符)".into());
    }

    if test_text.len() > 100_000 {
        return Err("测试文本过长 (上限 100,000 字符)".into());
    }

    // 编译正则（带 size_limit 防 ReDoS）
    let regex = RegexBuilder::new(&pattern)
        .size_limit(1 << 20)
        .build()
        .map_err(|e| format!("正则编译失败: {}", e))?;

    // 查找匹配
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
        });
    }

    let match_infos: Vec<MatchInfo> = raw_matches.iter().map(MatchInfo::from).collect();

    // 替换
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
    let mut detector = state.detector.write().await;
    let count = detector.load_from_dir(&dir)
        .map_err(|e| format!("重新加载规则失败: {}", e))?;
    info!("规则已重新加载: {} 条", count);
    Ok(format!("已加载 {} 条规则", count))
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
        return Err("分类名长度必须在 1-64 个字符之间".into());
    }
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err("分类名只能包含字母、数字、下划线和连字符".into());
    }

    let dir = rules_dir(&state);
    let path = dir.join(format!("{}.yaml", name));
    if path.exists() {
        return Err(format!("分类 {} 已存在", name));
    }

    let file = RuleFile {
        version: "1".into(),
        name: name.clone(),
        description: String::new(),
        rules: Vec::new(),
    };
    write_rule_file(&dir, &name, &file)?;
    info!("分类已创建: {}", name);
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
        return Err(format!("分类 {} 不存在", name));
    }
    std::fs::remove_file(&path)
        .map_err(|e| format!("删除分类文件失败: {}", e))?;

    let mut detector = state.detector.write().await;
    let _ = detector.load_from_dir(&dir);
    info!("分类已删除: {}", name);
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
        return Err("测试样例不能为空".into());
    }
    if sample_text.len() > 50_000 {
        return Err("测试样例过长（上限 50,000 字符）".into());
    }

    // 获取默认上游配置
    let config = state.config.read().await;
    let upstream = config.upstreams.iter()
        .find(|u| u.default)
        .or_else(|| config.upstreams.first())
        .ok_or("未配置上游 LLM，请先在「大模型接入」中添加")?;

    let api_key = upstream.api_key.as_deref()
        .or_else(|| if config.api_key.is_empty() { None } else { Some(config.api_key.as_str()) })
        .ok_or("未设置 API Key")?;

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

    // 构造 prompt
    let system_prompt = r#"你是一个正则表达式专家。请根据用户提供的测试样例，生成一条敏感数据检测规则。

要求：
1. 仔细分析样例中哪部分是需要检测的敏感数据
2. 编写精确的正则表达式，避免误匹配正常文本
3. 选择合适的策略：placeholder（占位符替换）或 mask（部分掩码）
4. 选择合适的模式：detect（仅检测记录）或 filter（检测并替换）

请严格按照以下 JSON 格式返回，不要包含其他说明文字：
{
  "name": "规则中文名称",
  "pattern": "正则表达式",
  "strategy": "placeholder 或 mask",
  "mode": "detect 或 filter",
  "priority": 100
}"#;

    let user_prompt = format!("请为以下测试样例生成检测规则：\n\n{}", sample_text);

    let model = upstream.models.first().map(|s| s.as_str()).unwrap_or("gpt-4");

    // 调用 LLM API
    let client = reqwest::Client::builder()
        .use_rustls_tls()
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

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
            .map_err(|e| format!("调用 LLM 失败: {}", e))?
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
            .map_err(|e| format!("调用 LLM 失败: {}", e))?
    };

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("LLM 返回错误 ({}): {}", status.as_u16(), body));
    }

    let body: serde_json::Value = resp.json().await
        .map_err(|e| format!("解析 LLM 响应失败: {}", e))?;

    // 提取 assistant 回复
    let content = if is_anthropic {
        body
            .get("content")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|c| c.get("text"))
            .and_then(|t| t.as_str())
            .ok_or("LLM 响应格式异常")?
    } else {
        body
            .get("choices")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .ok_or("LLM 响应格式异常")?
    };

    // 解析 JSON（尝试提取 ```json ... ``` 包裹的内容）
    let json_str = if let Some(start) = content.find("```json") {
        let start = start + 7;
        let end = content[start..].find("```").map(|e| start + e).unwrap_or(content.len());
        &content[start..end]
    } else if let Some(start) = content.find('{') {
        &content[start..]
    } else {
        return Err(format!("无法从 LLM 回复中提取 JSON: {}", content));
    };

    let rule: GeneratedRule = serde_json::from_str(json_str.trim())
        .map_err(|e| format!("解析生成规则失败: {} — 原始回复: {}", e, content))?;

    // 校验
    if rule.name.is_empty() || rule.pattern.is_empty() {
        return Err("LLM 生成的规则缺少必要字段".into());
    }

    // 编译正则验证有效性
    let _ = regex::RegexBuilder::new(&rule.pattern)
        .size_limit(1 << 20)
        .build()
        .map_err(|e| format!("LLM 生成的正则无效: {} — pattern: {}", e, rule.pattern))?;

    info!("LLM 生成规则: {} — pattern: {}", rule.name, rule.pattern);
    Ok(rule)
}

#[tauri::command]
pub async fn rename_category(
    state: tauri::State<'_, AppState>,
    old_name: String,
    new_name: String,
) -> Result<(), String> {
    if new_name.is_empty() || new_name.len() > 64 {
        return Err("分类名长度必须在 1-64 个字符之间".into());
    }
    if !new_name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err("分类名只能包含字母、数字、下划线和连字符".into());
    }

    let dir = rules_dir(&state);
    let old_path = dir.join(format!("{}.yaml", old_name));
    let new_path = dir.join(format!("{}.yaml", new_name));

    if !old_path.exists() {
        return Err(format!("分类 {} 不存在", old_name));
    }
    if new_path.exists() {
        return Err(format!("分类 {} 已存在", new_name));
    }

    std::fs::rename(&old_path, &new_path)
        .map_err(|e| format!("重命名分类失败: {}", e))?;

    let mut detector = state.detector.write().await;
    let _ = detector.load_from_dir(&dir);
    info!("分类已重命名: {} -> {}", old_name, new_name);
    Ok(())
}
