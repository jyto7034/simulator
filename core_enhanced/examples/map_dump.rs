use std::collections::BTreeMap;

use game_core::game::map::{MapGenerationConfig, MapGenerator, MapNodeCategory, RunMap};

fn main() {
    let seed = std::env::args()
        .nth(1)
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(42);
    let map = MapGenerator::generate(seed, MapGenerationConfig::default());

    println!("seed={seed}");
    print_summary(&map);
    println!();
    print_rows(&map);
    println!();
    print_edges(&map);
}

fn print_summary(map: &RunMap) {
    let mut counts = BTreeMap::<&'static str, usize>::new();
    for node in &map.nodes {
        *counts.entry(category_name(node.category)).or_default() += 1;
    }

    println!("nodes={}", map.nodes.len());
    println!("start_nodes={}", map.start_node_ids.len());
    println!("boss_node={}", short_id(map.boss_node_id.0));
    println!("category_counts:");
    for (category, count) in counts {
        println!("  {category}: {count}");
    }
}

fn print_rows(map: &RunMap) {
    let mut rows = BTreeMap::<u8, Vec<_>>::new();
    for node in &map.nodes {
        rows.entry(node.depth).or_default().push(node);
    }

    println!("rows:");
    for (depth, mut nodes) in rows {
        nodes.sort_by_key(|node| node.lane);
        let rendered = nodes
            .iter()
            .map(|node| {
                format!(
                    "{}{}:{}:{}",
                    category_symbol(node.category),
                    node.lane,
                    node.kind_id.as_str(),
                    short_id(node.id.0)
                )
            })
            .collect::<Vec<_>>()
            .join("  ");
        println!("  d{depth:02}  {rendered}");
    }
}

fn print_edges(map: &RunMap) {
    println!("edges:");
    let mut nodes = map.nodes.iter().collect::<Vec<_>>();
    nodes.sort_by_key(|node| (node.depth, node.lane));
    for node in nodes {
        let outgoing = node
            .outgoing
            .iter()
            .filter_map(|id| map.node(*id))
            .map(|to| format!("d{}:{}{}", to.depth, category_symbol(to.category), to.lane))
            .collect::<Vec<_>>()
            .join(", ");
        println!(
            "  d{}:{}{} -> [{}]",
            node.depth,
            category_symbol(node.category),
            node.lane,
            outgoing
        );
    }
}

fn category_symbol(category: MapNodeCategory) -> &'static str {
    match category {
        MapNodeCategory::Start => "A",
        MapNodeCategory::Combat => "C",
        MapNodeCategory::Support => "U",
        MapNodeCategory::HeadquartersContact => "H",
        MapNodeCategory::Shop => "S",
        MapNodeCategory::Boss => "B",
        MapNodeCategory::Reward => "R",
    }
}

fn category_name(category: MapNodeCategory) -> &'static str {
    match category {
        MapNodeCategory::Start => "Start",
        MapNodeCategory::Combat => "Combat",
        MapNodeCategory::Support => "Support",
        MapNodeCategory::HeadquartersContact => "HeadquartersContact",
        MapNodeCategory::Shop => "Shop",
        MapNodeCategory::Boss => "Boss",
        MapNodeCategory::Reward => "Reward",
    }
}

fn short_id(id: uuid::Uuid) -> String {
    let value = id.to_string();
    format!("{}..{}", &value[..4], &value[value.len() - 4..])
}
