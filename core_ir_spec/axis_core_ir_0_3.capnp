@0xb3c9d2a4f18e6c71;

# ============================================================================
# Axis Core IR 0.3 — Canonical Binary Schema
#
# Status: CANONICAL for Core IR version 0.3
#
# This schema defines the authoritative serialized representation of Axis
# Core IR 0.3. All consumers MUST conform.
# ============================================================================

# ----------------------------------------------------------------------------
# CHANGE BLOCK — Core IR 0.2 → 0.3 (Normative)
# ----------------------------------------------------------------------------
#
# Core IR 0.3 introduces the following intentional, breaking changes:
#
# 1. Uniform Function Model
#    - Core IR no longer distinguishes "foreign", "built-in", "platform",
#      or "user" functions.
#    - A function is a function is a function.
#    - If a call target is registry-valid at emission time, it is valid.
#
# 2. Canonical Name as Call Target
#    - CCall targets are identified by a canonical function name (Text).
#    - Numeric identifiers are removed from the call ABI.
#
# 3. Library-First Linking Model
#    - Bridges are expected to compile Core IR bundles into native libraries.
#    - Libraries are linked against other libraries (including shims).
#    - All symbol resolution is delegated to the linker.
#
# 4. No Registry or Semantic Leakage
#    - Consumers MUST NOT load registries.
#    - Consumers MUST treat call targets as opaque ABI symbols.
#
# Core IR 0.3 preserves:
# - the node set
# - lambda-calculus semantics
# - ANF-style structure
# - node identity
# - annotation non-authority
#
# ----------------------------------------------------------------------------


# ============================================================================
# Core Bundle
# ============================================================================

struct CoreBundle {
  version        @0 :Text;        # MUST be "0.3"
  coreTerm       @1 :CoreTerm;

  # Optional, non-authoritative fields
  entrypointName @2 :Text;
  entrypointId   @3 :UInt64;

  annotations    @4 :List(Annotation);
}


# ============================================================================
# Core Terms (Authoritative Semantic Nodes)
# ============================================================================

struct CoreTerm {
  nodeId @0 :UInt64;
  span   @1 :Span;

  union {
    cIntLit  @2  :CIntLit;
    cBoolLit @3  :CBoolLit;
    cUnitLit @4  :CUnitLit;
    cLam     @5  :CLam;
    cLet     @6  :CLet;
    cIf      @7  :CIf;
    cVar     @8  :CVar;
    cApp     @9  :CApp;
    cCall    @10 :CCall;
  }
}


# ============================================================================
# Literal Nodes
# ============================================================================

struct CIntLit {
  value @0 :Int64;
}

struct CBoolLit {
  value @0 :Bool;
}

struct CUnitLit {
}


# ============================================================================
# Binding and Control Nodes
# ============================================================================

struct CLam {
  param @0 :Text;
  body  @1 :CoreTerm;
}

struct CLet {
  name  @0 :Text;
  value @1 :CoreTerm;
  body  @2 :CoreTerm;
}

struct CIf {
  cond @0 :CoreTerm;
  then @1 :CoreTerm;
  else @2 :CoreTerm;
}

struct CVar {
  name @0 :Text;
}

struct CApp {
  fn  @0 :CoreTerm;
  arg @1 :CoreTerm;
}


# ============================================================================
# CCall — Uniform Function Call (Core IR 0.3)
# ============================================================================

struct CCall {
  # Canonical function name.
  #
  # Normative properties:
  # - Globally unique (by producer guarantee)
  # - Stable across builds
  # - Simple string token (no dots, no namespaces encoded)
  #
  # Semantics:
  # - Identifies a registry-valid function.
  # - Carries NO information about origin, implementation, or substrate.
  #
  # Consumers MUST:
  # - Treat this as an opaque ABI symbol name.
  # - Emit a direct call to this symbol.
  # - Perform no registry lookup or semantic interpretation.
  #
  targetName @0 :Text;

  args       @1 :List(CoreTerm);
}


# ============================================================================
# Annotations (Non-Authoritative)
# ============================================================================

struct Annotation {
  id   @0 :Text;
  kind @1 :Text;
  data @2 :Text;
}


# ============================================================================
# Diagnostics
# ============================================================================

struct Span {
  file  @0 :Text;
  start @1 :UInt32;
  end   @2 :UInt32;
}
