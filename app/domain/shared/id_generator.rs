use tempoid::TempoIdOptions;

pub struct IdGenerator;

const ALPHABET: &str = "0123456789abcdefghijklmnopqrstuvwxyz";

impl IdGenerator {
    pub fn primary_id() -> String {
        tempoid::TempoId::generate_custom(TempoIdOptions {
            time_length: 8,
            random_length: 16,
            alphabet: ALPHABET,
            ..Default::default()
        })
        .to_string()
    }

    pub fn filename() -> String {
        tempoid::TempoId::generate_custom(TempoIdOptions {
            time_length: 8,
            random_length: 24,
            alphabet: ALPHABET,
            ..Default::default()
        })
        .to_string()
    }

    pub fn random() -> String {
        tempoid::TempoId::generate_custom(TempoIdOptions {
            time_length: 8,
            random_length: 12,
            alphabet: ALPHABET,
            ..Default::default()
        })
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primary_id() {
        let id = IdGenerator::primary_id();
        assert_eq!(id.len(), 24);
    }

    #[test]
    fn test_filename() {
        let id = IdGenerator::filename();
        assert_eq!(id.len(), 32);
    }

    #[test]
    fn test_random() {
        let id = IdGenerator::random();
        assert_eq!(id.len(), 20);
    }
}
