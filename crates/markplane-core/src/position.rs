/// Fractional indexing utilities for persistent manual ordering.
///
/// Generates position keys compatible with the `fractional-indexing` npm package
/// (Rocicorp). Keys are lexicographically sortable base-62 strings.
///
/// Format: integer part (prefix char + digits) + optional fractional part.
/// - Lowercase prefixes `a`–`z`: a=2 chars, b=3, ..., z=27 (values going up)
/// - Uppercase prefixes `A`–`Z`: Z=2 chars, Y=3, ..., A=27 (values going down)
const DIGITS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
const BASE: usize = 62;

/// Smallest possible integer part: A followed by 26 zeros (27 chars total).
const SMALLEST_INTEGER: &str = "A00000000000000000000000000";

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

/// Get the total length of the integer part (head char + digits) for a given head.
///
/// Lowercase `a`–`z`: length = head - 'a' + 2  (a→2, b→3, ..., z→27)
/// Uppercase `A`–`Z`: length = 'Z' - head + 2  (Z→2, Y→3, ..., A→27)
fn get_integer_length(head: u8) -> usize {
    match head {
        b'a'..=b'z' => (head - b'a' + 2) as usize,
        b'A'..=b'Z' => (b'Z' - head + 2) as usize,
        _ => panic!("invalid order key head: {:?}", head as char),
    }
}

/// Extract the integer part of a position key (prefix char + integer digits).
fn get_integer_part(key: &str) -> &str {
    let head = key.as_bytes()[0];
    let len = get_integer_length(head);
    assert!(len <= key.len(), "invalid order key: {}", key);
    &key[..len]
}

/// Validate an order key. Panics if invalid.
fn validate_order_key(key: &str) {
    assert!(key.len() >= 2, "invalid order key: {}", key);
    assert!(key != SMALLEST_INTEGER, "invalid order key: {}", key);
    let int_part = get_integer_part(key);
    let frac = &key[int_part.len()..];
    assert!(
        !frac.ends_with(DIGITS[0] as char),
        "invalid order key (trailing zero): {}",
        key
    );
}

/// Increment the integer part by 1. Returns `None` at the ceiling (`z` + max digits).
fn increment_integer(x: &str) -> Option<String> {
    debug_assert!(
        x.len() == get_integer_length(x.as_bytes()[0]),
        "invalid integer part: {}",
        x
    );
    let bytes = x.as_bytes();
    let head = bytes[0];
    let mut digs: Vec<u8> = bytes[1..].to_vec();

    let mut carry = true;
    for i in (0..digs.len()).rev() {
        if !carry {
            break;
        }
        let d = digit_value(digs[i]) + 1;
        if d == BASE {
            digs[i] = DIGITS[0];
        } else {
            digs[i] = DIGITS[d];
            carry = false;
        }
    }

    if carry {
        if head == b'Z' {
            // Crossover: uppercase Z → lowercase a
            return Some(format!("a{}", DIGITS[0] as char));
        }
        if head == b'z' {
            // Ceiling reached
            return None;
        }
        let h = head + 1;
        if h > b'a' {
            // Lowercase: growing length — append a digit
            digs.push(DIGITS[0]);
        } else {
            // Uppercase: shrinking length — pop a digit
            digs.pop();
        }
        let digs_str: String = digs.iter().map(|&b| b as char).collect();
        Some(format!("{}{}", h as char, digs_str))
    } else {
        let digs_str: String = digs.iter().map(|&b| b as char).collect();
        Some(format!("{}{}", head as char, digs_str))
    }
}

/// Decrement the integer part by 1. Returns `None` at the floor (`A` + min digits).
fn decrement_integer(x: &str) -> Option<String> {
    debug_assert!(
        x.len() == get_integer_length(x.as_bytes()[0]),
        "invalid integer part: {}",
        x
    );
    let bytes = x.as_bytes();
    let head = bytes[0];
    let mut digs: Vec<u8> = bytes[1..].to_vec();

    let mut borrow = true;
    for i in (0..digs.len()).rev() {
        if !borrow {
            break;
        }
        let d = digit_value(digs[i]) as isize - 1;
        if d == -1 {
            digs[i] = DIGITS[BASE - 1];
        } else {
            digs[i] = DIGITS[d as usize];
            borrow = false;
        }
    }

    if borrow {
        if head == b'a' {
            // Crossover: lowercase a → uppercase Z
            return Some(format!("Z{}", DIGITS[BASE - 1] as char));
        }
        if head == b'A' {
            // Floor reached
            return None;
        }
        let h = head - 1;
        if h < b'Z' {
            // Uppercase: growing length — append a digit
            digs.push(DIGITS[BASE - 1]);
        } else {
            // Lowercase: shrinking length — pop a digit
            digs.pop();
        }
        let digs_str: String = digs.iter().map(|&b| b as char).collect();
        Some(format!("{}{}", h as char, digs_str))
    } else {
        let digs_str: String = digs.iter().map(|&b| b as char).collect();
        Some(format!("{}{}", head as char, digs_str))
    }
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
/// Returns `None` only at the absolute floor (below `A` + 26 zeros + fractional).
pub fn generate_key_between(a: Option<&str>, b: Option<&str>) -> Option<String> {
    if cfg!(debug_assertions) {
        if let Some(a_key) = a {
            validate_order_key(a_key);
        }
        if let Some(b_key) = b {
            validate_order_key(b_key);
        }
    }
    if let (Some(a_val), Some(b_val)) = (a, b) {
        debug_assert!(a_val < b_val, "generate_key_between: {} >= {}", a_val, b_val);
    }

    match (a, b) {
        (None, None) => Some("a0".to_string()),

        (None, Some(b_key)) => {
            let int_b = get_integer_part(b_key);
            let frac_b = &b_key[int_b.len()..];
            if int_b == SMALLEST_INTEGER {
                return Some(format!("{}{}", int_b, midpoint("", Some(frac_b))));
            }
            // If b has a fractional part, its integer part is a valid key before b
            if int_b.len() < b_key.len() {
                return Some(int_b.to_string());
            }
            decrement_integer(int_b)
        }

        (Some(a_key), None) => {
            let int_a = get_integer_part(a_key);
            let frac_a = &a_key[int_a.len()..];
            match increment_integer(int_a) {
                Some(inc) => Some(inc),
                None => Some(format!("{}{}", int_a, midpoint(frac_a, None))),
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
                match increment_integer(int_a) {
                    Some(inc) if inc.as_str() < b_key => Some(inc),
                    _ => Some(format!("{}{}", int_a, midpoint(frac_a, None))),
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
        // After "a5V" — npm behavior: increment integer part → "a6"
        assert_eq!(
            generate_key_between(Some("a5V"), None),
            Some("a6".to_string())
        );
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
    fn test_key_between_before_a0() {
        // Before "a0" → decrement crosses over to Z-prefix → "Zz"
        assert_eq!(
            generate_key_between(None, Some("a0")),
            Some("Zz".to_string())
        );
    }

    #[test]
    fn test_key_between_before_a0_fractional() {
        // Before "a0V" → integer part "a0" < "a0V", so return "a0"
        assert_eq!(
            generate_key_between(None, Some("a0V")),
            Some("a0".to_string())
        );
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

    // ── Uppercase prefix tests ──────────────────────────────────────────

    #[test]
    fn test_key_between_uppercase_prefix() {
        // Keys produced by the npm package when dragging above the first item
        // These should not panic and should produce valid results
        let key = generate_key_between(Some("Zy"), Some("Zz")).unwrap();
        assert!(key.as_str() > "Zy", "{} should be > Zy", key);
        assert!(key.as_str() < "Zz", "{} should be < Zz", key);
    }

    #[test]
    fn test_key_between_before_uppercase() {
        // Before a Z-prefix key
        let key = generate_key_between(None, Some("Zy")).unwrap();
        assert!(key.as_str() < "Zy", "{} should be < Zy", key);
    }

    #[test]
    fn test_key_between_after_uppercase() {
        // After a Z-prefix key — should produce the next integer
        assert_eq!(
            generate_key_between(Some("Zz"), None),
            Some("a0".to_string())
        );
    }

    #[test]
    fn test_key_between_across_boundary() {
        // Between uppercase and lowercase range
        let key = generate_key_between(Some("Zz"), Some("a1")).unwrap();
        assert!(key.as_str() > "Zz", "{} should be > Zz", key);
        assert!(key.as_str() < "a1", "{} should be < a1", key);
    }

    #[test]
    fn test_regression_uppercase_no_panic() {
        // Exact scenario that caused the crash: generate_key_between with
        // uppercase-prefix keys should not panic
        let _ = generate_key_between(None, Some("Zy"));
        let _ = generate_key_between(None, Some("Zz"));
        let _ = generate_key_between(None, Some("Zzx"));
        let _ = generate_key_between(Some("Zy"), Some("Zz"));
        let _ = generate_key_between(Some("Zy"), None);
        let _ = generate_key_between(Some("ZzV"), None);
    }

    #[test]
    fn test_uppercase_sorting() {
        // Verify Z-prefix keys sort before a-prefix keys (lexicographic)
        assert!("Zy" < "Zz");
        assert!("Zz" < "a0");
        assert!("Zzx" < "a0");
        assert!("ZzV" < "a0");
    }

    #[test]
    fn test_key_between_deep_uppercase() {
        // Repeated insertion before first item should keep producing valid keys
        let mut hi = "a0".to_string();
        for _ in 0..10 {
            let key = generate_key_between(None, Some(&hi)).unwrap();
            assert!(key.as_str() < hi.as_str(), "{} should be < {}", key, hi);
            hi = key;
        }
    }

    // ── Internal helpers ────────────────────────────────────────────────

    #[test]
    fn test_get_integer_length() {
        // Lowercase: a=2, b=3, z=27
        assert_eq!(get_integer_length(b'a'), 2);
        assert_eq!(get_integer_length(b'b'), 3);
        assert_eq!(get_integer_length(b'z'), 27);
        // Uppercase: Z=2, Y=3, A=27
        assert_eq!(get_integer_length(b'Z'), 2);
        assert_eq!(get_integer_length(b'Y'), 3);
        assert_eq!(get_integer_length(b'A'), 27);
    }

    #[test]
    fn test_get_integer_part() {
        assert_eq!(get_integer_part("a5"), "a5");
        assert_eq!(get_integer_part("a5V"), "a5");
        assert_eq!(get_integer_part("b00"), "b00");
        assert_eq!(get_integer_part("b00V"), "b00");
        // Uppercase prefixes
        assert_eq!(get_integer_part("Zz"), "Zz");
        assert_eq!(get_integer_part("ZzV"), "Zz");
        assert_eq!(get_integer_part("Zy"), "Zy");
        assert_eq!(get_integer_part("Yzz"), "Yzz");
        assert_eq!(get_integer_part("YzzV"), "Yzz");
    }

    #[test]
    fn test_increment_decrement_crossover() {
        // Crossover: Zz → increment → a0
        assert_eq!(increment_integer("Zz"), Some("a0".to_string()));
        // Crossover: a0 → decrement → Zz
        assert_eq!(decrement_integer("a0"), Some("Zz".to_string()));
    }

    #[test]
    fn test_increment_decrement_uppercase() {
        // Within uppercase range
        assert_eq!(decrement_integer("Zz"), Some("Zy".to_string()));
        assert_eq!(decrement_integer("Z0"), Some("Yzz".to_string()));
        assert_eq!(increment_integer("Yzz"), Some("Z0".to_string()));
        assert_eq!(increment_integer("Zy"), Some("Zz".to_string()));
    }

    #[test]
    fn test_increment_decrement_lowercase() {
        // Within lowercase range
        assert_eq!(increment_integer("a0"), Some("a1".to_string()));
        assert_eq!(increment_integer("az"), Some("b00".to_string()));
        assert_eq!(decrement_integer("b00"), Some("az".to_string()));
        assert_eq!(decrement_integer("a1"), Some("a0".to_string()));
    }

    #[test]
    fn test_increment_ceiling() {
        // z + max digits → None (ceiling)
        let max_z = format!("z{}", "z".repeat(26));
        assert_eq!(increment_integer(&max_z), None);
    }

    #[test]
    fn test_decrement_floor() {
        // A + min digits → None (floor)
        let min_a = format!("A{}", "0".repeat(26));
        assert_eq!(decrement_integer(&min_a), None);
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
