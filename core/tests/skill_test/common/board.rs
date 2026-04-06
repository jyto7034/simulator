use std::collections::HashMap;

use game_core::ecs::resources::Position;

use crate::common::{BOARD_HEIGHT, BOARD_WIDTH};

use super::{PlacedUnitKind, UnitPatch};

#[derive(Debug, Clone)]
pub struct UnitSeed {
    pub kind: PlacedUnitKind,
    pub position: Position,
    pub raw_token: String,
    pub unit_symbol: char,
    pub patch: UnitPatch,
}

#[derive(Debug, Clone)]
pub struct CasterSeed {
    pub position: Position,
    pub raw_token: String,
    pub unit_symbol: char,
    pub patch: UnitPatch,
}

#[derive(Debug, Clone)]
pub struct BoardCell {
    pub position: Position,
    pub raw_token: String,
    pub unit_symbol: char,
    pub marker_symbols: Vec<char>,
    pub patch: UnitPatch,
}

#[derive(Debug, Clone)]
pub struct BoardScenario {
    pub width: u8,
    pub height: u8,
    pub caster: CasterSeed,
    pub player_support_units: Vec<UnitSeed>,
    pub opponent_units: Vec<UnitSeed>,
    cells: HashMap<Position, BoardCell>,
}

impl BoardScenario {
    pub fn cell_at(&self, position: Position) -> Option<&BoardCell> {
        self.cells.get(&position)
    }

    pub fn cells(&self) -> impl Iterator<Item = &BoardCell> {
        self.cells.values()
    }

    pub fn all_unit_positions(&self) -> impl Iterator<Item = Position> + '_ {
        self.cells.keys().copied()
    }
}

#[derive(Debug, Clone)]
pub enum BoardEntry {
    Caster,
    Player(PlacedUnitKind),
    Opponent(PlacedUnitKind),
}

#[derive(Debug, Default)]
pub struct BoardLegend {
    pub units: HashMap<char, BoardEntry>,
    pub patches: HashMap<char, UnitPatch>,
}

pub fn skill_dummy_board_legend() -> BoardLegend {
    BoardLegend {
        units: HashMap::from([
            ('C', BoardEntry::Caster),
            ('S', BoardEntry::Player(PlacedUnitKind::SkillDummy)),
            ('D', BoardEntry::Opponent(PlacedUnitKind::SkillDummy)),
        ]),
        patches: HashMap::new(),
    }
}

fn parse_cell(token: &str) -> Option<(char, Vec<char>)> {
    if token == "." {
        return None;
    }

    let mut chars = token.chars();
    let unit_symbol = chars.next().expect("non-empty token expected");
    assert!(
        unit_symbol.is_ascii_alphabetic(),
        "board cell token must start with an alphabetic unit symbol: {token}"
    );

    let marker_symbols: Vec<char> = chars.collect();
    assert!(
        marker_symbols
            .iter()
            .all(|marker| !marker.is_ascii_alphabetic()),
        "only the first character may be alphabetic in board token: {token}"
    );

    Some((unit_symbol, marker_symbols))
}

pub fn scenario_from_board(board: &str, legend: &BoardLegend) -> BoardScenario {
    let rows: Vec<Vec<&str>> = board
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| line.split_whitespace().collect())
        .collect();

    assert!(
        !rows.is_empty(),
        "board must contain at least one non-empty row"
    );

    let width = rows[0].len();
    assert!(width > 0, "board rows must contain at least one token");
    for row in &rows {
        assert_eq!(row.len(), width, "board rows must have a consistent width");
    }

    let height = rows.len();
    assert!(
        width <= BOARD_WIDTH as usize,
        "board width {} exceeds test battlefield width {}",
        width,
        BOARD_WIDTH
    );
    assert!(
        height <= BOARD_HEIGHT as usize,
        "board height {} exceeds test battlefield height {}",
        height,
        BOARD_HEIGHT
    );

    let mut caster: Option<CasterSeed> = None;
    let mut player_support_units = Vec::new();
    let mut opponent_units = Vec::new();
    let mut cells = HashMap::new();

    for (y, row) in rows.iter().enumerate() {
        for (x, token) in row.iter().enumerate() {
            let Some((unit_symbol, marker_symbols)) = parse_cell(token) else {
                continue;
            };

            let mut patch = UnitPatch::default();
            for marker in &marker_symbols {
                let marker_patch = legend
                    .patches
                    .get(marker)
                    .unwrap_or_else(|| panic!("unknown board patch marker '{marker}'"));
                patch.merge_from(marker_patch);
            }

            let position = Position::new(x as i32, y as i32);
            let raw_token = (*token).to_string();
            let cell = BoardCell {
                position,
                raw_token: raw_token.clone(),
                unit_symbol,
                marker_symbols: marker_symbols.clone(),
                patch: patch.clone(),
            };
            cells.insert(position, cell);

            match legend
                .units
                .get(&unit_symbol)
                .unwrap_or_else(|| panic!("unknown board unit symbol '{unit_symbol}'"))
            {
                BoardEntry::Caster => {
                    assert!(caster.is_none(), "board may contain only one caster symbol");
                    caster = Some(CasterSeed {
                        position,
                        raw_token,
                        unit_symbol,
                        patch,
                    });
                }
                BoardEntry::Player(kind) => player_support_units.push(UnitSeed {
                    kind: kind.clone(),
                    position,
                    raw_token,
                    unit_symbol,
                    patch,
                }),
                BoardEntry::Opponent(kind) => opponent_units.push(UnitSeed {
                    kind: kind.clone(),
                    position,
                    raw_token,
                    unit_symbol,
                    patch,
                }),
            }
        }
    }

    BoardScenario {
        width: width as u8,
        height: height as u8,
        caster: caster.expect("board must contain exactly one caster"),
        player_support_units,
        opponent_units,
        cells,
    }
}
