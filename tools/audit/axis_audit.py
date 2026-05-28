#!/usr/bin/env python3
"""
axis_audit.py — Load-bearing dataflow checker for Axis Core IR 0.3 bundles.

This is the decidable auditability check: the thing arbitrary host code
(Python/JS emitted by a script-writing agent) cannot offer, because the analysis
below is undecidable on a Turing-complete substrate and tractable on the closed
9-node IR.

TWO JOBS
  1. Structural validation: closed node set, exactly one variant per term,
     unique nodeIds. (The hard-limits the emission battery surfaced.)
  2. The load-bearing audit: for each generative / intelligence cCall, decide --
     by mechanical taint propagation over the 9 nodes -- whether its output
     reaches a CONSEQUENTIAL position:
        (a) a branch condition  (cIf.cond), or
        (b) an argument to a consequential-effect call.
     If yes  -> LOAD-BEARING  (the model is on the critical path; flag/gate/log).
     If only benign sinks -> SIDE-EFFECT (provably detachable / downgradable to
     a mechanical substitute without changing what the workflow *does*).

HONEST DESIGN NOTE
  The classification of which call names are 'intelligence' sources and which are
  'consequential effects' is an explicit INPUT, not a heuristic. The audit is
  therefore itself auditable -- the verdict is a function of a stated policy plus
  a mechanical propagation. The tool's contribution is the propagation.

KNOWN BOUNDARY
  First-order dataflow. Higher-order taint capture (a tainted value closed over
  in a cLam and applied elsewhere) is conservatively under-tracked; the agentic
  workflows this targets are first-order pipelines. Documented, not hidden.
"""

import sys
import json

sys.setrecursionlimit(10000)

NODE_KINDS = {
    "cIntLit", "cBoolLit", "cUnitLit", "cLam", "cLet",
    "cIf", "cVar", "cApp", "cCall",
}


# --------------------------------------------------------------------------
# Structural validation
# --------------------------------------------------------------------------

class ValidationError(Exception):
    pass


def variant_of(term):
    """Return the single variant key present on a term, or raise."""
    present = [k for k in term.keys() if k in NODE_KINDS]
    if len(present) == 0:
        raise ValidationError(f"node {term.get('nodeId')} has no known variant")
    if len(present) > 1:
        raise ValidationError(
            f"node {term.get('nodeId')} has multiple variants: {present}")
    return present[0]


def validate(term, seen_ids):
    """Recursively validate structure; collect nodeIds to check uniqueness."""
    if not isinstance(term, dict):
        raise ValidationError(f"term is not an object: {term!r}")
    nid = term.get("nodeId")
    if nid is None:
        raise ValidationError(f"term missing nodeId: {term!r}")
    if nid in seen_ids:
        raise ValidationError(f"duplicate nodeId: {nid}")
    seen_ids.add(nid)

    v = variant_of(term)
    body = term[v]
    if v in ("cIntLit", "cBoolLit", "cUnitLit", "cVar"):
        pass
    elif v == "cLam":
        validate(body["body"], seen_ids)
    elif v == "cLet":
        validate(body["value"], seen_ids)
        validate(body["body"], seen_ids)
    elif v == "cIf":
        validate(body["cond"], seen_ids)
        validate(body["then"], seen_ids)
        validate(body["else"], seen_ids)
    elif v == "cApp":
        validate(body["fn"], seen_ids)
        validate(body["arg"], seen_ids)
    elif v == "cCall":
        # call-name token rule: single bare token, no dots/namespaces
        name = body["targetName"]
        if "." in name or not name or any(c.isspace() for c in name):
            raise ValidationError(
                f"node {nid}: illegal call target '{name}' "
                f"(must be a single bare token, no dots)")
        for a in body["args"]:
            validate(a, seen_ids)
    return True


# --------------------------------------------------------------------------
# Taint propagation (the load-bearing audit)
# --------------------------------------------------------------------------

class Findings:
    def __init__(self):
        self.decisions = []   # (label, cif_nodeId)
        self.effects = []     # (label, call_nodeId, targetName)
        self.effect_guards = {}  # call_nodeId -> list of enclosing cIf nodeIds
        self.sources = {}     # label -> source nodeId

    def record_decision(self, label, cif_nid):
        self.decisions.append((label, cif_nid))

    def record_effect(self, label, call_nid, name):
        self.effects.append((label, call_nid, name))


def analyze(term, env, intel, conseq, findings, guards):
    """
    Return the taint set (set of source labels) carried by this term's *result*.
    Records findings as side effects:
      - taint reaching a cIf.cond           -> a decision use
      - taint reaching a consequential-call  -> a consequential-effect use
      - guard context for every consequential call (gating cIf nodeIds)
    `env`: name -> taint set.  `guards`: list of enclosing cIf nodeIds.
    """
    v = variant_of(term)
    body = term[v]
    nid = term["nodeId"]

    if v in ("cIntLit", "cBoolLit", "cUnitLit"):
        return set()

    if v == "cVar":
        return set(env.get(body["name"], set()))

    if v == "cLam":
        # param is external input: no taint. Body findings still recorded.
        env2 = dict(env)
        env2[body["param"]] = set()
        analyze(body["body"], env2, intel, conseq, findings, guards)
        return set()  # closure value (first-order boundary)

    if v == "cLet":
        t_value = analyze(body["value"], env, intel, conseq, findings, guards)
        env2 = dict(env)
        env2[body["name"]] = t_value
        return analyze(body["body"], env2, intel, conseq, findings, guards)

    if v == "cIf":
        t_cond = analyze(body["cond"], env, intel, conseq, findings, guards)
        for label in t_cond:
            findings.record_decision(label, nid)            # taint drives a branch
        guards2 = guards + [nid]
        t_then = analyze(body["then"], env, intel, conseq, findings, guards2)
        t_else = analyze(body["else"], env, intel, conseq, findings, guards2)
        return t_cond | t_then | t_else                     # conservative

    if v == "cApp":
        t_fn = analyze(body["fn"], env, intel, conseq, findings, guards)
        t_arg = analyze(body["arg"], env, intel, conseq, findings, guards)
        return t_fn | t_arg                                 # conservative

    if v == "cCall":
        name = body["targetName"]
        arg_taint = set()
        for a in body["args"]:
            arg_taint |= analyze(a, env, intel, conseq, findings, guards)

        if name in conseq:
            findings.effect_guards[nid] = list(guards)
            for label in arg_taint:
                findings.record_effect(label, nid, name)    # taint into real effect

        if name in intel:
            label = f"{name}#{nid}"
            findings.sources[label] = nid
            return {label} | arg_taint                       # source emits its taint

        return arg_taint                                     # mechanical pass-through

    raise ValidationError(f"unhandled variant {v}")


def audit(bundle, intel, conseq):
    """Run validation + taint audit. Returns a report dict."""
    term = bundle["coreTerm"]
    seen = set()
    validate(term, seen)

    f = Findings()
    analyze(term, {}, set(intel), set(conseq), f, [])

    # per-source verdict
    load_bearing = {}
    for label in f.sources:
        reasons = []
        for (lab, cif) in f.decisions:
            if lab == label:
                reasons.append(("decision", cif))
        for (lab, cnid, nm) in f.effects:
            if lab == label:
                reasons.append(("effect", cnid, nm))
        load_bearing[label] = reasons

    return {
        "entrypoint": bundle.get("entrypointName"),
        "node_count": len(seen),
        "sources": f.sources,
        "verdicts": load_bearing,
        "consequential_effects": f.effect_guards,
        "decisions": f.decisions,
    }


def report(bundle, intel, conseq, title):
    print("=" * 72)
    print(f"AUDIT: {title}")
    print("=" * 72)
    try:
        r = audit(bundle, intel, conseq)
    except ValidationError as e:
        print(f"  STRUCTURAL FAILURE: {e}")
        print()
        return
    print(f"  entrypoint: {r['entrypoint']}   ({r['node_count']} nodes, structure OK)")
    print(f"  policy: intelligence={sorted(intel)}  consequential={sorted(conseq)}")
    print()

    # consequential effects and their gating
    if r["consequential_effects"]:
        print("  consequential effects:")
        for cnid, gds in r["consequential_effects"].items():
            gate = (f"gated by cIf node(s) {gds}" if gds
                    else "UNGATED (runs unconditionally)")
            print(f"    - node {cnid}: {gate}")
    else:
        print("  consequential effects: none declared")
    print()

    # the verdict
    if not r["sources"]:
        print("  generative/intelligence calls: NONE")
        print("  VERDICT: consequential path is fully mechanical "
              "(no model on the critical path).")
        print()
        return

    print("  generative/intelligence calls:")
    for label, reasons in r["verdicts"].items():
        nid = r["sources"][label]
        if reasons:
            print(f"    - {label} (node {nid}): *** LOAD-BEARING ***")
            for reason in reasons:
                if reason[0] == "decision":
                    print(f"        -> output reaches branch condition at cIf node {reason[1]}")
                else:
                    print(f"        -> output flows into consequential effect "
                          f"'{reason[2]}' at node {reason[1]}")
        else:
            print(f"    - {label} (node {nid}): side-effect "
                  f"(reaches only benign sinks; provably detachable / downgradable)")
    print()


# --------------------------------------------------------------------------
# Test fixtures
# --------------------------------------------------------------------------
# Two are REAL bundles emitted by the LLM earlier in the session.
# Two are hand-authored to exercise the positive-detection (LOAD-BEARING) path,
# which a test suite needs just as much as the negative path.

def L(nid, **kw):  # tiny helper to keep fixtures readable
    return {"nodeId": nid, **kw}


CLEAN_TMP = {  # REAL emission
    "version": "0.3", "entrypointName": "clean_tmp",
    "coreTerm": L(12, cLam={"param": "_unit", "body":
        L(11, cLet={"name": "usage",
            "value": L(4, cCall={"targetName": "get_tmp_disk_usage", "args": []}),
            "body": L(10, cLet={"name": "cond",
                "value": L(6, cCall={"targetName": "int_gt", "args": [
                    L(5, cVar={"name": "usage"}), L(1, cIntLit={"value": 90})]}),
                "body": L(9, cIf={
                    "cond": L(7, cVar={"name": "cond"}),
                    "then": L(8, cCall={"targetName": "remove_tmp_files_older_than",
                                        "args": [L(2, cIntLit={"value": 7})]}),
                    "else": L(3, cUnitLit={})})})})}),
}

DESCRIBE_ENTRY = {  # REAL emission (the forced-decomposition result)
    "version": "0.3", "entrypointName": "describe_entry",
    "coreTerm": L(1, cLam={"param": "worksheet_ref", "body":
        L(2, cLet={"name": "raw_content",
          "value": L(3, cCall={"targetName": "read_worksheet",
                               "args": [L(4, cVar={"name": "worksheet_ref"})]}),
          "body": L(5, cLet={"name": "entry_ref",
            "value": L(6, cCall={"targetName": "get_entry",
                                 "args": [L(7, cVar={"name": "worksheet_ref"})]}),
            "body": L(8, cLet={"name": "cells",
              "value": L(9, cCall={"targetName": "extract_cells",
                                   "args": [L(10, cVar={"name": "raw_content"})]}),
              "body": L(57, cLet={"name": "context",
                "value": L(58, cCall={"targetName": "bundle_context", "args": [
                    L(59, cVar={"name": "raw_content"}),
                    L(66, cVar={"name": "cells"})]}),
                "body": L(67, cLet={"name": "analysis",
                  "value": L(68, cCall={"targetName": "synthesize_analysis",
                                        "args": [L(69, cVar={"name": "context"})]}),
                  "body": L(70, cCall={"targetName": "set_description_field", "args": [
                      L(71, cVar={"name": "entry_ref"}),
                      L(72, cVar={"name": "analysis"})]})})})})})})}),
}

# CONSTRUCTED: intelligence output drives a deploy decision (must be caught).
TRIAGE_DEPLOY = {
    "version": "0.3", "entrypointName": "triage",
    "coreTerm": L(1, cLam={"param": "incident", "body":
        L(2, cLet={"name": "sev",
            "value": L(3, cCall={"targetName": "classify_severity",
                                 "args": [L(4, cVar={"name": "incident"})]}),
            "body": L(5, cIf={
                "cond": L(6, cCall={"targetName": "int_gt", "args": [
                    L(7, cVar={"name": "sev"}), L(8, cIntLit={"value": 7})]}),
                "then": L(9, cCall={"targetName": "deploy_rollback",
                                    "args": [L(10, cVar={"name": "incident"})]}),
                "else": L(11, cUnitLit={})})})}),
}

# CONSTRUCTED: intelligence output LAUNDERED through two mechanical calls before
# reaching a decision that gates a real effect. Soundness test: taint must survive
# indirection (this is the case a naive checker -- or arbitrary Python -- misses).
LAUNDERING = {
    "version": "0.3", "entrypointName": "screen",
    "coreTerm": L(1, cLam={"param": "req", "body":
        L(2, cLet={"name": "raw",
          "value": L(3, cCall={"targetName": "analyze_request",
                               "args": [L(4, cVar={"name": "req"})]}),
          "body": L(5, cLet={"name": "norm",
            "value": L(6, cCall={"targetName": "normalize",
                                 "args": [L(7, cVar={"name": "raw"})]}),
            "body": L(8, cIf={
                "cond": L(9, cCall={"targetName": "is_high_risk",
                                    "args": [L(10, cVar={"name": "norm"})]}),
                "then": L(11, cCall={"targetName": "block_transaction",
                                     "args": [L(12, cVar={"name": "req"})]}),
                "else": L(13, cUnitLit={})})})})}),
}


if __name__ == "__main__":
    report(CLEAN_TMP, intel=set(), conseq={"remove_tmp_files_older_than"},
           title="clean_tmp  [REAL emission] - mechanical disk cleanup")

    report(DESCRIBE_ENTRY, intel={"synthesize_analysis"}, conseq=set(),
           title="describe_entry  [REAL emission] - AI analysis -> description field")

    report(TRIAGE_DEPLOY, intel={"classify_severity"}, conseq={"deploy_rollback"},
           title="triage  [constructed] - AI severity drives a deploy decision")

    report(LAUNDERING, intel={"analyze_request"}, conseq={"block_transaction"},
           title="screen  [constructed] - AI verdict LAUNDERED through 2 calls into a block")

    # Demonstrate the policy-transparency point: re-audit describe_entry but now
    # treat the description write as consequential. The verdict flips -- showing
    # the verdict is a function of the explicit, auditable policy.
    report(DESCRIBE_ENTRY, intel={"synthesize_analysis"},
           conseq={"set_description_field"},
           title="describe_entry  [same bundle, stricter policy] - description write = consequential")
