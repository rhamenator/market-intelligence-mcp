//! `market-intelligence-server` — Phase 2 fixture-only vertical slice,
//! now with a real MCP stdio server (`mcp` subcommand).
//!
//! Status: fixture-tested + real end-to-end MCP transport. This binary
//! wires together the source-policy engine, a fixture source adapter, and
//! the `NoActionBaseline` signal to prove the crates compose end to end
//! with no network access and no external credentials. Run with no
//! arguments for a one-shot demo; `--check` runs the same path and exits
//! nonzero on any policy violation (useful in CI); `mcp` runs a real MCP
//! stdio JSON-RPC server (`market_intelligence_mcp_transport`) exposing
//! the same underlying logic as 11 callable tools.

use market_intelligence_core::{
    BundlePurpose, CorrectionState, EvidenceBundle, EvidenceRecordRef, MnpiClassification,
    SourceRecord, SourceType, SourceUseDecision, UsePermission, UtcTimestamp,
};
use market_intelligence_mcp_transport::mcp::run_stdio;
use market_intelligence_mcp_transport::state::IntelligenceState;
use market_intelligence_signals::{NoActionBaseline, SignalPlugin};
use market_intelligence_source_sdk::{FixtureSourceAdapter, SourceAdapter};

fn demo_fixture_record() -> SourceRecord {
    SourceRecord {
        schema_version: "2.0.0".into(),
        source_record_id: "fixture-political-disclosure-1".into(),
        source_id: "us-house-periodic-transaction-report".into(),
        provider_record_id: None,
        source_type: SourceType::OfficialFiling,
        jurisdiction: Some("US".into()),
        retrieval_method: "fixture".into(),
        transaction_time: Some(UtcTimestamp::UNIX_EPOCH),
        event_time: None,
        filing_time: Some(UtcTimestamp::UNIX_EPOCH),
        published_time: Some(UtcTimestamp::UNIX_EPOCH),
        public_availability_time: Some(UtcTimestamp::UNIX_EPOCH),
        retrieved_time: UtcTimestamp::UNIX_EPOCH,
        corrected_time: None,
        content_hash: Some("sha256:0000000000000000000000000000000000000000000000000000000000000".into()),
        execution_eligible: false,
        redistribution_eligible: false,
        quality: Some(0.8),
        staleness_seconds: Some(3_024_000),
        expected_reporting_lag_seconds: Some(3_024_000),
        revision: 1,
        correction_state: CorrectionState::Original,
        untrusted_text: false,
        mnpi_classification: MnpiClassification::PublicConfirmed,
    }
}

fn demo_decision() -> SourceUseDecision {
    SourceUseDecision {
        source_id: "us-house-periodic-transaction-report".into(),
        policy_version: "2026-07-19.1".into(),
        public_access: true,
        automated_retrieval: UsePermission::LegalReviewRequired,
        commercial_use: UsePermission::ProhibitedOrUnclear,
        trading_use: UsePermission::ResearchOnly,
        redistribution: UsePermission::ProhibitedOrUnclear,
        model_training: None,
        retention_days: None,
        attribution_required: true,
        terms_snapshot_hash: None,
        robots_snapshot_hash: None,
        contract_id: None,
        reviewed_at: UtcTimestamp::UNIX_EPOCH,
        reviewed_by: "operator".into(),
        reason_codes: vec!["statutory-commercial-use-restriction".into()],
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("mcp") {
        let adapter = FixtureSourceAdapter::new(
            "us-house-periodic-transaction-report",
            vec![demo_fixture_record()],
        );
        let mut state = IntelligenceState::new(vec![adapter]);
        if let Err(e) = run_stdio(&mut state) {
            eprintln!("market-intelligence-server: stdio transport error: {e}");
            std::process::exit(1);
        }
        return;
    }

    let check_mode = args.iter().any(|arg| arg == "--check");

    let adapter = FixtureSourceAdapter::new(
        "us-house-periodic-transaction-report",
        vec![demo_fixture_record()],
    );
    let records = adapter.records();
    let record = &records[0];
    let decision = demo_decision();
    let now = UtcTimestamp::UNIX_EPOCH;

    let research_result =
        market_intelligence_source_policy::evaluate_record(record, &decision, BundlePurpose::Research, now);
    let live_result = market_intelligence_source_policy::evaluate_record(
        record,
        &decision,
        BundlePurpose::LiveExecution,
        now,
    );

    println!("source: {}", adapter.source_id());
    println!("research-purpose policy check: {:?}", research_result);
    println!(
        "live-execution-purpose policy check (expected denial, research-only source): {:?}",
        live_result
    );

    let bundle = EvidenceBundle {
        evidence_bundle_id: "demo-bundle-1".into(),
        created_at: now,
        decision_time: Some(now),
        records: vec![EvidenceRecordRef {
            source_record_id: record.source_record_id.clone(),
            content_hash: record.content_hash.clone(),
            source_use_decision_id: Some(decision.source_id.clone()),
            mnpi_classification: record.mnpi_classification,
            public_availability_time: record.public_availability_time,
            freshness_seconds: record.staleness_seconds,
            execution_eligible: research_result.is_ok(),
        }],
        bundle_purpose: BundlePurpose::Research,
        all_records_execution_eligible: false,
        quarantine_count: 0,
        service_attestation: None,
        canonical_hash: "sha256:demo".into(),
    };

    let assessment = NoActionBaseline.evaluate(&bundle);
    println!(
        "signal: {:?} no-action-score={:?} intent_inferred={}",
        assessment.signal_type, assessment.no_action_score, assessment.intent_inferred
    );

    if check_mode {
        let mut failures = Vec::new();
        if research_result.is_err() {
            failures.push("research-purpose check should have succeeded".to_string());
        }
        if live_result.is_ok() {
            failures.push("live-execution-purpose check should have been denied".to_string());
        }
        if assessment.no_action_score != Some(1.0) || assessment.intent_inferred {
            failures.push("NoActionBaseline invariant violated".to_string());
        }
        if failures.is_empty() {
            println!("--check: all invariants held");
        } else {
            eprintln!("--check: FAILED");
            for f in &failures {
                eprintln!("  - {f}");
            }
            std::process::exit(1);
        }
    }
}
