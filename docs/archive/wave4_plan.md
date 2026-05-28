# Wave 4 Plan — Configuration and Profile Selection

## Status

PLANNED
Not started

This document is authoritative for Wave 4.

---

## Purpose

Wave 4 introduces **configuration as data**, allowing the compiler to select:

* which registries are active
* which profile is enforced

Wave 4 does **not** introduce new language semantics.
It externalises decisions that were previously hardcoded in Wave 3.

---

## Non-Goals (Explicit)

Wave 4 does NOT include:

* runtime execution
* effect inference
* mutation or variables
* concurrency or async
* dynamic registry modification
* hot reloading
* policy evaluation engines

Wave 4 is **selection and gating only**.

---

## Governing Constraints

1. **Configuration never introduces semantics**
2. **Lowering remains the semantic authority**
3. **Registry resolution still happens during lowering**
4. **Core IR remains configuration-free**
5. **Earlier waves remain unchanged**

---

## Configuration Model

### New Artifact

Introduce a configuration file:

```
config/langlab.toml
```

This file is **read once**, before compilation.

---

### Required Configuration Fields

```
active_profile = "default"

registries = [
  "registries/wave3_test.axreg"
]
```

Constraints:

* paths are explicit
* order is significant
* no globbing
* no conditionals

---

## Profile Semantics

Profiles are defined **only** by registry metadata.

Wave 4 adds:

* profile selection
* profile enforcement

Wave 4 does NOT add:

* profile inference
* profile composition
* profile hierarchies

---

### Lowering Impact

Lowering must now:

1. Read `active_profile` from config
2. Enforce profile admission for registry calls
3. Reject calls disallowed by profile

No other lowering rules change.

---

## Registry Loading Rules

Wave 4 allows:

* multiple registries to be loaded
* registry search order defined by config list

Resolution rule:

1. Search registries in listed order
2. First matching function name wins
3. Ambiguity is an error

Registry IDs remain:

* numeric
* assigned by load order across all registries

---

## Core IR Impact

None.

Wave 4 must **not** change:

* Core IR schema
* Cap’n Proto encoding
* Core IR validation rules

This is a hard constraint.

---

## Validation Rules

Validation must enforce:

* all calls resolved under selected profile
* no calls from disallowed profiles
* no unresolved or ambiguous registry references

Validation must NOT:

* interpret configuration beyond selection
* embed config data into Core IR

---

## Golden Tests (Mandatory)

Add goldens for:

1. Same source, different profile → reject
2. Same source, different registry order → different numeric IDs
3. Multiple registries, shadowed function name
4. Explicit failure on ambiguous resolution

Goldens must:

* remain byte-stable
* not affect earlier waves

---

## Execution Strategy

Wave 4 is executed as **one bundled wave**.

Includes:

* config parser
* registry loader extension
* lowering updates
* validation updates
* golden tests

No partial merges.

---

## Acceptance Criteria

Wave 4 is complete when:

* registry and profile selection are config-driven
* lowering enforces profile admission
* Core IR remains unchanged
* earlier waves’ goldens remain unchanged
* all new behavior is covered by goldens

---

## Change Control

Any change that:

* affects Core IR
* adds execution semantics
* adds new language constructs

requires a new wave and explicit plan update.

---

## End of Document
