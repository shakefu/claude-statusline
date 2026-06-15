/// Format a token count into a human-readable string.
///
/// - >= 1,000,000: format as `{n/1000000:.1}m`
/// - >= 1,000 and < 1,000,000: format as `{n/1000:.1}k`
/// - < 1,000: format as the plain decimal integer
pub fn format_tokens(count: u64) -> String {
    if count >= 1_000_000 {
        format!("{:.1}m", count as f64 / 1_000_000.0)
    } else if count >= 1_000 {
        format!("{:.1}k", count as f64 / 1_000.0)
    } else {
        format!("{}", count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Values >= 1,000,000 format with one decimal place + "m"
    #[test]
    fn million_exact() {
        assert_eq!(format_tokens(1_000_000), "1.0m");
    }

    #[test]
    fn million_and_a_half() {
        assert_eq!(format_tokens(1_500_000), "1.5m");
    }

    #[test]
    fn millions_truncated() {
        assert_eq!(format_tokens(2_345_678), "2.3m");
    }

    #[test]
    fn large_millions() {
        assert_eq!(format_tokens(10_000_000), "10.0m");
    }

    // Values >= 1,000 and < 1,000,000 format with one decimal place + "k"
    #[test]
    fn thousand_exact() {
        assert_eq!(format_tokens(1_000), "1.0k");
    }

    #[test]
    fn thousand_and_a_half() {
        assert_eq!(format_tokens(1_500), "1.5k");
    }

    #[test]
    fn sixty_thousand() {
        assert_eq!(format_tokens(60_000), "60.0k");
    }

    #[test]
    fn hundred_thousand() {
        assert_eq!(format_tokens(100_000), "100.0k");
    }

    #[test]
    fn just_under_million() {
        assert_eq!(format_tokens(999_999), "1000.0k");
    }

    // Values < 1,000 format as plain integer
    #[test]
    fn zero() {
        assert_eq!(format_tokens(0), "0");
    }

    #[test]
    fn five_hundred() {
        assert_eq!(format_tokens(500), "500");
    }

    #[test]
    fn nine_ninety_nine() {
        assert_eq!(format_tokens(999), "999");
    }

    // Boundary tests
    #[test]
    fn boundary_999_to_1000() {
        assert_eq!(format_tokens(999), "999");
        assert_eq!(format_tokens(1_000), "1.0k");
    }

    #[test]
    fn boundary_999999_to_1000000() {
        assert_eq!(format_tokens(999_999), "1000.0k");
        assert_eq!(format_tokens(1_000_000), "1.0m");
    }
}
