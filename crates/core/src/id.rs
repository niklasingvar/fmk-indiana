//! Id generator — pronounceable syllable tokens (D2, IN_IDENTITY.md).
//!
//! Pattern: `[a-z]+-[a-z]+(-[0-9]+)?` (e.g. `frata-nimta`, `lurvo-pannik`).
//! Generated, not drawn from a dictionary: syllables are random CV(C) shapes.

use fastrand::Rng;
use std::collections::HashSet;

/// Syllable shapes: CV or CVC.
const SYLLABLES: &[&str] = &[
    "ba", "be", "bi", "bo", "bu", "da", "de", "di", "do", "du", "fa", "fe", "fi", "fo", "fu", "ga",
    "ge", "gi", "go", "gu", "ha", "he", "hi", "ho", "hu", "ja", "je", "ji", "jo", "ju", "ka", "ke",
    "ki", "ko", "ku", "la", "le", "li", "lo", "lu", "ma", "me", "mi", "mo", "mu", "na", "ne", "ni",
    "no", "nu", "pa", "pe", "pi", "po", "pu", "ra", "re", "ri", "ro", "ru", "sa", "se", "si", "so",
    "su", "ta", "te", "ti", "to", "tu", "va", "ve", "vi", "vo", "vu", "wa", "we", "wi", "wo", "wu",
    "ya", "ye", "yi", "yo", "yu", "za", "ze", "zi", "zo", "zu",
    // CVC variants for variety.
    "bak", "bel", "bin", "bok", "bun", "dak", "del", "din", "dok", "dun", "fak", "fel", "fin",
    "fok", "fun", "gak", "gel", "gin", "gok", "gun", "hak", "hel", "hin", "hok", "hun", "jak",
    "jel", "jin", "jok", "jun", "kak", "kel", "kin", "kok", "kun", "lak", "lel", "lin", "lok",
    "lun", "mak", "mel", "min", "mok", "mun", "nak", "nel", "nin", "nok", "nun", "pak", "pel",
    "pin", "pok", "pun", "rak", "rel", "rin", "rok", "run", "sak", "sel", "sin", "sok", "sun",
    "tak", "tel", "tin", "tok", "tun", "vak", "vel", "vin", "vok", "vun", "wak", "wel", "win",
    "wok", "wun", "yak", "yel", "yin", "yok", "yun", "zak", "zel", "zin", "zok", "zun",
];

/// Generate one pronounceable id: `syl1-syl2`.
pub fn generate(rng: &mut Rng) -> String {
    let a = SYLLABLES[rng.u16(..SYLLABLES.len() as u16) as usize];
    let b = SYLLABLES[rng.u16(..SYLLABLES.len() as u16) as usize];
    format!("{a}-{b}")
}

/// Collision-safe id generator. Tracks ids seen this scan; appends `-N` on
/// collision (D2).
pub struct IdGenerator {
    rng: Rng,
    seen: HashSet<String>,
}

impl IdGenerator {
    pub fn new() -> Self {
        Self {
            rng: Rng::new(),
            seen: HashSet::new(),
        }
    }

    /// Return a unique id. On collision, appends `-2`, `-3`, etc.
    pub fn next(&mut self) -> String {
        loop {
            let base = generate(&mut self.rng);
            if self.seen.insert(base.clone()) {
                return base;
            }
            // Collision: try numbered variants.
            for n in 2.. {
                let candidate = format!("{base}-{n}");
                if self.seen.insert(candidate.clone()) {
                    return candidate;
                }
            }
        }
    }

    /// Check if an id is already seen (for repair: keep the existing valid id).
    pub fn contains(&self, id: &str) -> bool {
        self.seen.contains(id)
    }

    /// Register an existing valid id so it is not reused.
    pub fn register(&mut self, id: &str) {
        self.seen.insert(id.to_string());
    }

    /// Validate an id against the pattern `[a-z]+-[a-z]+(-[0-9]+)?`.
    pub fn is_valid(id: &str) -> bool {
        let mut parts = id.splitn(2, '-');
        match (parts.next(), parts.next()) {
            (Some(a), Some(b)) => {
                let a_ok = !a.is_empty() && a.chars().all(|c| c.is_ascii_lowercase());
                // b is either a syllable or `syllable-N`.
                let b_ok = if let Some(dash_pos) = b.rfind('-') {
                    // Could be a collision suffix: check if the part after '-' is numeric.
                    let base = &b[..dash_pos];
                    let suffix = &b[dash_pos + 1..];
                    !base.is_empty()
                        && base.chars().all(|c| c.is_ascii_lowercase())
                        && !suffix.is_empty()
                        && suffix.chars().all(|c| c.is_ascii_digit())
                } else {
                    !b.is_empty() && b.chars().all(|c| c.is_ascii_lowercase())
                };
                a_ok && b_ok
            }
            _ => false,
        }
    }

    /// Validate a status word. Only `done` and `failed` are valid.
    pub fn is_valid_status(status: &str) -> bool {
        status == "done" || status == "failed"
    }

    /// Parse a bracket string `id` or `id:status`, returning (id, status).
    /// Returns (None, None) if the bracket is malformed.
    pub fn parse_bracket(bracket: &str) -> (Option<String>, Option<String>) {
        let (id_part, status_part) = match bracket.split_once(':') {
            Some((a, b)) => (Some(a.to_string()), Some(b.to_string())),
            None => (Some(bracket.to_string()), None),
        };

        let id = match id_part {
            Some(id) if !id.is_empty() && Self::is_valid(&id) => Some(id),
            _ => None,
        };

        let status = match (&id, status_part) {
            (Some(_), Some(s)) if !s.is_empty() && Self::is_valid_status(&s) => Some(s),
            _ => None,
        };

        (id, status)
    }

    /// Format a bracket string: `[id]` or `[id:status]`.
    pub fn format_bracket(id: &str, status: Option<&str>) -> String {
        match status {
            Some(s) => format!("[{id}:{s}]"),
            None => format!("[{id}]"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_pattern() {
        let mut gen = IdGenerator::new();
        for _ in 0..100 {
            let id = gen.next();
            assert!(IdGenerator::is_valid(&id), "id {id} should match pattern");
        }
    }

    #[test]
    fn test_id_uniqueness() {
        let mut gen = IdGenerator::new();
        let ids: Vec<String> = (0..100).map(|_| gen.next()).collect();
        let unique: HashSet<String> = ids.iter().cloned().collect();
        assert_eq!(ids.len(), unique.len(), "all ids should be unique");
    }

    #[test]
    fn test_id_valid_pattern() {
        assert!(IdGenerator::is_valid("happy-otter"));
        assert!(IdGenerator::is_valid("frata-nimta"));
        assert!(IdGenerator::is_valid("lurvo-pannik"));
        assert!(IdGenerator::is_valid("frata-nimta-2"));
        assert!(IdGenerator::is_valid("ab-cd-3"));
    }

    #[test]
    fn test_id_invalid_pattern() {
        assert!(!IdGenerator::is_valid(""));
        assert!(!IdGenerator::is_valid("not_valid"));
        assert!(!IdGenerator::is_valid("-bad"));
        assert!(!IdGenerator::is_valid("bad-"));
        assert!(!IdGenerator::is_valid("Bad-Otter")); // uppercase
        assert!(!IdGenerator::is_valid("a-b-3c")); // suffix not purely numeric
    }

    #[test]
    fn test_id_collision() {
        let mut gen = IdGenerator::new();
        // Force a collision by registering an id.
        let first = gen.next();
        gen.seen.insert(first.clone());
        // Next call should produce a different id.
        let second = gen.next();
        assert_ne!(first, second);
    }

    #[test]
    fn test_parse_bracket_valid() {
        let (id, status) = IdGenerator::parse_bracket("happy-otter");
        assert_eq!(id, Some("happy-otter".to_string()));
        assert_eq!(status, None);

        let (id, status) = IdGenerator::parse_bracket("happy-otter:done");
        assert_eq!(id, Some("happy-otter".to_string()));
        assert_eq!(status, Some("done".to_string()));

        let (id, status) = IdGenerator::parse_bracket("happy-otter:failed");
        assert_eq!(id, Some("happy-otter".to_string()));
        assert_eq!(status, Some("failed".to_string()));
    }

    #[test]
    fn test_parse_bracket_invalid_id() {
        // Bad id → (None, None).
        let (id, status) = IdGenerator::parse_bracket("not_valid:done");
        assert_eq!(id, None);
        // Status is dropped when id is invalid.
        assert_eq!(status, None);

        let (id, status) = IdGenerator::parse_bracket("");
        assert_eq!(id, None);
        assert_eq!(status, None);
    }

    #[test]
    fn test_parse_bracket_invalid_status() {
        // Valid id, bad status → (Some(id), None).
        let (id, status) = IdGenerator::parse_bracket("happy-otter:unknown");
        assert_eq!(id, Some("happy-otter".to_string()));
        assert_eq!(status, None);

        let (id, status) = IdGenerator::parse_bracket("happy-otter:pending");
        assert_eq!(id, Some("happy-otter".to_string()));
        assert_eq!(status, None);
    }

    #[test]
    fn test_format_bracket() {
        assert_eq!(
            IdGenerator::format_bracket("happy-otter", None),
            "[happy-otter]"
        );
        assert_eq!(
            IdGenerator::format_bracket("happy-otter", Some("done")),
            "[happy-otter:done]"
        );
        assert_eq!(
            IdGenerator::format_bracket("happy-otter", Some("failed")),
            "[happy-otter:failed]"
        );
    }

    #[test]
    fn test_register_prevents_reuse() {
        let mut gen = IdGenerator::new();
        gen.register("happy-otter");
        assert!(gen.contains("happy-otter"));
    }

    #[test]
    fn test_is_valid_status() {
        assert!(IdGenerator::is_valid_status("done"));
        assert!(IdGenerator::is_valid_status("failed"));
        assert!(!IdGenerator::is_valid_status("pending"));
        assert!(!IdGenerator::is_valid_status("unknown"));
    }
}
