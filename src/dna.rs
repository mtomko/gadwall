pub fn is_strict_dna(dna: &str) -> bool {
    return dna
        .chars()
        .all(|b| matches!(b, 'A' | 'a' | 'T' | 't' | 'C' | 'c' | 'G' | 'g'));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_strict_dna_test() {
        assert_eq!(is_strict_dna("CATAGGTTG"), true);
        assert_eq!(is_strict_dna("cataggttg"), true);
        assert_eq!(is_strict_dna("catangttg"), false);
    }
}
