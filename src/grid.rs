use bevy::{
    prelude::{
        BuildChildren, Color, Commands, Component, Name, Plugin, Res, Resource, SpatialBundle,
        Transform, Vec2, Visibility,
    },
    sprite::{Sprite, SpriteBundle},
};

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(GridOptions::default());
        // .add_startup_system(setup_grid);
    }
}

const ROW_COUNT: u32 = 20;
const CELL_COUNT_PER_ROW: u32 = 20;
const CELL_SIZE: f32 = 25.;
const CELL_PADDING: f32 = 2.5;

#[derive(Debug, Resource)]
pub struct GridOptions {
    pub row_count: u32,
    pub cell_count_per_row: u32,
    pub cell_width: f32,
    pub cell_height: f32,
    pub cell_padding: f32,
}

impl Default for GridOptions {
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

pub fn setup_grid(mut commands: Commands, grid_options: Res<GridOptions>) {
    println!("generating grid...");
    let grid_size = (
        // Define the width of the grid based on the count of cell per row and the cell width.
        grid_options.cell_count_per_row as f32 * grid_options.cell_width,
        // Define the width of the grid based on the count of row and the cell height.
        grid_options.row_count as f32 * grid_options.cell_height as f32,
    );

    // Set the grid origin position based on the size calculated previously.
    let grid_origin_pos = Transform::from_xyz(-grid_size.0 / 2., grid_size.1 / 2., 0.);

    commands
        .spawn(Name::new("Grid"))
        .insert(Grid)
        .insert(SpatialBundle {
            // todo: changer en visibilitybundle
            visibility: Visibility::Visible,
            transform: grid_origin_pos,
            ..Default::default()
        })
        .insert(SpriteBundle {
            sprite: Sprite {
                color: Color::DARK_GRAY,
                custom_size: Some(Vec2::new(grid_size.0, grid_size.1)),
                ..Default::default()
            },
            // todo déplacer le transform ici
            ..Default::default()
        })
        .with_children(|parent| {
            for row in 0..grid_options.row_count {
                for col in 0..grid_options.cell_count_per_row {
                    let children_pos = Transform::from_xyz(
                        (grid_origin_pos.translation.x + grid_options.cell_width / 2.)
                            + (grid_options.cell_width * col as f32),
                        (grid_origin_pos.translation.y - grid_options.cell_height / 2.)
                            - (grid_options.cell_height * row as f32),
                        1.,
                    );
                    parent
                        .spawn(Name::new(format!("Cell ({}, {})", row, col)))
                        .insert(Cell)
                        .insert(SpatialBundle {
                            visibility: Visibility::Visible,
                            transform: children_pos,
                            ..Default::default()
                        })
                        .insert(SpriteBundle {
                            sprite: Sprite {
                                color: Color::WHITE,
                                custom_size: Some(Vec2::new(
                                    grid_options.cell_width - grid_options.cell_padding,
                                    grid_options.cell_height - grid_options.cell_padding,
                                )),
                                ..Default::default()
                            },
                            transform: children_pos,
                            ..Default::default()
                        })
                        .insert(Position {
                            x: col as i32,
                            y: row as i32,
                        });
                }
            }
        });
}

#[derive(Debug, Clone, Copy, Component)]
pub struct Grid;

#[derive(Debug, Clone, Copy, Component)]
pub struct Cell;

#[derive(Debug, Component, PartialEq, Eq, Clone)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}
