---
description: Run cross-cluster contract verifier (CC-1..CC-7) via the cc-contract-verifier agent
argument-hint: (no args) — verifies all 7 contracts; or pass a single CC ID (e.g. CC-4) to scope
---

# /cc-verify — cross-cluster contract verifier

Run the `cc-contract-verifier` subagent against all 7 cross-cluster synergy contracts (or a single named contract). Reports per-contract alignment between producer and consumer cluster specs (pre-G9) or source (post-G9).

```bash
#!/usr/bin/env bash
ROOT="/home/louranicas/claude-code-workspace/the-workflow-engine"
TARGET="${1:-all}"

echo "=== CC Verify — workflow-trace ==="
echo "Target: $TARGET"
echo ""

if [[ "$TARGET" = "all" ]]; then
  echo "Dispatching cc-contract-verifier agent for all 7 contracts (CC-1..CC-7)..."
  echo "Reads cluster-A through cluster-H specs in parallel."
  echo ""
  echo "Expected report sections:"
  echo "  CC-1 Cascade-Cost Coupling             (m4 + m6 -> m7 JSONB join)"
  echo "  CC-2 Trust Layer Woven                 (Cluster D -> ALL)"
  echo "  CC-3 Evidence-Driven Iteration         (Cluster E -> Cluster F)"
  echo "  CC-4 Proposal -> Bank -> Dispatch      (m23 -> [HUMAN] -> m30 -> m32)"
  echo "  CC-5 Substrate Learning Loop           (Cluster H -> m22 back-feed)"
  echo "  CC-6 Verification-Gated Dispatch       (m33 -> m32 refusal-mode)"
  echo "  CC-7 Pressure-Driven Evolution         (m15 -> spec interviews)"
else
  CC_NUM=$(echo "$TARGET" | sed 's/^CC-//')
  echo "Dispatching cc-contract-verifier for $TARGET only..."
fi

echo ""
echo "Agent definition: $ROOT/.claude/agents/cc-contract-verifier.md"
echo "Source spec context: $ROOT/CLAUDE.md § Cross-cluster synergies"
echo ""
echo "(invoke the agent via Task tool: subagent_type=cc-contract-verifier with target=$TARGET)"
```
