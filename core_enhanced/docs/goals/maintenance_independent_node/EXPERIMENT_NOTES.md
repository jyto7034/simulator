# Maintenance Independent Node Experiment Notes

## 2026-06-08 - Source Of Truth Boundary

Current code shows Maintenance as:

```text
MapNodeCategory::Support
MapNodePayload::Support { support_type: SupportNodeType::Maintenance, ... }
```

The new policy aims for:

```text
MapNodeCategory::Maintenance
MapNodePayload::Maintenance
selected_event.type == "maintenance"
```

## 2026-06-08 - Design Rationale

Medical/Rest are support effects applied at node completion. Maintenance is different:

- It exposes multiple repeatable operations while inside the node.
- It has operation previews, costs, gains, and warnings.
- Completing the node does not apply the main Maintenance effect; the work commands do.

This makes Maintenance a better fit for an independent node than a Support subtype.

## 2026-06-08 - Policy Questions To Watch

Stop and ask the user if any of these become necessary:

- Keeping Maintenance selectable through `SupportNodeMode::FullChoice`.
- Supporting old and new Maintenance node schemas at the same time.
- Migrating persisted run/save data.
- Keeping Unity's old `support_type == "Maintenance"` contract temporarily.

## 2026-06-08 - Runtime Shape Decision

Runtime code confirms that Maintenance is currently only reachable through
`ActiveNodeContent::Support(SupportSessionState)` and `SupportNodeType::Maintenance`.

Chosen long-term direction:

- Add a dedicated `MaintenanceSessionState { node_id }`.
- Add `ActiveNodeContent::Maintenance`.
- Keep existing Maintenance operation command/result DTOs.
- Move preview generation behind a Maintenance-specific helper instead of keeping it tied to Support.

Reason:

- Maintenance does not need support choices, medical target selection, or medical treatment fields.
- The node's identity becomes explicit in snapshot and action gating.
- This avoids a compatibility layer where both Support-Maintenance and independent Maintenance remain valid.
