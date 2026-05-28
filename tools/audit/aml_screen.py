#!/usr/bin/env python3
"""
aml_screen.py — A REAL repeatable process (AML transaction screening) lowered to
Core IR 0.3, in two architectural variants that mirror how it actually gets
deployed, run through the load-bearing checker.

Process (from published transaction-monitoring descriptions):
  - sanctions screening is a hard gate at point of payment -> match = block
  - behavioural rules (amount threshold, velocity) flag for review
  - an ML risk score increasingly feeds the decision
  - tiered disposition: hard hits auto-act; softer signals queue for analyst

The two variants differ in ONE wiring choice -- whether the ML score gates the
HOLD directly, or only annotates an alert a human dispositions. That single
difference is exactly what the load-bearing audit is built to detect, and it is
the difference a regulator cares about: 'is the model deciding, or assisting?'
"""

import sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from axis_audit import report  # reuse the checker built earlier


def L(nid, **kw):
    return {"nodeId": nid, **kw}


# ---------------------------------------------------------------------------
# VARIANT 1 — "AI in the decision loop"
# The ML risk score feeds the condition that gates HOLD on the transaction.
# Structure (entrypoint = txn):
#   let sanctioned = sanctions_match(txn)
#   if sanctioned then block_transaction(txn)
#   else
#     let amt_hi   = amount_over_threshold(txn)
#     let vel_hi   = velocity_spike(txn)
#     let score    = ml_risk_score(txn)            # <-- the model
#     let risky    = any_true(amt_hi, vel_hi, score_over(score))
#     if risky then hold_transaction(txn) else allow_transaction(txn)
# ---------------------------------------------------------------------------

AML_AI_IN_LOOP = {
    "version": "0.3", "entrypointName": "screen_txn",
    "coreTerm": L(1, cLam={"param": "txn", "body":
      L(2, cLet={"name": "sanctioned",
        "value": L(3, cCall={"targetName": "sanctions_match",
                             "args": [L(4, cVar={"name": "txn"})]}),
        "body": L(5, cIf={
          "cond": L(6, cVar={"name": "sanctioned"}),
          "then": L(7, cCall={"targetName": "block_transaction",
                              "args": [L(8, cVar={"name": "txn"})]}),
          "else": L(9, cLet={"name": "amt_hi",
            "value": L(10, cCall={"targetName": "amount_over_threshold",
                                  "args": [L(11, cVar={"name": "txn"})]}),
            "body": L(12, cLet={"name": "vel_hi",
              "value": L(13, cCall={"targetName": "velocity_spike",
                                    "args": [L(14, cVar={"name": "txn"})]}),
              "body": L(15, cLet={"name": "score",
                "value": L(16, cCall={"targetName": "ml_risk_score",
                                      "args": [L(17, cVar={"name": "txn"})]}),
                "body": L(18, cLet={"name": "score_hi",
                  "value": L(19, cCall={"targetName": "score_over",
                      "args": [L(20, cVar={"name": "score"}),
                               L(21, cIntLit={"value": 80})]}),
                  "body": L(22, cLet={"name": "risky",
                    "value": L(23, cCall={"targetName": "any_true", "args": [
                        L(24, cVar={"name": "amt_hi"}),
                        L(25, cVar={"name": "vel_hi"}),
                        L(26, cVar={"name": "score_hi"})]}),
                    "body": L(27, cIf={
                      "cond": L(28, cVar={"name": "risky"}),
                      "then": L(29, cCall={"targetName": "hold_transaction",
                                           "args": [L(30, cVar={"name": "txn"})]}),
                      "else": L(31, cCall={"targetName": "allow_transaction",
                                           "args": [L(32, cVar={"name": "txn"})]})})})})})})})})})}),
}


# ---------------------------------------------------------------------------
# VARIANT 2 — "AI assists, human decides"
# IDENTICAL sanctions + rules spine. The HOLD is gated ONLY by the mechanical
# rules (sanctions / amount / velocity). The ML score is computed and ATTACHED
# to an alert for an analyst -- it never reaches the hold/allow condition.
#   ... (same sanctions gate) ...
#   else
#     let amt_hi = amount_over_threshold(txn)
#     let vel_hi = velocity_spike(txn)
#     let score  = ml_risk_score(txn)              # <-- model, but...
#     let risky  = any_true(amt_hi, vel_hi)        # ...NOT in this condition
#     if risky then hold_transaction(txn)
#              else attach_score_to_alert(txn, score)   # model -> benign sink
# ---------------------------------------------------------------------------

AML_AI_ASSIST = {
    "version": "0.3", "entrypointName": "screen_txn",
    "coreTerm": L(1, cLam={"param": "txn", "body":
      L(2, cLet={"name": "sanctioned",
        "value": L(3, cCall={"targetName": "sanctions_match",
                             "args": [L(4, cVar={"name": "txn"})]}),
        "body": L(5, cIf={
          "cond": L(6, cVar={"name": "sanctioned"}),
          "then": L(7, cCall={"targetName": "block_transaction",
                              "args": [L(8, cVar={"name": "txn"})]}),
          "else": L(9, cLet={"name": "amt_hi",
            "value": L(10, cCall={"targetName": "amount_over_threshold",
                                  "args": [L(11, cVar={"name": "txn"})]}),
            "body": L(12, cLet={"name": "vel_hi",
              "value": L(13, cCall={"targetName": "velocity_spike",
                                    "args": [L(14, cVar={"name": "txn"})]}),
              "body": L(15, cLet={"name": "score",
                "value": L(16, cCall={"targetName": "ml_risk_score",
                                      "args": [L(17, cVar={"name": "txn"})]}),
                "body": L(22, cLet={"name": "risky",
                  "value": L(23, cCall={"targetName": "any_true", "args": [
                      L(24, cVar={"name": "amt_hi"}),
                      L(25, cVar={"name": "vel_hi"})]}),
                  "body": L(27, cIf={
                    "cond": L(28, cVar={"name": "risky"}),
                    "then": L(29, cCall={"targetName": "hold_transaction",
                                         "args": [L(30, cVar={"name": "txn"})]}),
                    "else": L(33, cCall={"targetName": "attach_score_to_alert",
                        "args": [L(34, cVar={"name": "txn"}),
                                 L(35, cVar={"name": "score"})]})})})})})})})})}),
}


CONSEQUENTIAL = {"block_transaction", "hold_transaction"}
# allow_transaction and attach_score_to_alert are NOT consequential:
#   allow = the default non-action; attach_score = writes to an analyst alert.
INTELLIGENCE = {"ml_risk_score"}


if __name__ == "__main__":
    report(AML_AI_IN_LOOP, intel=INTELLIGENCE, conseq=CONSEQUENTIAL,
           title="AML screen — VARIANT 1: ML score gates the HOLD ('AI in the loop')")
    report(AML_AI_ASSIST, intel=INTELLIGENCE, conseq=CONSEQUENTIAL,
           title="AML screen — VARIANT 2: ML score only annotates alert ('AI assists')")
