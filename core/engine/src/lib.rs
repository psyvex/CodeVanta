pub mod finding;

pub use finding::{Confidence, Finding, Severity};

#[cfg(test)]
mod tests {
    use super::{Confidence, Finding, Severity};

    #[test]
    fn finding_round_trips_as_json() {
        let finding = Finding {
            rule_id: "security.sql-injection".into(),
            category: "security".into(),
            severity: Severity::High,
            confidence: Confidence::Likely,
            message: "User-controlled input reaches a SQL sink.".into(),
            path: Some("src/db.ts".into()),
            line: Some(42),
        };

        let encoded = serde_json::to_string(&finding).expect("finding should serialize");
        let decoded: Finding = serde_json::from_str(&encoded).expect("finding should deserialize");

        assert_eq!(decoded, finding);
    }
}
