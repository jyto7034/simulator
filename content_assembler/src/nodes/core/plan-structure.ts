import type { GraphDocument, GraphNode } from "../types";
import { inputSources } from "./connections";

// Traverse authored definitions only; execution snapshots and iteration live in Core.
export function planStructure(doc: GraphDocument) {
  const source = inputSources(doc);
  const planChildren = (node: GraphNode) => {
    const ports =
      node.kind === "skill-on-hit-plan"
        ? ["body", "after"]
        : node.kind === "skill-sequence-plan"
          ? ["first", "next"]
          : [
                "skill-repeat-plan",
                "skill-spend-plan",
                "skill-timed-plan",
                "skill-scheduled-plan",
                "skill-convert-status-plan",
              ].includes(node.kind)
            ? ["body"]
            : [];
    return ports
      .map((p) => source(node, p))
      .filter((n): n is GraphNode => Boolean(n));
  };
  const plans = (root: GraphNode | undefined) => {
    const seen = new Map<string, GraphNode>();
    const visit = (n: GraphNode | undefined) => {
      if (!n || seen.has(n.id)) return;
      seen.set(n.id, n);
      planChildren(n).forEach(visit);
    };
    visit(root);
    return [...seen.values()];
  };
  const entries = (program: GraphNode | undefined): GraphNode[] => {
    if (program?.kind === "skill-action-entry") return [program];
    if (program?.kind !== "skill-mode-program") return [];
    return [source(program, "normal"), source(program, "active")].filter(
      (n): n is GraphNode => n?.kind === "skill-action-entry",
    );
  };
  const counts = (root: GraphNode | undefined) => {
    const seen = new Map<string, GraphNode>();
    const visit = (n: GraphNode | undefined) => {
      if (!n || seen.has(n.id)) return;
      seen.set(n.id, n);
      if (n.kind === "skill-count-quotient") visit(source(n, "count"));
      if (n.kind === "skill-count-sum") {
        visit(source(n, "first"));
        visit(source(n, "second"));
      }
    };
    visit(root);
    return [...seen.values()];
  };
  const preparation = (plan: GraphNode | undefined): GraphNode | undefined => {
    if (!plan) return;
    if (plan.kind === "skill-plan-merge") {
      const precise = source(plan, "precise");
      const layout = precise && source(precise, "layout");
      return layout && source(layout, "preparation");
    }
    const layout = source(plan, "layout");
    return layout && source(layout, "preparation");
  };
  const chargeForVolley = (volley: GraphNode | undefined) => {
    const right = volley && source(volley, "right");
    return right && source(right, "charge");
  };
  return { planChildren, plans, entries, counts, preparation, chargeForVolley };
}
