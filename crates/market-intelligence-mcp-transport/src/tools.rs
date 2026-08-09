//! Tool implementations dispatched by `mcp.rs`. Read-only intelligence
//! tools only — **no order-entry tool exists anywhere in this crate, and
//! never will**: this repository's whole reason for being a separate
//! security boundary is that it must never hold broker credentials or
//! submit orders (see the workspace README "Why this is a separate
//! repository"). Every tool here operates on fixture/in-memory data:
//!
//! ```text
//! health, capabilities, tool-catalog, self-test
//! list-configured-sources, ingest-source-records
//! get-source-record-history, get-source-record-as-of
//! check-source-use-eligibility, build-evidence-bundle, evaluate-signal
//! ```

use serde_json::{Value, json};

use market_intelligence_core::{
    BundlePurpose, EvidenceBundle, SourceRecord, SourceUseDecision, UtcTimestamp,
};
use market_intelligence_signals::{NoActionBaseline, SignalPlugin};
use market_intelligence_source_sdk::SourceAdapter;
use market_intelligence_storage::SourceRecordStore;

use crate::evidence_builder::{RecordAndDecision, build_evidence_bundle};
use crate::state::IntelligenceState;

pub fn tool_definitions() -> Value {
    json!([
        {"name": "health", "description": "Service health; proves no order-entry tool or broker credential exists in this build.", "inputSchema": {"type": "object", "properties": {}, "additionalProperties": false}},
        {"name": "capabilities", "description": "What this service can and cannot do: fixture-only sources, in-memory storage, one mock signal.", "inputSchema": {"type": "object", "properties": {}, "additionalProperties": false}},
        {"name": "tool-catalog", "description": "The full list of tools this server exposes.", "inputSchema": {"type": "object", "properties": {}, "additionalProperties": false}},
        {"name": "self-test", "description": "Runs internal invariant checks (hash/timestamp primitives, MNPI policy gate, signal baseline) and reports pass/fail.", "inputSchema": {"type": "object", "properties": {}, "additionalProperties": false}},
        {"name": "list-configured-sources", "description": "Lists the source_id of every configured (fixture) source adapter.", "inputSchema": {"type": "object", "properties": {}, "additionalProperties": false}},
        {"name": "ingest-source-records", "description": "Pulls current records from a configured adapter into the in-memory store.", "inputSchema": {"type": "object", "properties": {"source_id": {"type": "string"}}, "required": ["source_id"], "additionalProperties": false}},
        {"name": "get-source-record-history", "description": "Every stored revision of a source record, oldest first.", "inputSchema": {"type": "object", "properties": {"source_record_id": {"type": "string"}}, "required": ["source_record_id"], "additionalProperties": false}},
        {"name": "get-source-record-as-of", "description": "The revision that was current as of a given public-availability time -- the point-in-time read path.", "inputSchema": {"type": "object", "properties": {"source_record_id": {"type": "string"}, "as_of": {"type": "string"}}, "required": ["source_record_id", "as_of"], "additionalProperties": false}},
        {"name": "check-source-use-eligibility", "description": "Runs the deterministic MNPI/source-policy gate for one record against a decision and purpose.", "inputSchema": {"type": "object", "properties": {"record": {"type": "object"}, "decision": {"type": "object"}, "purpose": {"type": "string"}, "decision_time": {"type": "string"}}, "required": ["record", "decision", "purpose", "decision_time"], "additionalProperties": false}},
        {"name": "build-evidence-bundle", "description": "Builds a signed-shape EvidenceBundle from one or more (record, decision) pairs, running the policy gate on each.", "inputSchema": {"type": "object", "properties": {"evidence_bundle_id": {"type": "string"}, "items": {"type": "array"}, "purpose": {"type": "string"}, "decision_time": {"type": "string"}}, "required": ["evidence_bundle_id", "items", "purpose", "decision_time"], "additionalProperties": false}},
        {"name": "evaluate-signal", "description": "Runs the NoActionBaseline signal (the only signal implemented) against an EvidenceBundle.", "inputSchema": {"type": "object", "properties": {"evidence": {"type": "object"}}, "required": ["evidence"], "additionalProperties": false}},
    ])
}

fn pretty(value: &Value) -> String {
    serde_json::to_string_pretty(value).expect("Value serialization is infallible")
}

pub fn health(state: &IntelligenceState) -> Result<String, String> {
    Ok(pretty(&json!({
        "status": "ok",
        "order_entry_tools_present": false,
        "broker_credentials_present": false,
        "configured_source_count": state.adapters.len(),
    })))
}

pub fn capabilities() -> Result<String, String> {
    Ok(pretty(&json!({
        "source_adapters": "fixture-only",
        "storage": "in-memory-only",
        "signals": ["no-action-baseline (mock)"],
        "entity_resolution": false,
        "mcp_transport": "stdio-only",
        "order_entry": false,
    })))
}

pub fn tool_catalog() -> Result<String, String> {
    Ok(pretty(&json!({
        "tools": tool_definitions(),
        "order_entry_tools_absent_because": "not compiled into this repository at all -- see tools.rs module doc comment",
    })))
}

pub fn self_test() -> Result<String, String> {
    let mut checks: Vec<(&'static str, bool)> = Vec::new();

    checks.push((
        "sha256_empty_string_matches_nist_vector",
        market_intelligence_core::sha256_hex(b"")
            == "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    ));

    let timestamp_round_trip = UtcTimestamp::parse_rfc3339("2026-07-19T00:00:00Z")
        .map(|t| t.to_rfc3339() == "2026-07-19T00:00:00Z")
        .unwrap_or(false);
    checks.push(("utc_timestamp_round_trip", timestamp_round_trip));

    let research_only_denied_for_live = {
        let record = fixture_record_for_self_test();
        let decision =
            fixture_decision_for_self_test(market_intelligence_core::UsePermission::ResearchOnly);
        market_intelligence_source_policy::evaluate_record(
            &record,
            &decision,
            BundlePurpose::LiveExecution,
            UtcTimestamp::UNIX_EPOCH,
        )
        .is_err()
    };
    checks.push((
        "research_only_source_denied_for_live_execution",
        research_only_denied_for_live,
    ));

    let research_only_allowed_for_research = {
        let record = fixture_record_for_self_test();
        let decision =
            fixture_decision_for_self_test(market_intelligence_core::UsePermission::ResearchOnly);
        market_intelligence_source_policy::evaluate_record(
            &record,
            &decision,
            BundlePurpose::Research,
            UtcTimestamp::UNIX_EPOCH,
        )
        .is_ok()
    };
    checks.push((
        "research_only_source_allowed_for_research",
        research_only_allowed_for_research,
    ));

    let no_action_baseline_never_recommends = {
        let bundle = fixture_bundle_for_self_test();
        let assessment = NoActionBaseline.evaluate(&bundle);
        assessment.no_action_score == Some(1.0) && !assessment.intent_inferred
    };
    checks.push((
        "no_action_baseline_never_recommends_direction",
        no_action_baseline_never_recommends,
    ));

    let all_passed = checks.iter().all(|(_, ok)| *ok);
    let result = json!({
        "status": if all_passed { "PASS" } else { "FAIL" },
        "checks": checks.into_iter().map(|(name, ok)| json!({"name": name, "passed": ok})).collect::<Vec<_>>(),
    });
    if all_passed {
        Ok(pretty(&result))
    } else {
        Err(pretty(&result))
    }
}

fn fixture_record_for_self_test() -> SourceRecord {
    SourceRecord {
        schema_version: "2.0.0".into(),
        source_record_id: "self-test-record".into(),
        source_id: "self-test-source".into(),
        provider_record_id: None,
        source_type: market_intelligence_core::SourceType::Other,
        jurisdiction: None,
        retrieval_method: "self-test".into(),
        transaction_time: None,
        event_time: None,
        filing_time: None,
        published_time: None,
        public_availability_time: Some(UtcTimestamp::UNIX_EPOCH),
        retrieved_time: UtcTimestamp::UNIX_EPOCH,
        corrected_time: None,
        content_hash: None,
        execution_eligible: true,
        redistribution_eligible: false,
        quality: None,
        staleness_seconds: None,
        expected_reporting_lag_seconds: None,
        revision: 1,
        correction_state: market_intelligence_core::CorrectionState::Original,
        untrusted_text: false,
        mnpi_classification: market_intelligence_core::MnpiClassification::PublicConfirmed,
    }
}

fn fixture_decision_for_self_test(
    trading_use: market_intelligence_core::UsePermission,
) -> SourceUseDecision {
    SourceUseDecision {
        source_id: "self-test-source".into(),
        policy_version: "self-test".into(),
        public_access: true,
        automated_retrieval: market_intelligence_core::UsePermission::LegalReviewRequired,
        commercial_use: market_intelligence_core::UsePermission::ProhibitedOrUnclear,
        trading_use,
        redistribution: market_intelligence_core::UsePermission::ProhibitedOrUnclear,
        model_training: None,
        retention_days: None,
        attribution_required: true,
        terms_snapshot_hash: None,
        robots_snapshot_hash: None,
        contract_id: None,
        reviewed_at: UtcTimestamp::UNIX_EPOCH,
        reviewed_by: "self-test".into(),
        reason_codes: vec![],
    }
}

fn fixture_bundle_for_self_test() -> EvidenceBundle {
    EvidenceBundle {
        evidence_bundle_id: "self-test-bundle".into(),
        created_at: UtcTimestamp::UNIX_EPOCH,
        decision_time: None,
        records: vec![],
        bundle_purpose: BundlePurpose::Research,
        all_records_execution_eligible: false,
        quarantine_count: 0,
        service_attestation: None,
        canonical_hash: "sha256:0000000000000000000000000000000000000000000000000000000000000000"
            .to_string(),
    }
}

pub fn list_configured_sources(state: &IntelligenceState) -> Result<String, String> {
    let ids: Vec<&str> = state.adapters.iter().map(|a| a.source_id()).collect();
    Ok(pretty(&json!({ "source_ids": ids })))
}

pub fn ingest_source_records(
    state: &mut IntelligenceState,
    arguments: &Value,
) -> Result<String, String> {
    let source_id = arguments
        .get("source_id")
        .and_then(Value::as_str)
        .ok_or_else(|| "\"source_id\" is required".to_string())?;
    let records = state
        .adapter(source_id)
        .ok_or_else(|| format!("no configured adapter for source_id {source_id:?}"))?
        .records();
    let mut ingested = 0u32;
    let mut errors = Vec::new();
    for record in records {
        let id = record.source_record_id.clone();
        match state.store.append(record) {
            Ok(()) => ingested += 1,
            Err(e) => errors.push(format!("{id}: {e}")),
        }
    }
    Ok(pretty(&json!({ "ingested": ingested, "errors": errors })))
}

pub fn get_source_record_history(
    state: &IntelligenceState,
    arguments: &Value,
) -> Result<String, String> {
    let id = arguments
        .get("source_record_id")
        .and_then(Value::as_str)
        .ok_or_else(|| "\"source_record_id\" is required".to_string())?;
    let history = state.store.history(id);
    Ok(pretty(
        &serde_json::to_value(&history).expect("history serialization is infallible"),
    ))
}

pub fn get_source_record_as_of(
    state: &IntelligenceState,
    arguments: &Value,
) -> Result<String, String> {
    let id = arguments
        .get("source_record_id")
        .and_then(Value::as_str)
        .ok_or_else(|| "\"source_record_id\" is required".to_string())?;
    let as_of_str = arguments
        .get("as_of")
        .and_then(Value::as_str)
        .ok_or_else(|| "\"as_of\" is required".to_string())?;
    let as_of = UtcTimestamp::parse_rfc3339(as_of_str)
        .map_err(|e| format!("invalid as_of timestamp: {e}"))?;
    match state.store.as_of(id, as_of) {
        Some(record) => Ok(pretty(
            &serde_json::to_value(&record).expect("record serialization is infallible"),
        )),
        None => Err(format!(
            "no revision of {id:?} was public as of {as_of_str}"
        )),
    }
}

fn parse_purpose(arguments: &Value) -> Result<BundlePurpose, String> {
    let raw = arguments
        .get("purpose")
        .and_then(Value::as_str)
        .ok_or_else(|| "\"purpose\" is required".to_string())?;
    serde_json::from_value(Value::String(raw.to_string()))
        .map_err(|e| format!("invalid purpose: {e}"))
}

fn parse_decision_time(arguments: &Value) -> Result<UtcTimestamp, String> {
    let raw = arguments
        .get("decision_time")
        .and_then(Value::as_str)
        .ok_or_else(|| "\"decision_time\" is required".to_string())?;
    UtcTimestamp::parse_rfc3339(raw).map_err(|e| format!("invalid decision_time: {e}"))
}

pub fn check_source_use_eligibility(arguments: &Value) -> Result<String, String> {
    let record: SourceRecord = serde_json::from_value(
        arguments
            .get("record")
            .cloned()
            .ok_or("\"record\" is required")?,
    )
    .map_err(|e| format!("invalid record: {e}"))?;
    let decision: SourceUseDecision = serde_json::from_value(
        arguments
            .get("decision")
            .cloned()
            .ok_or("\"decision\" is required")?,
    )
    .map_err(|e| format!("invalid decision: {e}"))?;
    let purpose = parse_purpose(arguments)?;
    let decision_time = parse_decision_time(arguments)?;

    match market_intelligence_source_policy::evaluate_record(
        &record,
        &decision,
        purpose,
        decision_time,
    ) {
        Ok(()) => Ok(pretty(&json!({"eligible": true}))),
        Err(reason) => Err(pretty(
            &json!({"eligible": false, "reason": reason.to_string()}),
        )),
    }
}

pub fn build_evidence_bundle_tool(arguments: &Value) -> Result<String, String> {
    let evidence_bundle_id = arguments
        .get("evidence_bundle_id")
        .and_then(Value::as_str)
        .ok_or_else(|| "\"evidence_bundle_id\" is required".to_string())?;
    let items_raw = arguments
        .get("items")
        .and_then(Value::as_array)
        .ok_or_else(|| "\"items\" must be an array".to_string())?;
    let purpose = parse_purpose(arguments)?;
    let decision_time = parse_decision_time(arguments)?;

    let mut items = Vec::with_capacity(items_raw.len());
    for (i, item) in items_raw.iter().enumerate() {
        let record: SourceRecord = serde_json::from_value(
            item.get("record")
                .cloned()
                .ok_or_else(|| format!("items[{i}].record is required"))?,
        )
        .map_err(|e| format!("items[{i}].record invalid: {e}"))?;
        let decision: SourceUseDecision = serde_json::from_value(
            item.get("decision")
                .cloned()
                .ok_or_else(|| format!("items[{i}].decision is required"))?,
        )
        .map_err(|e| format!("items[{i}].decision invalid: {e}"))?;
        items.push(RecordAndDecision { record, decision });
    }

    let bundle = build_evidence_bundle(
        evidence_bundle_id,
        &items,
        purpose,
        decision_time,
        UtcTimestamp::now(),
    );
    Ok(pretty(
        &serde_json::to_value(&bundle).expect("bundle serialization is infallible"),
    ))
}

pub fn evaluate_signal(arguments: &Value) -> Result<String, String> {
    let bundle: EvidenceBundle = serde_json::from_value(
        arguments
            .get("evidence")
            .cloned()
            .ok_or("\"evidence\" is required")?,
    )
    .map_err(|e| format!("invalid evidence bundle: {e}"))?;
    let assessment = NoActionBaseline.evaluate(&bundle);
    Ok(pretty(
        &serde_json::to_value(&assessment).expect("assessment serialization is infallible"),
    ))
}

/// Dispatches a `tools/call` request's `name`/`arguments` to the matching
/// tool. `Err` becomes an MCP `isError: true` result, not a JSON-RPC
/// protocol error.
pub fn call_tool(
    state: &mut IntelligenceState,
    name: &str,
    arguments: &Value,
) -> Result<String, String> {
    match name {
        "health" => health(state),
        "capabilities" => capabilities(),
        "tool-catalog" => tool_catalog(),
        "self-test" => self_test(),
        "list-configured-sources" => list_configured_sources(state),
        "ingest-source-records" => ingest_source_records(state, arguments),
        "get-source-record-history" => get_source_record_history(state, arguments),
        "get-source-record-as-of" => get_source_record_as_of(state, arguments),
        "check-source-use-eligibility" => check_source_use_eligibility(arguments),
        "build-evidence-bundle" => build_evidence_bundle_tool(arguments),
        "evaluate-signal" => evaluate_signal(arguments),
        other => Err(format!("unknown tool: {other}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use market_intelligence_source_sdk::FixtureSourceAdapter;

    fn test_state() -> IntelligenceState {
        let record = fixture_record_for_self_test();
        let adapter = FixtureSourceAdapter::new("self-test-source", vec![record]);
        IntelligenceState::new(vec![adapter])
    }

    #[test]
    fn health_reports_no_order_entry_tools() {
        let state = test_state();
        let text = health(&state).unwrap();
        assert!(text.contains("\"order_entry_tools_present\": false"));
        assert!(text.contains("\"broker_credentials_present\": false"));
    }

    #[test]
    fn tool_catalog_never_lists_an_order_entry_tool() {
        let names: Vec<String> = tool_definitions()
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["name"].as_str().unwrap().to_string())
            .collect();
        for forbidden in [
            "place-order",
            "submit-order",
            "authorize-and-submit-live-order",
        ] {
            assert!(!names.contains(&forbidden.to_string()));
        }
    }

    #[test]
    fn self_test_passes() {
        assert!(self_test().is_ok());
    }

    #[test]
    fn list_configured_sources_reflects_state() {
        let state = test_state();
        let text = list_configured_sources(&state).unwrap();
        assert!(text.contains("self-test-source"));
    }

    #[test]
    fn ingest_then_get_history_round_trips() {
        let mut state = test_state();
        let ingest_result =
            ingest_source_records(&mut state, &json!({"source_id": "self-test-source"})).unwrap();
        assert!(ingest_result.contains("\"ingested\": 1"));

        let history =
            get_source_record_history(&state, &json!({"source_record_id": "self-test-record"}))
                .unwrap();
        assert!(history.contains("self-test-record"));
    }

    #[test]
    fn ingest_unknown_source_is_an_error() {
        let mut state = test_state();
        assert!(ingest_source_records(&mut state, &json!({"source_id": "unknown"})).is_err());
    }

    #[test]
    fn get_source_record_as_of_finds_the_public_revision() {
        let mut state = test_state();
        ingest_source_records(&mut state, &json!({"source_id": "self-test-source"})).unwrap();
        let result = get_source_record_as_of(
            &state,
            &json!({"source_record_id": "self-test-record", "as_of": "2026-07-19T00:00:00Z"}),
        )
        .unwrap();
        assert!(result.contains("self-test-record"));
    }

    #[test]
    fn check_source_use_eligibility_denies_research_only_for_live() {
        let record = serde_json::to_value(fixture_record_for_self_test()).unwrap();
        let decision = serde_json::to_value(fixture_decision_for_self_test(
            market_intelligence_core::UsePermission::ResearchOnly,
        ))
        .unwrap();
        let args = json!({"record": record, "decision": decision, "purpose": "live-execution", "decision_time": "2026-07-19T00:00:00Z"});
        assert!(check_source_use_eligibility(&args).is_err());
    }

    #[test]
    fn check_source_use_eligibility_allows_research_purpose() {
        let record = serde_json::to_value(fixture_record_for_self_test()).unwrap();
        let decision = serde_json::to_value(fixture_decision_for_self_test(
            market_intelligence_core::UsePermission::ResearchOnly,
        ))
        .unwrap();
        let args = json!({"record": record, "decision": decision, "purpose": "research", "decision_time": "2026-07-19T00:00:00Z"});
        assert!(check_source_use_eligibility(&args).is_ok());
    }

    #[test]
    fn build_evidence_bundle_tool_produces_a_valid_bundle() {
        let record = serde_json::to_value(fixture_record_for_self_test()).unwrap();
        let decision = serde_json::to_value(fixture_decision_for_self_test(
            market_intelligence_core::UsePermission::Allowed,
        ))
        .unwrap();
        let args = json!({
            "evidence_bundle_id": "bundle-1",
            "items": [{"record": record, "decision": decision}],
            "purpose": "live-execution",
            "decision_time": "2026-07-19T00:00:00Z",
        });
        let result = build_evidence_bundle_tool(&args).unwrap();
        assert!(result.contains("\"all-records-execution-eligible\": true"));
    }

    #[test]
    fn evaluate_signal_never_recommends_a_direction() {
        let bundle = json!({
            "evidence-bundle-id": "b1",
            "created-at": "2026-07-19T00:00:00Z",
            "records": [],
            "bundle-purpose": "research",
            "all-records-execution-eligible": false,
            "quarantine-count": 0,
            "canonical-hash": "sha256:0000000000000000000000000000000000000000000000000000000000000000",
        });
        let result = evaluate_signal(&json!({"evidence": bundle})).unwrap();
        assert!(result.contains("\"no-action-score\": 1.0"));
    }

    #[test]
    fn call_tool_returns_an_error_for_an_unknown_tool_name() {
        let mut state = test_state();
        assert!(call_tool(&mut state, "place-order", &json!({})).is_err());
    }
}
