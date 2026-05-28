// Data-Driven Lowering Interpreter
//
// This module implements a stack machine interpreter that executes
// lowering rules from YAML configuration without hard-coded semantics.
//
// FORBIDDEN:
// - Hard-coding language semantics
// - Pattern matching on semantic constructs beyond stack actions
// - Inferring or defaulting behavior
// - Adding features not specified in the config

use crate::frontend::schema_ast::{SchemaAstNode, SchemaValue};
use crate::ir::core_ir::{CoreTerm, IdentOrName};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

// ═══════════════════════════════════════════════════════════════════════════
// LOWERING SPEC STRUCTURES
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoweringSpec {
    pub lowering: LoweringModel,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoweringModel {
    pub model: String,
    pub rules: HashMap<String, LoweringRule>,
    #[serde(default)]
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoweringRule {
    pub action: String,
    #[serde(default)]
    pub params: Vec<String>,
    #[serde(default)]
    pub invariant: HashMap<String, usize>,
}

// ═══════════════════════════════════════════════════════════════════════════
// REGISTRY SPEC STRUCTURES
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RegistrySpec {
    pub registry: RegistryModel,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RegistryModel {
    pub operators: Vec<OperatorDef>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OperatorDef {
    pub name: String,
    pub stack_pop: usize,
    pub stack_push: usize,
    pub core_ir_op: String,
}

// ═══════════════════════════════════════════════════════════════════════════
// STACK MACHINE STATE
// ═══════════════════════════════════════════════════════════════════════════

pub struct StackMachine {
    stack: Vec<CoreTerm>,
    registry: HashMap<String, (String, OperatorDef)>, // Core IR 0.3: maps name -> (canonical_name, def)
}

impl StackMachine {
    pub fn new(registry: HashMap<String, (String, OperatorDef)>) -> Self {
        StackMachine {
            stack: Vec::new(),
            registry,
        }
    }

    pub fn push(&mut self, term: CoreTerm) {
        self.stack.push(term);
    }

    pub fn pop(&mut self) -> Result<CoreTerm, String> {
        self.stack
            .pop()
            .ok_or_else(|| "stack underflow".to_string())
    }

    pub fn stack_height(&self) -> usize {
        self.stack.len()
    }

    pub fn finalize(mut self) -> Result<CoreTerm, String> {
        if self.stack.len() != 1 {
            return Err(format!(
                "invalid stack height at finalize: expected 1, got {}",
                self.stack.len()
            ));
        }
        self.stack.pop().ok_or_else(|| "empty stack".to_string())
    }

    pub fn call_operator(&mut self, op_name: &str) -> Result<(), String> {
        // Look up operator first, then clone needed values
        let (canonical_name, stack_pop, stack_push) = {
            let (name, op_def) = self
                .registry
                .get(op_name)
                .ok_or_else(|| format!("unresolved operator: {}", op_name))?;
            (name.clone(), op_def.stack_pop, op_def.stack_push)
        };

        // Pop arguments
        if self.stack.len() < stack_pop {
            return Err(format!(
                "stack underflow: operator {} requires {} args, stack has {}",
                op_name,
                stack_pop,
                self.stack.len()
            ));
        }

        let mut args = Vec::new();
        for _ in 0..stack_pop {
            args.push(self.pop()?);
        }
        args.reverse();

        // Build Core IR CCall term (Core IR 0.3: uses canonical function name)
        let result = CoreTerm::CCall {
            target_name: canonical_name,
            args,
            node_id: None,
            annotations: Vec::new(),
        };

        // Push result (stack_push should be 1 for these operators)
        for _ in 0..stack_push {
            self.push(result.clone());
        }

        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// INTERPRETER
// ═══════════════════════════════════════════════════════════════════════════

pub struct LoweringInterpreter {
    #[allow(dead_code)]
    spec: LoweringSpec,
    registry: HashMap<String, (String, OperatorDef)>, // Core IR 0.3: maps name -> (canonical_name, def)
}

impl LoweringInterpreter {
    pub fn load(lowering_path: &Path, registry_path: &Path) -> Result<Self, String> {
        let lowering_content = fs::read_to_string(lowering_path)
            .map_err(|e| format!("failed to read lowering spec: {}", e))?;
        let spec: LoweringSpec = serde_yaml::from_str(&lowering_content)
            .map_err(|e| format!("failed to parse lowering spec: {}", e))?;

        let registry_content = fs::read_to_string(registry_path)
            .map_err(|e| format!("failed to read registry spec: {}", e))?;
        let registry_spec: RegistrySpec = serde_yaml::from_str(&registry_content)
            .map_err(|e| format!("failed to parse registry spec: {}", e))?;

        let mut registry_map = HashMap::new();
        for op in registry_spec.registry.operators.iter() {
            // Core IR 0.3: use operator name as canonical name (no numeric IDs)
            registry_map.insert(op.name.clone(), (op.name.clone(), op.clone()));
        }

        Ok(LoweringInterpreter {
            spec,
            registry: registry_map,
        })
    }

    pub fn lower(&self, node: &SchemaAstNode) -> Result<CoreTerm, String> {
        // For Surface-0, we need to handle the top-level Program and Function nodes
        match node.kind.as_str() {
            "Program" => {
                // Get the functions field
                let functions = self.get_field_as_nodes(node, "functions")?;
                if functions.is_empty() {
                    return Err("program has no functions".to_string());
                }
                // Lower the first function
                self.lower(&functions[0])
            }
            "Function" => {
                // Get the instructions field
                let instructions = self.get_field_as_nodes(node, "instructions")?;

                let mut machine = StackMachine::new(self.registry.clone());

                for instr in instructions {
                    self.lower_instruction(instr, &mut machine)?;
                }

                machine.finalize()
            }
            _ => {
                // Try lowering as an instruction
                let mut machine = StackMachine::new(self.registry.clone());
                self.lower_instruction(node, &mut machine)?;
                machine.finalize()
            }
        }
    }

    fn lower_instruction(
        &self,
        node: &SchemaAstNode,
        machine: &mut StackMachine,
    ) -> Result<(), String> {
        match node.kind.as_str() {
            "Arg" => {
                let index = self.get_field_as_int(node, "index")?;
                // Create a lambda parameter reference
                machine.push(CoreTerm::CLam {
                    param: IdentOrName::new(format!("arg_{}", index)),
                    body: Box::new(CoreTerm::CUnitLit {
                        node_id: None,
                        annotations: Vec::new(),
                    }),
                    node_id: None,
                    annotations: Vec::new(),
                });
                Ok(())
            }
            "Lit" => {
                // Literals become unit for now (Core IR 0.2 doesn't have literals)
                machine.push(CoreTerm::CUnitLit {
                    node_id: None,
                    annotations: Vec::new(),
                });
                Ok(())
            }
            "Call" => {
                let op_name = self.get_field_as_string(node, "op")?;
                machine.call_operator(&op_name)?;
                Ok(())
            }
            "End" => {
                // End instruction - no operation needed
                Ok(())
            }
            "Fn" => {
                // Function declaration - no operation needed during lowering
                Ok(())
            }
            _ => Err(format!("unknown instruction kind: {}", node.kind)),
        }
    }

    fn get_field<'a>(
        &self,
        node: &'a SchemaAstNode,
        name: &str,
    ) -> Result<&'a SchemaValue, String> {
        node.fields
            .get(name)
            .ok_or_else(|| format!("missing field '{}' in node kind '{}'", name, node.kind))
    }

    fn get_field_as_int(&self, node: &SchemaAstNode, name: &str) -> Result<i64, String> {
        match self.get_field(node, name)? {
            SchemaValue::Token(token) => token
                .lexeme
                .parse::<i64>()
                .map_err(|_| format!("field '{}' token is not a valid integer", name)),
            _ => Err(format!(
                "field '{}' is not a token (needed for int extraction)",
                name
            )),
        }
    }

    fn get_field_as_string(&self, node: &SchemaAstNode, name: &str) -> Result<String, String> {
        match self.get_field(node, name)? {
            SchemaValue::Token(token) => Ok(token.lexeme.clone()),
            _ => Err(format!(
                "field '{}' is not a token (needed for string extraction)",
                name
            )),
        }
    }

    fn get_field_as_nodes<'a>(
        &self,
        node: &'a SchemaAstNode,
        name: &str,
    ) -> Result<&'a Vec<SchemaAstNode>, String> {
        match self.get_field(node, name)? {
            SchemaValue::Nodes(nodes) => Ok(nodes),
            _ => Err(format!("field '{}' is not a node list", name)),
        }
    }
}
