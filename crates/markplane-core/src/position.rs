/// Fractional indexing utilities for persistent manual ordering.
///
/// Generates position keys compatible with the `fractional-indexing` npm package
/// (Rocicorp). Keys are lexicographically sortable base-62 strings.
///
/// Format: integer part (prefix char + digits) + optional fractional part.
/// - `a` prefix = 1 digit (values 0–61): a0, a1, ..., az
/// - `b` prefix = 2 digits (values 62–3905): b00, b01, ..., bzz
const DIGITS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
const BASE: usize = 62;

/// Generate N sequential position keys for sync normalization.
///
/// Produces clean, short keys: "a0", "a1", ..., "az", "b00", "b01", ...
/// These are valid keys in the Rocicorp fractional-indexing format.
pub fn sequential_keys(n: usize) -> Vec<String> {
    let mut keys = Vec::with_capacity(n);
    for i in 0..n {
        keys.push(index_to_key(i));
    }
    keys
}

pub fn index_to_key(i: usize) -> String {
    if i < 62 {
        format!("a{}", DIGITS[i] as char)
    } else {
        let i2 = i - 62;
        let d1 = i2 / 62;
        let d0 = i2 % 62;
        debug_assert!(d1 < 62, "position index too large for 2-digit encoding");
        format!("b{}{}", DIGITS[d1] as char, DIGITS[d0] as char)
    }
}

// ── Key-between generation ───────────────────────────────────────────────

/// Map a base-62 digit character to its index (0–61).
fn digit_value(c: u8) -> usize {
    match c {
        b'0'..=b'9' => (c - b'0') as usize,
        b'A'..=b'Z' => (c - b'A' + 10) as usize,
        b'a'..=b'z' => (c - b'a' + 36) as usize,
        _ => panic!("invalid fractional-indexing digit: {:?}", c as char),
    }
}

/// Extract the integer part of a position key (prefix char + integer digits).
fn get_integer_part(key: &str) -> &str {
    let prefix = key.as_bytes()[0];
    let n = (prefix - b'a' + 1) as usize;
    &key[..1 + n]
}

/// Decode the integer part of a key to its numeric value.
fn key_to_integer(int_part: &str) -> usize {
    let bytes = int_part.as_bytes();
    let prefix = bytes[0];
    let n = (prefix - b'a' + 1) as usize;
    let mut val: usize = 0;
    for &b in &bytes[1..1 + n] {
        val = val * BASE + digit_value(b);
    }
    if prefix == b'b' {
        val += BASE;
    }
    val
}

/// Increment the integer part of a key by 1.
fn increment_integer(int_part: &str) -> String {
    index_to_key(key_to_integer(int_part) + 1)
}

/// Decrement the integer part of a key by 1. Returns `None` if already at 0.
fn decrement_integer(int_part: &str) -> Option<String> {
    let val = key_to_integer(int_part);
    if val == 0 { None } else { Some(index_to_key(val - 1)) }
}

/// Find a fractional digit string that sorts between `a` and `b`.
///
/// Both arguments are base-62 digit strings (the fractional part of a key).
/// `b` can be `None` to indicate no upper bound.
fn midpoint(a: &str, b: Option<&str>) -> String {
    // Strip common prefix with b
    if let Some(b_str) = b {
        let a_bytes = a.as_bytes();
        let b_bytes = b_str.as_bytes();
        let mut n = 0;
        while n < b_bytes.len() {
            let ca = if n < a_bytes.len() { a_bytes[n] } else { DIGITS[0] };
            if ca == b_bytes[n] {
                n += 1;
            } else {
                break;
            }
        }
        if n > 0 {
            let rest_a = if n < a.len() { &a[n..] } else { "" };
            return format!("{}{}", &b_str[..n], midpoint(rest_a, Some(&b_str[n..])));
        }
    }

    // First digits differ (or a is empty)
    let idx_a = if !a.is_empty() { digit_value(a.as_bytes()[0]) } else { 0 };
    let idx_b = b.map_or(BASE, |bs| {
        if bs.is_empty() { BASE } else { digit_value(bs.as_bytes()[0]) }
    });

    if idx_b - idx_a > 1 {
        return String::from(DIGITS[(idx_a + idx_b) / 2] as char);
    }

    // Digits are adjacent or equal — take b's first char if b has more digits
    if let Some(b_str) = b
        && b_str.len() > 1 {
            return String::from(DIGITS[idx_b] as char);
    }

    // Extend: take a's digit, then recurse with no upper bound
    let rest_a = if a.len() > 1 { &a[1..] } else { "" };
    format!("{}{}", DIGITS[idx_a] as char, midpoint(rest_a, None))
}

/// Generate a position key that sorts between `a` and `b`.
///
/// Compatible with the `fractional-indexing` npm package (Rocicorp format).
///
/// - `(None, None)` → first key ("a0")
/// - `(Some(a), None)` → key after `a`
/// - `(None, Some(b))` → key before `b`
/// - `(Some(a), Some(b))` → key between `a` and `b`
///
/// Returns `None` only when asked to go before the minimum key "a0" (no room).
pub fn generate_key_between(a: Option<&str>, b: Option<&str>) -> Option<String> {
    if let (Some(a_val), Some(b_val)) = (a, b) {
        debug_assert!(a_val < b_val, "generate_key_between: {} >= {}", a_val, b_val);
    }

    match (a, b) {
        (None, None) => Some("a0".to_string()),

        (None, Some(b_key)) => {
            let int_b = get_integer_part(b_key);
            if let Some(dec) = decrement_integer(int_b) {
                Some(dec)
            } else {
                // Integer is already "a0" — need fractional sub-key
                let frac_b = &b_key[int_b.len()..];
                if !frac_b.is_empty() {
                    Some(format!("{}{}", int_b, midpoint("", Some(frac_b))))
                } else {
                    None // can't go before "a0"
                }
            }
        }

        (Some(a_key), None) => {
            let int_a = get_integer_part(a_key);
            let frac_a = &a_key[int_a.len()..];
            if frac_a.is_empty() {
                Some(increment_integer(int_a))
            } else {
                Some(format!("{}{}", int_a, midpoint(frac_a, None)))
            }
        }

        (Some(a_key), Some(b_key)) => {
            let int_a = get_integer_part(a_key);
            let frac_a = &a_key[int_a.len()..];
            let int_b = get_integer_part(b_key);
            let frac_b = &b_key[int_b.len()..];

            if int_a == int_b {
                Some(format!("{}{}", int_a, midpoint(frac_a, Some(frac_b))))
            } else {
                let inc = increment_integer(int_a);
                if inc.as_str() < b_key {
                    Some(inc)
                } else {
                    Some(format!("{}{}", int_a, midpoint(frac_a, None)))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequential_keys_basic() {
        let keys = sequential_keys(5);
        assert_eq!(keys, vec!["a0", "a1", "a2", "a3", "a4"]);
    }

    #[test]
    fn test_sequential_keys_ordering() {
        let keys = sequential_keys(100);
        for i in 1..keys.len() {
            assert!(
                keys[i - 1] < keys[i],
                "keys not ordered: {} >= {}",
                keys[i - 1],
                keys[i]
            );
        }
    }

    #[test]
    fn test_sequential_keys_boundary() {
        let keys = sequential_keys(65);
        // Last single-digit key
        assert_eq!(keys[61], "az");
        // First two-digit key
        assert_eq!(keys[62], "b00");
        assert_eq!(keys[63], "b01");
        assert_eq!(keys[64], "b02");
    }

    #[test]
    fn test_sequential_keys_empty() {
        let keys = sequential_keys(0);
        assert!(keys.is_empty());
    }

    #[test]
    fn test_fractional_key_sorts_between() {
        // Verify that a fractional key generated by the npm package
        // (e.g., "a0V" between "a0" and "a1") sorts correctly
        let a0 = "a0";
        let fractional = "a0V";
        let a1 = "a1";
        assert!(a0 < fractional);
        assert!(fractional < a1);
    }

    // ── generate_key_between tests ──────────────────────────────────────

    #[test]
    fn test_key_between_both_none() {
        assert_eq!(generate_key_between(None, None), Some("a0".to_string()));
    }

    #[test]
    fn test_key_between_after() {
        // After "a0" → "a1"
        assert_eq!(generate_key_between(Some("a0"), None), Some("a1".to_string()));
        // After "az" → "b00"
        assert_eq!(generate_key_between(Some("az"), None), Some("b00".to_string()));
        // After "a5" → "a6"
        assert_eq!(generate_key_between(Some("a5"), None), Some("a6".to_string()));
    }

    #[test]
    fn test_key_between_after_fractional() {
        // After "a5V" — has fractional part, so extend fractional
        let key = generate_key_between(Some("a5V"), None).unwrap();
        assert!(key.as_str() > "a5V", "{} should be > a5V", key);
        assert!(key.starts_with("a5"), "{} should start with a5", key);
    }

    #[test]
    fn test_key_between_before() {
        // Before "a5" → decrement integer → "a4"
        assert_eq!(generate_key_between(None, Some("a5")), Some("a4".to_string()));
        // Before "a1" → "a0"
        assert_eq!(generate_key_between(None, Some("a1")), Some("a0".to_string()));
        // Before "b00" → "az"
        assert_eq!(generate_key_between(None, Some("b00")), Some("az".to_string()));
    }

    #[test]
    fn test_key_between_before_minimum() {
        // Before "a0" with no fractional → None (no room)
        assert_eq!(generate_key_between(None, Some("a0")), None);
        // Before "a0V" → "a0" + midpoint("", "V")
        let key = generate_key_between(None, Some("a0V")).unwrap();
        assert!(key.as_str() < "a0V", "{} should be < a0V", key);
        assert!(key.starts_with("a0"), "{} should start with a0", key);
    }

    #[test]
    fn test_key_between_two_keys() {
        // Between "a3" and "a5" → "a4"
        assert_eq!(
            generate_key_between(Some("a3"), Some("a5")),
            Some("a4".to_string())
        );
    }

    #[test]
    fn test_key_between_adjacent() {
        // Between "a3" and "a4" — adjacent integers, needs fractional
        let key = generate_key_between(Some("a3"), Some("a4")).unwrap();
        assert!(key.as_str() > "a3", "{} should be > a3", key);
        assert!(key.as_str() < "a4", "{} should be < a4", key);
    }

    #[test]
    fn test_key_between_different_integer_widths() {
        // Between "az" and "b01" → "b00"
        assert_eq!(
            generate_key_between(Some("az"), Some("b01")),
            Some("b00".to_string())
        );
    }

    #[test]
    fn test_key_between_ordering_stress() {
        // Generate keys between sequential pairs and verify ordering
        let keys = sequential_keys(20);
        for i in 0..keys.len() - 1 {
            let mid = generate_key_between(Some(&keys[i]), Some(&keys[i + 1])).unwrap();
            assert!(
                keys[i].as_str() < mid.as_str(),
                "{} should be < {} (between {} and {})",
                keys[i], mid, keys[i], keys[i + 1]
            );
            assert!(
                mid.as_str() < keys[i + 1].as_str(),
                "{} should be < {} (between {} and {})",
                mid, keys[i + 1], keys[i], keys[i + 1]
            );
        }
    }

    #[test]
    fn test_key_between_repeated_bisection() {
        // Repeatedly bisect the same interval to stress the fractional logic
        let mut lo = "a3".to_string();
        let hi = "a4".to_string();
        for _ in 0..10 {
            let mid = generate_key_between(Some(&lo), Some(&hi)).unwrap();
            assert!(lo.as_str() < mid.as_str(), "{} < {}", lo, mid);
            assert!(mid.as_str() < hi.as_str(), "{} < {}", mid, hi);
            lo = mid;
        }
    }

    // ── Internal helpers ────────────────────────────────────────────────

    #[test]
    fn test_get_integer_part() {
        assert_eq!(get_integer_part("a5"), "a5");
        assert_eq!(get_integer_part("a5V"), "a5");
        assert_eq!(get_integer_part("b00"), "b00");
        assert_eq!(get_integer_part("b00V"), "b00");
    }

    #[test]
    fn test_key_integer_roundtrip() {
        for i in 0..130 {
            let key = index_to_key(i);
            let int_part = get_integer_part(&key);
            assert_eq!(key_to_integer(int_part), i, "roundtrip failed for {}", key);
        }
    }

    #[test]
    fn test_midpoint_no_upper() {
        // midpoint("", None) → middle digit
        let m = midpoint("", None);
        assert_eq!(m.len(), 1);
        let idx = digit_value(m.as_bytes()[0]);
        assert_eq!(idx, BASE / 2);
    }

    #[test]
    fn test_midpoint_with_bounds() {
        // midpoint("", Some("V")) — between start and V
        let m = midpoint("", Some("V"));
        assert!(m.as_str() < "V");
        assert!(!m.is_empty());
    }
}
