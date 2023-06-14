use bevy::{
    prelude::{
        BuildChildren, Color, Commands, Component, Name, Query, Res, Resource, Transform, Vec2,
        Visibility,
    },
    sprite::{Anchor, Sprite, SpriteBundle},
};
use rand::seq::IteratorRandom;

const ROW_COUNT: u32 = 20;
const CELL_COUNT_PER_ROW: u32 = 20;
const CELL_SIZE: f32 = 25.;
const CELL_PADDING: f32 = 2.5;

#[derive(Debug, Resource)]
pub struct GridParams {
    pub row_count: u32,
    pub cell_count_per_row: u32,
    pub cell_width: f32,
    pub cell_height: f32,
    pub cell_padding: f32,
}

impl Default for GridParams {
    fn default() -> Self {
        Self {
            row_count: ROW_COUNT,
            cell_count_per_row: CELL_COUNT_PER_ROW,
            cell_width: CELL_SIZE,
            cell_height: CELL_SIZE,
            cell_padding: CELL_PADDING,
        }
    }
}

pub fn setup_grid(mut commands: Commands, grid_params: Res<GridParams>) {
    let grid_size = (
        // Define the width of the grid based on the count of cell per row and the cell width.
        grid_params.cell_count_per_row as f32 * grid_params.cell_width,
        // Define the width of the grid based on the count of row and the cell height.
        grid_params.row_count as f32 * grid_params.cell_height as f32,
    );

    // Set the grid origin position based on the size calculated previously.
    let grid_origin_pos = Transform::IDENTITY;
    commands
        .spawn(Name::new("Grid"))
        .insert(Grid)
        .insert(SpriteBundle {
            sprite: Sprite {
                color: Color::DARK_GRAY,
                custom_size: Some(Vec2::new(grid_size.0, grid_size.1)),
                ..Default::default()
            },
            visibility: Visibility::Visible,
            transform: grid_origin_pos,
            ..Default::default()
        })
        .with_children(|parent| {
            for row in 0..grid_params.row_count {
                for col in 0..grid_params.cell_count_per_row {
                    let children_pos = Transform::from_xyz(
                        grid_origin_pos.translation.x + grid_params.cell_width * col as f32,
                        grid_origin_pos.translation.y + grid_params.cell_height * row as f32,
                        1.,
                    );
                    parent
                        .spawn(Name::new(format!("Cell ({}, {})", row, col)))
                        .insert(Cell)
                        .insert(SpriteBundle {
                            sprite: Sprite {
                                color: Color::WHITE,
                                custom_size: Some(Vec2::new(
                                    grid_params.cell_width - grid_params.cell_padding,
                                    grid_params.cell_height - grid_params.cell_padding,
                                )),
                                anchor: Anchor::BottomLeft,
                                ..Default::default()
                            },
                            visibility: Visibility::Visible,
                            transform: children_pos,
                            ..Default::default()
                        })
                        .insert(Position {
                            x: col as i32,
                            y: row as i32,
                        })
                        .insert(CellState::Covered)
                        .insert(CellKind::Empty);
                }
            }
        });
}

#[derive(Debug, Clone, Copy, Component)]
pub struct Grid;

#[derive(Debug, Clone, Copy, Component)]
pub struct Cell;

#[derive(Debug, Component, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum CellState {
    // todo: make separate component
    #[default]
    Covered,
    Uncovered,
    Flagged,
}

#[derive(Debug, Component, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

pub fn drop_mines(mut cell_entities: Query<&mut CellKind>) {
    let mine_count = 40; // todo: baser sur la taille de la grille
    let mut rng = rand::thread_rng();
    let bomb_cells = cell_entities
        .iter_mut()
        .choose_multiple(&mut rng, mine_count);
    for mut cell_kind in bomb_cells {
        *cell_kind = CellKind::Mine;
    }
}

pub fn set_mines_neighbors_count(mut cell_entities: Query<(&Position, &mut CellKind)>) {
    let mines_pos = cell_entities
        .iter()
        .filter_map(|(position, cell_kind)| {
            if *cell_kind != CellKind::Mine {
                return None;
            }

            Some(*position)
        })
        .collect::<Vec<Position>>();

    for (position, mut cell_kind) in cell_entities.iter_mut() {
        if *cell_kind != CellKind::Empty {
            continue;
        }

        let mut neighbor_mines_count = 0;
        let neighbors_pos = calc_pos_neighbors(*position);
        for mine_pos in mines_pos.iter() {
            for cell_neighbor_pos in neighbors_pos.iter() {
                if *mine_pos == *cell_neighbor_pos {
                    neighbor_mines_count += 1;
                }
            }
        }

        if neighbor_mines_count > 0 {
            *cell_kind = CellKind::MineNeighbor {
                mines_count: neighbor_mines_count,
            };
        }
    }
}

pub fn calc_pos_neighbors(pos: Position) -> Vec<Position> {
    vec![
        Position {
            x: pos.x - 1,
            y: pos.y - 1,
        },
        Position {
            x: pos.x,
            y: pos.y - 1,
        },
        Position {
            x: pos.x + 1,
            y: pos.y - 1,
        },
        Position {
            x: pos.x + 1,
            y: pos.y,
        },
        Position {
            x: pos.x + 1,
            y: pos.y + 1,
        },
        Position {
            x: pos.x,
            y: pos.y + 1,
        },
        Position {
            x: pos.x - 1,
            y: pos.y + 1,
        },
        Position {
            x: pos.x - 1,
            y: pos.y,
        },
    ]
}

#[derive(Debug, Component, PartialEq, Eq, Default)]
pub enum CellKind {
    #[default]
    Empty,
    Mine,
    MineNeighbor {
        mines_count: u8,
    },
}
