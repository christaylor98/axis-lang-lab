# Axis Language Lab — Phase 7 Freeze Note

**Phase:** 7
**Title:** Configuration and Policy
**Status:** FROZEN

---

## Purpose of Phase 7

Phase 7 establishes **configuration as policy**, not semantics.

Configuration governs:

* admission
* validation
* tooling behavior

Configuration does **not** define or alter program meaning.

---

## What Is Frozen

Phase 7 freezes the following principles as authoritative:

1. **Configuration is non-semantic**

   * Config must not change Core IR meaning
   * Config must not affect lowering outcomes
   * Config must not affect runtime execution semantics

2. **Configuration is policy-only**

   * Profile selection
   * Registry admission and ordering
   * Validation strictness
   * Tooling enablement/disablement

3. **Semantic authority remains unchanged**

   * Lowering is the sole semantic authority
   * Core IR remains configuration-free
   * Execution consumes Core IR as-is

4. **Configuration is evaluated pre-execution**

   * Config is applied before lowering or execution
   * Runtime does not interpret configuration

---

## Explicit Non-Goals

Phase 7 explicitly does **not** include:

* semantic feature toggles
* conditional lowering behavior
* runtime policy enforcement
* dynamic configuration changes
* execution-time semantics switching

Any such behavior requires a new phase.

---

## Architectural Consequence

With Phase 7 frozen:

* Configuration may restrict *whether* something runs
* Configuration may restrict *which tools* are active
* Configuration may not alter *what a program means*

This separation is mandatory.

---

## Completion Statement

Phase 7 is complete and frozen.

Configuration is a policy mechanism only and must not be used to introduce or encode semantics.

---

**End of Phase 7 Freeze Note**

