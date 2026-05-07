use std::collections::HashMap;
use uuid::Uuid;

use crate::detector::{Match, Strategy};

/// 占位符 → 原始文本的映射表
#[derive(Debug, Clone, Default)]
pub struct PlaceholderMap {
    mappings: HashMap<String, String>,
}

impl PlaceholderMap {
    pub fn new() -> Self {
        Self {
            mappings: HashMap::new(),
        }
    }

    /// 记录一条映射，返回生成的占位符
    pub fn insert(&mut self, original: &str, rule_id: &str) -> String {
        let short_id = &Uuid::new_v4().to_string()[..8];
        let placeholder = format!("[[{}@{}]]", rule_id.to_uppercase(), short_id);
        self.mappings.insert(placeholder.clone(), original.to_string());
        placeholder
    }

    /// 根据占位符查询原始文本
    pub fn get(&self, placeholder: &str) -> Option<&str> {
        self.mappings.get(placeholder).map(|s| s.as_str())
    }

    /// 映射表条目数
    pub fn len(&self) -> usize {
        self.mappings.len()
    }

    /// 返回所有占位符（映射的 key）
    pub fn placeholders(&self) -> impl Iterator<Item = &String> {
        self.mappings.keys()
    }

    /// Return a reference to the full mappings table.
    pub fn mappings(&self) -> &HashMap<String, String> {
        &self.mappings
    }
}

/// 将文本中的匹配项替换为占位符或掩码。
///
/// 返回 (替换后的文本, 映射表)。
/// 匹配项按从后往前的顺序替换，避免位置偏移。
pub fn replace(text: &str, matches: &[Match]) -> (String, PlaceholderMap) {
    let mut map = PlaceholderMap::new();
    let mut result = text.to_string();

    // 从后往前替换，避免偏移
    let mut sorted: Vec<&Match> = matches.iter().collect();
    sorted.sort_by(|a, b| b.start.cmp(&a.start));

    for m in &sorted {
        let replacement = match m.strategy {
            Strategy::Placeholder => map.insert(&m.text, &m.rule_id),
            Strategy::Mask => mask_value(&m.text),
        };

        // Safety: byte indices from regex are valid UTF-8 boundaries
        let start = m.start;
        let end = m.end;
        if start <= result.len() && end <= result.len() {
            result.replace_range(start..end, &replacement);
        }
    }

    (result, map)
}

/// 将文本中的占位符还原为原始值
pub fn restore(text: &str, map: &PlaceholderMap) -> String {
    let mut result = text.to_string();
    // 按占位符长度降序替换，避免短占位符匹配到长占位符的部分前缀
    let mut entries: Vec<(&String, &String)> = map.mappings.iter().collect();
    entries.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

    for (placeholder, original) in &entries {
        result = result.replace(placeholder.as_str(), original.as_str());
    }

    result
}

/// 简单掩码：保留首尾少量字符，中间用 * 代替
pub fn mask_value(text: &str) -> String {
    let len = text.chars().count();
    if len <= 3 {
        return "*".repeat(len);
    }

    let chars: Vec<char> = text.chars().collect();

    // 取前 1/3 和后 1/3，中间为 ***
    let keep_front = (len / 3).max(1);
    let keep_back = (len / 3).max(1);

    let front: String = chars[..keep_front].iter().collect();
    let back: String = chars[len - keep_back..].iter().collect();

    format!("{}***{}", front, back)
}
