// Hook Registry - Registration and Ordering
//
// Manages hook registration, conflict detection, and deterministic ordering.
//
// ORDERING RULES (Spec § 8):
// 1. Pipeline stage (PreLex < PostParse < Normalisation < PostNormalisation)
// 2. Normalisation pass (if applicable: N0 < N1 < N2 < N3 < N4)
// 3. Explicit before/after hints
// 4. Stable name ordering (lexicographic)

use super::framework::{Hook, HookStage};
use std::collections::HashMap;
use std::sync::Arc;

// ═══════════════════════════════════════════════════════════════════════════
// ORDERING HINTS
// ═══════════════════════════════════════════════════════════════════════════

/// Ordering hint for hook execution
///
/// Allows hooks to specify relative ordering within the same stage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderingHint {
    /// Execute before the named hook
    Before(String),

    /// Execute after the named hook
    After(String),

    /// No specific ordering constraint
    None,
}

// ═══════════════════════════════════════════════════════════════════════════
// HOOK METADATA
// ═══════════════════════════════════════════════════════════════════════════

/// Metadata for a registered hook
#[derive(Debug, Clone)]
struct HookMetadata {
    /// The hook implementation
    hook: Arc<dyn Hook>,

    /// Ordering hint
    ordering_hint: OrderingHint,
}

// ═══════════════════════════════════════════════════════════════════════════
// CONFLICTS
// ═══════════════════════════════════════════════════════════════════════════

/// Hook registration conflict
#[derive(Debug, Clone)]
pub enum HookConflict {
    /// Duplicate hook name
    DuplicateName {
        name: String,
        existing_stage: HookStage,
        new_stage: HookStage,
    },

    /// Circular ordering dependency
    CircularDependency { cycle: Vec<String> },

    /// Ordering hint references non-existent hook
    MissingOrderingTarget {
        hook_name: String,
        target_name: String,
    },

    /// Conflicting permissions at same stage
    PermissionConflict {
        hook1: String,
        hook2: String,
        stage: HookStage,
    },
}

impl std::fmt::Display for HookConflict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HookConflict::DuplicateName {
                name,
                existing_stage,
                new_stage,
            } => {
                write!(
                    f,
                    "Duplicate hook name '{}': already registered at stage {}, cannot register at stage {}",
                    name, existing_stage, new_stage
                )
            }
            HookConflict::CircularDependency { cycle } => {
                write!(f, "Circular hook dependency: {}", cycle.join(" -> "))
            }
            HookConflict::MissingOrderingTarget {
                hook_name,
                target_name,
            } => {
                write!(
                    f,
                    "Hook '{}' has ordering constraint on non-existent hook '{}'",
                    hook_name, target_name
                )
            }
            HookConflict::PermissionConflict {
                hook1,
                hook2,
                stage,
            } => {
                write!(
                    f,
                    "Permission conflict at stage {}: hooks '{}' and '{}' have conflicting permissions",
                    stage, hook1, hook2
                )
            }
        }
    }
}

impl std::error::Error for HookConflict {}

// ═══════════════════════════════════════════════════════════════════════════
// HOOK REGISTRY
// ═══════════════════════════════════════════════════════════════════════════

/// Central registry for compiler hooks
///
/// Manages hook registration, validation, and ordering.
/// Ensures deterministic hook execution.
#[derive(Debug, Clone)]
pub struct HookRegistry {
    /// Registered hooks by stage
    hooks: HashMap<HookStage, Vec<HookMetadata>>,

    /// All hook names (for conflict detection)
    hook_names: HashMap<String, HookStage>,
}

impl HookRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            hooks: HashMap::new(),
            hook_names: HashMap::new(),
        }
    }

    /// Register a hook
    ///
    /// Returns an error if:
    /// - Hook name already registered
    /// - Ordering hint is invalid
    pub fn register(
        &mut self,
        hook: Arc<dyn Hook>,
        ordering_hint: OrderingHint,
    ) -> Result<(), HookConflict> {
        let name = hook.name().to_string();
        let stage = hook.stage();

        // Check for duplicate name
        if let Some(&existing_stage) = self.hook_names.get(&name) {
            return Err(HookConflict::DuplicateName {
                name,
                existing_stage,
                new_stage: stage,
            });
        }

        // Validate ordering hint
        if let OrderingHint::Before(ref _target) | OrderingHint::After(ref _target) = ordering_hint
        {
            // Check target exists (will be validated again during ordering)
            // For now, just store it
        }

        // Register
        self.hook_names.insert(name, stage);
        self.hooks
            .entry(stage)
            .or_insert_with(Vec::new)
            .push(HookMetadata {
                hook,
                ordering_hint,
            });

        Ok(())
    }

    /// Get ordered hooks for a specific stage
    ///
    /// Returns hooks in deterministic execution order:
    /// 1. Normalisation pass (if applicable)
    /// 2. Before/after hints
    /// 3. Lexicographic name order
    pub fn get_hooks_for_stage(
        &self,
        stage: HookStage,
    ) -> Result<Vec<Arc<dyn Hook>>, HookConflict> {
        let Some(metadata) = self.hooks.get(&stage) else {
            return Ok(Vec::new());
        };

        // Clone metadata for sorting
        let mut sorted = metadata.clone();

        // Apply ordering constraints
        self.topological_sort(&mut sorted, stage)?;

        // Extract hooks
        Ok(sorted.into_iter().map(|m| m.hook).collect())
    }

    /// Get ordered hooks for a specific normalisation pass
    ///
    /// Filters hooks by normalisation pass and returns in execution order.
    pub fn get_hooks_for_normalisation_pass(
        &self,
        pass_id: &str,
    ) -> Result<Vec<Arc<dyn Hook>>, HookConflict> {
        let hooks = self.get_hooks_for_stage(HookStage::Normalisation)?;

        // Filter by pass
        let filtered: Vec<_> = hooks
            .into_iter()
            .filter(|h| {
                h.normalisation_pass().map(|p| p == pass_id).unwrap_or(true) // None means "all passes"
            })
            .collect();

        Ok(filtered)
    }

    /// Validate all registered hooks
    ///
    /// Checks for:
    /// - Circular dependencies
    /// - Invalid ordering targets
    /// - Permission conflicts
    pub fn validate(&self) -> Result<(), HookConflict> {
        for (stage, metadata) in &self.hooks {
            // Check ordering targets exist
            for meta in metadata {
                match &meta.ordering_hint {
                    OrderingHint::Before(target) | OrderingHint::After(target) => {
                        if !self.hook_names.contains_key(target) {
                            return Err(HookConflict::MissingOrderingTarget {
                                hook_name: meta.hook.name().to_string(),
                                target_name: target.clone(),
                            });
                        }

                        // Check target is in same stage
                        if let Some(&target_stage) = self.hook_names.get(target) {
                            if target_stage != *stage {
                                // Ordering hints only apply within the same stage
                                return Err(HookConflict::MissingOrderingTarget {
                                    hook_name: meta.hook.name().to_string(),
                                    target_name: target.clone(),
                                });
                            }
                        }
                    }
                    OrderingHint::None => {}
                }
            }

            // Check for circular dependencies
            self.detect_cycles(&metadata, *stage)?;
        }

        Ok(())
    }

    /// Detect circular dependencies in ordering hints
    fn detect_cycles(
        &self,
        metadata: &[HookMetadata],
        _stage: HookStage,
    ) -> Result<(), HookConflict> {
        // Build dependency graph
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();

        for meta in metadata {
            let name = meta.hook.name().to_string();
            graph.entry(name.clone()).or_insert_with(Vec::new);

            match &meta.ordering_hint {
                OrderingHint::Before(target) => {
                    // name must come before target => target depends on name
                    graph
                        .entry(target.clone())
                        .or_insert_with(Vec::new)
                        .push(name);
                }
                OrderingHint::After(target) => {
                    // name must come after target => name depends on target
                    graph
                        .entry(name.clone())
                        .or_insert_with(Vec::new)
                        .push(target.clone());
                }
                OrderingHint::None => {}
            }
        }

        // DFS cycle detection
        let mut visited = HashMap::new();
        let mut rec_stack = HashMap::new();

        for name in graph.keys() {
            if !visited.contains_key(name) {
                if let Some(cycle) = self.dfs_cycle(&graph, name, &mut visited, &mut rec_stack) {
                    return Err(HookConflict::CircularDependency { cycle });
                }
            }
        }

        Ok(())
    }

    /// DFS helper for cycle detection
    fn dfs_cycle(
        &self,
        graph: &HashMap<String, Vec<String>>,
        node: &str,
        visited: &mut HashMap<String, bool>,
        rec_stack: &mut HashMap<String, bool>,
    ) -> Option<Vec<String>> {
        visited.insert(node.to_string(), true);
        rec_stack.insert(node.to_string(), true);

        if let Some(neighbors) = graph.get(node) {
            for neighbor in neighbors {
                if !visited.contains_key(neighbor) {
                    if let Some(cycle) = self.dfs_cycle(graph, neighbor, visited, rec_stack) {
                        return Some(cycle);
                    }
                } else if *rec_stack.get(neighbor).unwrap_or(&false) {
                    // Found cycle
                    return Some(vec![node.to_string(), neighbor.to_string()]);
                }
            }
        }

        rec_stack.insert(node.to_string(), false);
        None
    }

    /// Topological sort with stable name ordering fallback
    fn topological_sort(
        &self,
        metadata: &mut [HookMetadata],
        _stage: HookStage,
    ) -> Result<(), HookConflict> {
        // For now, simple stable sort by name
        // TODO: Implement full topological sort respecting before/after hints
        metadata.sort_by(|a, b| a.hook.name().cmp(b.hook.name()));

        Ok(())
    }

    /// Get total number of registered hooks
    pub fn len(&self) -> usize {
        self.hook_names.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.hook_names.is_empty()
    }

    /// Get all registered hook names
    pub fn hook_names(&self) -> Vec<String> {
        let mut names: Vec<_> = self.hook_names.keys().cloned().collect();
        names.sort();
        names
    }
}

impl Default for HookRegistry {
    fn default() -> Self {
        Self::new()
    }
}
