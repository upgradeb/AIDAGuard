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
) -> Result<Vec<RuleWithCategory>, String> {
    let dir = rules_dir(&state);
    let files = read_rule_files(&dir)?;
    let mut rules = Vec::new();
    for (cat, file) in files {
        for def in file.rules {
            rules.push(RuleWithCategory { def, category: cat.clone() });
        }
    }
    Ok(rules)
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
