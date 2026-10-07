"""Prove the equipment slot resolver contains one direct item-map lookup and no scan."""

from pathlib import Path


path = Path(
    "/mnt/f/work/simulator/core_enhanced/src/game/battle/core/value_expr_runtime.rs"
)
source = path.read_text()
start = source.index("    fn resolved_slot_binding<'a>(")
end = source.index("\n    fn resolved_relation_owner(", start)
body = source[start:end]
assert ".items.values()" not in body
assert ".items\n                .get(&entry.runtime_item_instance_id)" in body
assert body.count(".get(&entry.runtime_item_instance_id)") == 1
for identity_check in (
    "item.instance_id == entry.runtime_item_instance_id",
    "item.owner_unit_instance == unit.instance_id",
    "item.base_uuid == binding.base_uuid",
    "item.persistent_copy_uuid == Some(copy_uuid)",
):
    assert identity_check in body
print(
    "resolved_slot_binding: item_values_scans=0 direct_runtime_id_gets=1 "
    "identity_checks=instance,owner,base,copy"
)
