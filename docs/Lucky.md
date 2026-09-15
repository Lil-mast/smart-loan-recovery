# Lucky

## 1) Did we think of using an LLM for recovery?
Not in the current codebase.

- The “AI recovery” logic implemented here is **rule-based heuristics**, not an LLM: it computes a **risk score** from loan status (and a small interest-rate bump) and then applies **fixed thresholds** to pick an action.
- There is no pipeline that sends loan details, repayment history, or missed payments to an LLM to generate recommendations.

## 2) What an LLM approach would require (conceptually)
If you were to use an LLM, you’d typically:
- Provide structured inputs: loan terms + borrower context + repayment history (missed payments, delinquency days, amounts paid, prior contact outcomes, etc.).
- Ask the model to output:
  - recommended next action (remind/renegotiate/escalate)
  - a short rationale
  - optionally a message template/timing suggestion
- (Ideally) validate outputs against business rules and compliance constraints.

## 3) Using AWS AI (Nova / Bedrock) for “AI recovery” (recommended design)
Replace the current heuristic step with a Bedrock call:

1. **Collect features**
   - loan data: principal, interest_rate, term, current status, outstanding amount
   - history: missed payments count, last repayment date, delinquency duration, repayment amounts
   - optional signals: borrower profile/category, prior interventions

2. **Prompt/model contract (structured output)**
   - Provide a strict schema in the prompt (JSON-only) for:
     - `recommended_action`: enum
     - `risk_score`: 0-1 (or 0-100)
     - `key_reasons`: short bullets
     - `next_step_plan`: timeline of actions

3. **Guardrails**
   - Enforce allowed actions (map/validate enum)
   - Cap risk-score range
   - Log prompts/outputs for auditability
   - Keep compliance messaging separate from legal/collection language

## 4) Brief conclusion
- Current project: **heuristics**, not LLM.
- Target improvement: **call AWS Bedrock (Nova) with structured loan + repayment history** and use its structured recommendation as the new recovery engine, with guardrails and validation.
