/// Validate a numeric string using the Luhn (mod-10) algorithm.
/// Returns true if the number passes the Luhn check.
/// Input should be digits only (strip spaces/dashes before calling).
pub fn luhn_check(digits: &str) -> bool {
    if digits.is_empty() {
        return false;
    }

    let mut sum = 0u32;
    let mut double = false;

    for ch in digits.chars().rev() {
        let Some(d) = ch.to_digit(10) else {
            return false;
        };
        if double {
            let doubled = d * 2;
            sum += if doubled > 9 { doubled - 9 } else { doubled };
        } else {
            sum += d;
        }
        double = !double;
    }

    sum % 10 == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_visa() {
        // 4532015112830366 is a known valid Visa test number
        assert!(luhn_check("4532015112830366"));
    }

    #[test]
    fn test_valid_mastercard() {
        assert!(luhn_check("5555555555554444"));
    }

    #[test]
    fn test_valid_amex() {
        assert!(luhn_check("378282246310005"));
    }

    #[test]
    fn test_invalid_card() {
        assert!(!luhn_check("4532015112830367")); // last digit wrong
    }

    #[test]
    fn test_empty() {
        assert!(!luhn_check(""));
    }

    #[test]
    fn test_non_digit() {
        assert!(!luhn_check("4532a15112830366"));
    }

    #[test]
    fn test_single_digit() {
        assert!(!luhn_check("5"));
    }
}
