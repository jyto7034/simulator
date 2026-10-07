"""Measure the old and corrected equipment projection layouts with rustc."""

import subprocess
from pathlib import Path


deps = Path("/home/blast/.cache/d04-owner-f01-product-target/debug/deps")
rlib = max(deps.glob("libgame_core-*.rlib"), key=lambda path: path.stat().st_mtime_ns)
source = r"""
use game_core::game::{
    battle::{core::types::BattleEquipmentSlots, types::BattleEquipmentBinding},
    resources::item_slot::SlotLayoutKind,
};
#[allow(dead_code)]
struct PreviousBattleEquipmentSlots {
    layout: SlotLayoutKind,
    slots: [Option<BattleEquipmentBinding>; 3],
}
fn main() {
    let old_size = std::mem::size_of::<PreviousBattleEquipmentSlots>();
    let new_size = std::mem::size_of::<BattleEquipmentSlots>();
    let arc_header = 2 * std::mem::size_of::<usize>();
    println!(
        "previous_size={old_size} previous_retained={} corrected_size={new_size} corrected_retained={} retained_increase={}",
        old_size + arc_header,
        new_size + arc_header,
        new_size - old_size,
    );
}
"""
binary = Path("/tmp/wp02_d04_owner_f01_layout_size_probe")
subprocess.run(
    [
        "/home/blast/.cargo/bin/rustc",
        "--edition=2021",
        "-",
        "-L",
        f"dependency={deps}",
        "--extern",
        f"game_core={rlib}",
        "-o",
        str(binary),
    ],
    input=source.encode(),
    check=True,
)
output = subprocess.check_output([binary], text=True)
print(output, end="")
assert output == (
    "previous_size=106 previous_retained=122 corrected_size=154 "
    "corrected_retained=170 retained_increase=48\n"
)
