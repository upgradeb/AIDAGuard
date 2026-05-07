/// Generic weighted mod-N checksum validation.
///
/// Computes: sum(digit[i] * weight[i % weight.len()]) % modulus == 0
/// Returns true if the checksum passes.
pub fn mod_n_check(digits: &str, modulus: u32, weights: &[u32]) -> bool {
    if digits.is_empty() || weights.is_empty() || modulus == 0 {
        return false;
    }

    let mut sum = 0u64;
    let weight_len = weights.len();

    for (i, ch) in digits.chars().rev().enumerate() {
        let Some(d) = ch.to_digit(10) else {
            return false;
        };
        let w = weights[i % weight_len] as u64;
        sum += d as u64 * w;
    }

    sum % modulus as u64 == 0
}

/// Compute the check digit for a partial number using weight-based mod-N.
/// Returns None if the input is empty.
pub fn compute_check_digit(partial: &str, modulus: u32, weights: &[u32]) -> Option<u32> {
    if partial.is_empty() || weights.is_empty() || modulus == 0 {
        return None;
    }

    let mut sum = 0u64;
    let weight_len = weights.len();

    for (i, ch) in partial.chars().rev().enumerate() {
        let Some(d) = ch.to_digit(10) else {
            return None;
        };
        let w = weights[(i + 1) % weight_len] as u64; // Shift weights by one position
        sum += d as u64 * w;
    }

    let remainder = (sum % modulus as u64) as u32;
    let check_digit = (modulus - remainder) % modulus;
    Some(check_digit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mod_n_basic() {
        // Weighted mod-10: weights [2, 1], number "24" → reversed: 4*2 + 2*1 = 10 % 10 == 0
        assert!(mod_n_check("24", 10, &[2, 1]));
    }

    #[test]
    fn test_mod_n_empty() {
        assert!(!mod_n_check("", 10, &[1]));
    }

    #[test]
    fn test_mod_n_invalid_digit() {
        assert!(!mod_n_check("12x4", 10, &[1, 2]));
    }

    #[test]
    fn test_compute_check_digit() {
        // For a simple case: partial "123", mod 10, weights [1] (no weight)
        let cd = compute_check_digit("123", 10, &[1]);
        assert!(cd.is_some());
    }
}
