//! MCP stdio transport and tool registry.
//!
//! **Status: fixture-tested + real end-to-end.** A hand-rolled MCP stdio
//! JSON-RPC 2.0 transport (no MCP SDK dependency, matching the sibling
//! repositories' own hand-rolled transports) exposing 11 read-only tools
//! over the existing fixture/in-memory-storage/mock-signal crates. See
//! `tools.rs`'s module doc comment for the exact tool list and
//! `docs/CAPABILITY_STATUS.md` for what's real versus fixture/mock behind
//! each tool. Streamable HTTP is not implemented — stdio only, same
//! trusted-local-caller precedent the sibling repositories use.
//!
//! No order-entry tool exists in this crate, and none ever will — see the
//! workspace README "Why this is a separate repository."

pub mod evidence_builder;
pub mod mcp;
pub mod state;
pub mod tools;
