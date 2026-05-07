use aidaguard_core::EntityType;

use super::pattern_recognizer::PatternRecognizer;

pub fn new() -> PatternRecognizer {
    let pattern = regex::Regex::new(
        r"(?-u:\b)[京津沪渝冀豫云辽黑湘皖鲁新苏浙赣鄂桂甘晋蒙陕吉闽贵粤川青藏琼宁][A-Z][A-Z0-9]{4,5}[A-Z0-9挂学警港澳](?-u:\b)"
    ).expect("car_plate regex");

    PatternRecognizer::new(EntityType::CarPlate, "CarPlateRecognizer", pattern, 0.65)
        .with_context_words(vec![
            "车牌", "车牌号", "plate", "license plate", "车",
        ])
}
