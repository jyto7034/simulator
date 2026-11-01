// Reuse the release producer/consumer route that loads actual embedded data,
// enters GameCore through public commands, and reaches a live private battle.
// P05 additionally proves below that this valid route has no production event-
// log attachment API or durable result-log field.
include!("../../../battle_kernel_api_audit/valid/src/main.rs");
