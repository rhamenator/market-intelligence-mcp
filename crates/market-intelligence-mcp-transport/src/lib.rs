//! MCP stdio/Streamable-HTTP transport and tool registry.
//!
//! Status: not-started. This crate should implement the current stable MCP
//! specification (https://modelcontextprotocol.io/specification/) using an
//! official or well-maintained Rust SDK once one is selected, expose tools
//! in deterministic order, and enforce `ToolDescriptor` (risk tier,
//! read-only/destructive/idempotent flags, allowed modes/roles) at the
//! registry layer rather than trusting MCP annotations. See
//! `market-system-contracts/schemas/2.0.0/tool-descriptor.schema.json`.
