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
fn mask_value(text: &str) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detector::Match;

    fn make_match(start: usize, end: usize, text: &str, rule_id: &str, strategy: Strategy) -> Match {
        Match {
            rule_id: rule_id.into(),
            start,
            end,
            text: text.into(),
            priority: 100,
            strategy,
        }
    }

    #[test]
    fn test_replace_placeholder_single() {
        // "我的手机" = 12 bytes, "13812345678" = 11 bytes
        let hits = vec![make_match(12, 23, "13812345678", "phone_cn", Strategy::Placeholder)];
        let (result, map) = replace("我的手机13812345678，请记录", &hits);
        assert!(!result.contains("13812345678"));
        assert!(result.starts_with("我的手机[[PHONE_CN@"));
        assert_eq!(map.mappings.len(), 1);
    }

    #[test]
    fn test_replace_placeholder_multiple() {
        // "手机号" = 9 bytes, "13812345678" = 11 bytes, "，邮箱" = 9 bytes, "test@example.com" = 16 bytes
        let hits = vec![
            make_match(9, 20, "13812345678", "phone_cn", Strategy::Placeholder),
            make_match(29, 45, "test@example.com", "email", Strategy::Placeholder),
        ];
        let (result, map) = replace("手机号13812345678，邮箱test@example.com", &hits);
        assert!(!result.contains("13812345678"));
        assert!(!result.contains("test@example.com"));
        assert_eq!(map.mappings.len(), 2);
    }

    #[test]
    fn test_replace_then_restore() {
        let original = "手机号13812345678，邮箱test@example.com";
        let hits = vec![
            make_match(9, 20, "13812345678", "phone_cn", Strategy::Placeholder),
            make_match(29, 45, "test@example.com", "email", Strategy::Placeholder),
        ];
        let (sanitized, map) = replace(original, &hits);
        let restored = restore(&sanitized, &map);
        assert_eq!(restored, original);
    }

    #[test]
    fn test_mask_phone() {
        // mask: 13812345678 → 138****678
        let result = mask_value("13812345678");
        assert!(result.contains("***"));
        assert!(result.starts_with("138"));
        assert!(!result.contains("13812345678"));
    }

    #[test]
    fn test_mask_short() {
        // 3 chars or less → all *
        let result = mask_value("ab");
        assert_eq!(result, "**");
    }

    #[test]
    fn test_no_matches() {
        let (result, map) = replace("无敏感数据", &[]);
        assert_eq!(result, "无敏感数据");
        assert!(map.mappings.is_empty());
    }

    #[test]
    fn test_restore_empty() {
        let map = PlaceholderMap::new();
        assert_eq!(restore("原始文本", &map), "原始文本");
    }
}
