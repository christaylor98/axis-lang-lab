// Wave B: CLI Inspection Commands
//
// Read-only commands for introspecting trace data:
// - axis inspect trace
// - axis inspect core-ir
// - axis inspect annotations
// - axis explain <node-id>
//
// All commands are READ-ONLY and output stable, machine-readable formats.

use crate::introspection::{Inspector, TraceGraph};
use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════════════
// OUTPUT FORMATS
// ═══════════════════════════════════════════════════════════════════════════

/// Output format for inspection commands
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Human-readable text
    Text,
    /// JSON (machine-readable)
    Json,
}

impl OutputFormat {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "text" | "txt" => Ok(Self::Text),
            "json" => Ok(Self::Json),
            _ => Err(format!("Unknown format: {}", s)),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SERIALIZABLE OUTPUT TYPES
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Serialize, Deserialize)]
pub struct TraceStatsOutput {
    pub total_nodes: usize,
    pub total_links: usize,
    pub token_count: usize,
    pub generic_ast_count: usize,
    pub schema_ast_count: usize,
    pub core_ir_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeOriginOutput {
    pub trace_id: u64,
    pub description: String,
    pub source_spans: Vec<SpanOutput>,
    pub source_file: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpanOutput {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnnotationOutput {
    pub key: String,
    pub value: String,
}

// ═══════════════════════════════════════════════════════════════════════════
// COMMAND IMPLEMENTATIONS
// ═══════════════════════════════════════════════════════════════════════════

/// Inspect trace graph statistics
pub fn cmd_inspect_trace(trace: &TraceGraph, format: OutputFormat) -> Result<(), String> {
    let inspector = Inspector::new(trace.clone());
    let stats = inspector.stats();

    match format {
        OutputFormat::Text => {
            println!("{}", stats);
            Ok(())
        }
        OutputFormat::Json => {
            let output = TraceStatsOutput {
                total_nodes: stats.total_nodes,
                total_links: stats.total_links,
                token_count: stats.token_count,
                generic_ast_count: stats.generic_ast_count,
                schema_ast_count: stats.schema_ast_count,
                core_ir_count: stats.core_ir_count,
            };
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
            Ok(())
        }
    }
}

/// Inspect Core IR node
pub fn cmd_inspect_core_ir(
    trace: &TraceGraph,
    core_ir_id: u64,
    format: OutputFormat,
) -> Result<(), String> {
    let inspector = Inspector::new(trace.clone());

    let origin = inspector
        .origin(core_ir_id)
        .ok_or_else(|| format!("Core IR node {} not found", core_ir_id))?;

    match format {
        OutputFormat::Text => {
            println!("Core IR Node #{}", core_ir_id);
            println!("Description: {}", origin.description);
            if let Some(file) = &origin.source_file {
                println!("Source File: {}", file);
            }
            println!("Source Spans:");
            for span in &origin.source_spans {
                println!("  {}..{}", span.start, span.end);
            }

            if let Some(schema_info) = inspector.schema_rule(core_ir_id) {
                println!("Schema Node: {}", schema_info.node_kind);
                if let Some(rule) = schema_info.rule_name {
                    println!("Schema Rule: {}", rule);
                }
            }

            if let Some(lowering_info) = inspector.lowering_rule(core_ir_id) {
                if let Some(rule) = lowering_info.rule_name {
                    println!("Lowering Rule: {}", rule);
                }
                if let Some(phase) = lowering_info.phase {
                    println!("Lowering Phase: {}", phase);
                }
            }

            Ok(())
        }
        OutputFormat::Json => {
            let output = NodeOriginOutput {
                trace_id: origin.trace_id.as_u64(),
                description: origin.description,
                source_spans: origin
                    .source_spans
                    .iter()
                    .map(|s| SpanOutput {
                        start: s.start,
                        end: s.end,
                    })
                    .collect(),
                source_file: origin.source_file,
            };
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
            Ok(())
        }
    }
}

/// Inspect annotations on Core IR node
pub fn cmd_inspect_annotations(
    trace: &TraceGraph,
    core_ir_id: u64,
    format: OutputFormat,
) -> Result<(), String> {
    let inspector = Inspector::new(trace.clone());

    let ann_info = inspector
        .annotations(core_ir_id)
        .ok_or_else(|| format!("Core IR node {} not found", core_ir_id))?;

    match format {
        OutputFormat::Text => {
            println!("Annotations for Core IR Node #{}:", core_ir_id);
            println!("\nUser Annotations:");
            for ann in &ann_info.user {
                println!("  {} = {:?}", ann.key, ann.value);
            }
            println!("\nProvenance Annotations:");
            for ann in &ann_info.provenance {
                println!("  {} = {:?}", ann.key, ann.value);
            }
            Ok(())
        }
        OutputFormat::Json => {
            let output: Vec<AnnotationOutput> = ann_info
                .all
                .iter()
                .map(|ann| AnnotationOutput {
                    key: ann.key.clone(),
                    value: format!("{:?}", ann.value),
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
            Ok(())
        }
    }
}

/// Explain Core IR node (human-readable summary)
pub fn cmd_explain(trace: &TraceGraph, core_ir_id: u64) -> Result<(), String> {
    let inspector = Inspector::new(trace.clone());

    let explanation = inspector
        .explain(core_ir_id)
        .ok_or_else(|| format!("Core IR node {} not found", core_ir_id))?;

    println!("{}", explanation);
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_format_parsing() {
        assert_eq!(OutputFormat::from_str("text").unwrap(), OutputFormat::Text);
        assert_eq!(OutputFormat::from_str("json").unwrap(), OutputFormat::Json);
        assert_eq!(OutputFormat::from_str("JSON").unwrap(), OutputFormat::Json);
        assert!(OutputFormat::from_str("xml").is_err());
    }
}
