//! Tests for Guard (fact-check) types and helpers.

use wauldo::http_types::{GuardClaim, GuardResponse};

fn verified_response() -> GuardResponse {
    GuardResponse {
        verdict: "verified".into(),
        action: "allow".into(),
        hallucination_rate: 0.0,
        mode: "lexical".into(),
        total_claims: 1,
        supported_claims: 1,
        confidence: 1.0,
        claims: vec![GuardClaim {
            text: "Paris is in France".into(),
            claim_type: None,
            supported: true,
            confidence: 1.0,
            confidence_label: None,
            verdict: "verified".into(),
            action: "allow".into(),
            reason: None,
            evidence: None,
        }],
        mode_warning: None,
        processing_time_ms: Some(0),
    }
}

fn rejected_response() -> GuardResponse {
    GuardResponse {
        verdict: "rejected".into(),
        action: "block".into(),
        hallucination_rate: 1.0,
        mode: "lexical".into(),
        total_claims: 1,
        supported_claims: 0,
        confidence: 0.0,
        claims: vec![GuardClaim {
            text: "Returns within 60 days".into(),
            claim_type: None,
            supported: false,
            confidence: 0.3,
            confidence_label: None,
            verdict: "rejected".into(),
            action: "block".into(),
            reason: Some("numerical_mismatch".into()),
            evidence: None,
        }],
        mode_warning: None,
        processing_time_ms: Some(0),
    }
}

fn weak_response() -> GuardResponse {
    GuardResponse {
        verdict: "weak".into(),
        action: "review".into(),
        hallucination_rate: 0.5,
        mode: "lexical".into(),
        total_claims: 2,
        supported_claims: 1,
        confidence: 0.5,
        claims: vec![
            GuardClaim {
                text: "Claim A".into(),
                claim_type: None,
                supported: true,
                confidence: 0.8,
                confidence_label: None,
                verdict: "verified".into(),
                action: "allow".into(),
                reason: None,
                evidence: None,
            },
            GuardClaim {
                text: "Claim B".into(),
                claim_type: None,
                supported: false,
                confidence: 0.2,
                confidence_label: None,
                verdict: "rejected".into(),
                action: "block".into(),
                reason: Some("insufficient_evidence".into()),
                evidence: None,
            },
        ],
        mode_warning: None,
        processing_time_ms: None,
    }
}

#[test]
fn test_guard_verified_is_safe() {
    let resp = verified_response();
    assert!(resp.is_safe());
    assert!(!resp.is_blocked());
    assert_eq!(resp.confidence, 1.0);
    assert_eq!(resp.claims.len(), 1);
    assert!(resp.claims[0].supported);
}

#[test]
fn test_guard_rejected_is_blocked() {
    let resp = rejected_response();
    assert!(!resp.is_safe());
    assert!(resp.is_blocked());
    assert_eq!(resp.confidence, 0.0);
    assert_eq!(resp.claims[0].reason.as_deref(), Some("numerical_mismatch"));
}

#[test]
fn test_guard_weak_neither_safe_nor_blocked() {
    let resp = weak_response();
    assert!(!resp.is_safe());
    assert!(!resp.is_blocked());
    assert_eq!(resp.action, "review");
    assert_eq!(resp.total_claims, 2);
    assert_eq!(resp.supported_claims, 1);
}

#[test]
fn test_guard_response_json_roundtrip() {
    let resp = verified_response();
    let json = serde_json::to_string(&resp).expect("serialize");
    let parsed: GuardResponse = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(parsed.verdict, "verified");
    assert!(parsed.is_safe());
    assert_eq!(parsed.claims.len(), 1);
}

#[test]
fn test_guard_response_from_api_json() {
    let json = r#"{
        "verdict": "rejected",
        "action": "block",
        "hallucination_rate": 1.0,
        "mode": "lexical",
        "total_claims": 1,
        "supported_claims": 0,
        "confidence": 0.0,
        "claims": [{
            "text": "60 days",
            "supported": false,
            "confidence": 0.3,
            "verdict": "rejected",
            "action": "block",
            "reason": "numerical_mismatch"
        }]
    }"#;
    let resp: GuardResponse = serde_json::from_str(json).expect("parse API response");
    assert!(resp.is_blocked());
    assert!(!resp.is_safe());
    assert_eq!(resp.claims[0].reason.as_deref(), Some("numerical_mismatch"));
}
