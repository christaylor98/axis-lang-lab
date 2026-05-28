# DEAD LEGACY CODE — DO NOT USE

This directory (`src/normalisation/`) contains LEGACY normalization code that is NO LONGER USED by the Axis pipeline.

## Status: QUARANTINED

This code has been superseded by the YAML-driven normalization engine in `src/normalize/`.

## Why This Exists

Historical artifact from earlier pipeline iterations. Kept for reference only.

## Pipeline Reality (Authoritative)

The REAL normalization path is:

```
SchemaAstNode → src/normalize/NormalizationContext → NfAst
```

Using YAML rules from:
- `*/*-normalize.yaml` (per surface)

## What Happens If You Import This

**DO NOT IMPORT.** The pipeline uses `src/normalize` exclusively.

If you see imports from `crate::normalisation`, they are LEGACY and should be removed/replaced.

## Removal Plan

This directory will be deleted once all references are confirmed removed and tests pass.

---

**BOTTOM LINE:**  
src/normalisation/ = DEAD  
src/normalize/ = REAL
