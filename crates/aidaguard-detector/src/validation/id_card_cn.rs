/// Validate an 18-digit Chinese ID card number using weighted mod-11 checksum.
/// Also accepts 15-digit old format numbers (no checksum validation for those).
pub fn validate_id_card_cn(id: &str) -> bool {
    let id = id.trim().to_uppercase();

    match id.len() {
        15 => validate_15_digit(&id),
        18 => validate_18_digit(&id),
        _ => false,
    }
}

/// 15-digit IDs have no checksum; validate format only.
fn validate_15_digit(id: &str) -> bool {
    if id.len() != 15 {
        return false;
    }
    // First 6 digits: area code (basic format check)
    id.chars().take(6).all(|c| c.is_ascii_digit())
        // Next 6: birth date YYMMDD
        && id.chars().skip(6).take(6).all(|c| c.is_ascii_digit())
        // Last 3: sequence
        && id.chars().skip(12).all(|c| c.is_ascii_digit())
        && is_valid_birth_date_15(&id[6..12])
}

/// 18-digit IDs: weighted mod-11 with check digit 'X' representing 10.
fn validate_18_digit(id: &str) -> bool {
    if id.len() != 18 {
        return false;
    }

    let (body, check_char) = id.split_at(17);
    if !body.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }

    // GB 11643-1999 weights: [7, 9, 10, 5, 8, 4, 2, 1, 6, 3, 7, 9, 10, 5, 8, 4, 2]
    let weights: [u32; 17] = [7, 9, 10, 5, 8, 4, 2, 1, 6, 3, 7, 9, 10, 5, 8, 4, 2];

    let sum: u64 = body
        .chars()
        .enumerate()
        .map(|(i, c)| c.to_digit(10).unwrap() as u64 * weights[i] as u64)
        .sum();

    let remainder = (sum % 11) as u8;
    let expected = match remainder {
        0 => '1',
        1 => '0',
        2 => 'X',
        3 => '9',
        4 => '8',
        5 => '7',
        6 => '6',
        7 => '5',
        8 => '4',
        9 => '3',
        10 => '2',
        _ => unreachable!(),
    };

    // Validate birth date portion
    let birth_part = &id[6..14];
    is_valid_birth_date_18(birth_part)
        && check_char.chars().next().unwrap() == expected
}

/// Validate birth date in 15-digit format: YYMMDD
fn is_valid_birth_date_15(date: &str) -> bool {
    if date.len() != 6 {
        return false;
    }
    let year: u32 = date[..2].parse().unwrap_or(0);
    let month: u32 = date[2..4].parse().unwrap_or(0);
    let day: u32 = date[4..].parse().unwrap_or(0);
    // Assume 1900+year for old format
    let full_year = 1900 + year;
    is_valid_date(full_year, month, day)
}

/// Validate birth date in 18-digit format: YYYYMMDD
fn is_valid_birth_date_18(date: &str) -> bool {
    if date.len() != 8 {
        return false;
    }
    let year: u32 = date[..4].parse().unwrap_or(0);
    let month: u32 = date[4..6].parse().unwrap_or(0);
    let day: u32 = date[6..].parse().unwrap_or(0);
    is_valid_date(year, month, day)
}

fn is_valid_date(year: u32, month: u32, day: u32) -> bool {
    if !(1900..=2100).contains(&year) || !(1..=12).contains(&month) || day == 0 {
        return false;
    }
    let max_days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => return false,
    };
    day <= max_days
}

fn is_leap_year(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_18_digit_valid() {
        // Known valid format 18-digit ID
        assert!(validate_id_card_cn("110101199003076632"));
    }

    #[test]
    fn test_validate_18_digit_invalid_check() {
        // Last digit changed
        assert!(!validate_id_card_cn("110101199003076633"));
    }

    #[test]
    fn test_validate_15_digit_format() {
        // Old 15-digit format: area(6) + YYMMDD(6) + seq(3)
        assert!(validate_id_card_cn("110101900307663"));
    }

    #[test]
    fn test_wrong_length() {
        assert!(!validate_id_card_cn("1234567890"));
    }

    #[test]
    fn test_non_numeric_body() {
        assert!(!validate_id_card_cn("1101011990030766XX"));
    }

    #[test]
    fn test_invalid_birth_date() {
        // Month 99 doesn't exist
        assert!(!validate_id_card_cn("110101199099076632"));
    }

    #[test]
    fn test_whitespace_trimmed() {
        assert!(validate_id_card_cn(" 110101199003076632 "));
    }
}
