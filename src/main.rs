use std::collections::VecDeque;

use bevy::{
    prelude::{
        apply_system_buffers, App, Camera, Camera2dBundle, ClearColor, Color, Commands, Component,
        Entity, EventReader, EventWriter, GlobalTransform, Input, IntoSystemAppConfigs,
        IntoSystemConfigs, MouseButton, Name, NextState, OnEnter, OnUpdate, Parent, PluginGroup,
        Query, Res, ResMut, Resource, States, Transform, Visibility, With,
    },
    sprite::Sprite,
    text::Text,
    window::{PrimaryWindow, Window, WindowPlugin},
    DefaultPlugins,
};

use bevy_inspector_egui::quick::WorldInspectorPlugin;
use grid::{
    calc_pos_neighbors, drop_mines, set_mines_neighbors_count, setup_grid, CellKind, CellState,
    GridParams, Position,
};

mod grid;

fn main() {
    App::new()
        .add_state::<GameState>()
        .add_event::<CellClickedEvent>()
        .add_event::<CellFlaggedEvent>()
        .add_event::<MineNeighborUncoveredEvent>()
        .insert_resource(ClearColor(Color::DARK_GRAY))
        .insert_resource(GridParams::default()) // todo make it optional
        .insert_resource(GameData::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Mine sweeper !".into(),
                resolution: (550., 550.).into(),
                resizable: false,
                ..Default::default()
            }),
            ..Default::default()
        }))
        // .add_plugin(WorldInspectorPlugin::new())
        .add_startup_system(setup_camera)
        .add_systems(
            (
                setup_grid,
                apply_system_buffers,
                drop_mines,
                set_mines_neighbors_count,
                launch_game,
            )
                .chain()
                .in_schedule(OnEnter(GameState::Loading)),
        )
        .add_systems(
            (
                handle_click,
                flood_fill,
                display_uncovered_neighbor_count,
                toggle_cell_flag,
                game_win_wheck,
            )
                .in_set(OnUpdate(GameState::InGame)),
        )
        .run();
}

#[derive(States, PartialEq, Eq, Debug, Clone, Hash, Default)]
enum GameState {
    #[default]
    Loading,
    InGame,
    Win,
}

#[derive(Resource, Clone)]
struct GameData {
    remaining_flags: u32,
    remaining_mines: u32,
}

impl Default for GameData {
    fn default() -> Self {
        Self {
            remaining_flags: 40,
            remaining_mines: 40,
        }
    }
}

#[derive(Debug, Component)]
struct MainCamera;

struct CellClickedEvent(Position);

struct CellFlaggedEvent(Position);

struct MineNeighborUncoveredEvent(Entity);

fn setup_camera(mut commands: Commands, grid_params: Res<GridParams>) {
    let camera_pos = Transform::from_xyz(
        grid_params.cell_width * grid_params.cell_count_per_row as f32 / 2.,
        grid_params.cell_height * grid_params.row_count as f32 / 2.,
        999.,
    );

    commands
        .spawn(Name::new("main_camera"))
        .insert(Camera2dBundle {
            transform: camera_pos,
            ..Default::default()
        })
        .insert(MainCamera);
}

fn launch_game(mut next_state: ResMut<NextState<GameState>>, mut game_data: ResMut<GameData>) {
    *game_data = GameData::default();
    next_state.set(GameState::InGame);
}

fn game_win_wheck(mut next_state: ResMut<NextState<GameState>>, game_data: Res<GameData>) {
    if game_data.remaining_mines == 0 {
        next_state.set(GameState::Win); // todo ajouter système clear grid à l'entrée de ce statut
    }
}

fn handle_click(
    mut player_click_event_writer: EventWriter<CellClickedEvent>,
    mut player_flag_event_writer: EventWriter<CellFlaggedEvent>,
    mouse_buttons: Res<Input<MouseButton>>,
    grid_options: Res<GridParams>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    let (camera, camera_transform) = camera_q.get_single().unwrap();
    let primary_window = primary_window.get_single().unwrap();
    if mouse_buttons.just_pressed(MouseButton::Left) {
        if let Some(cursor_world_pos) = primary_window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
            .map(|ray| ray.origin.truncate())
        {
            let x = cursor_world_pos.x / grid_options.cell_width.floor();
            let y = cursor_world_pos.y / grid_options.cell_height.floor();

            player_click_event_writer.send(CellClickedEvent(Position {
                x: x as i32,
                y: y as i32,
            }));
        }
    }

    if mouse_buttons.just_pressed(MouseButton::Right) {
        if let Some(cursor_world_pos) = primary_window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
            .map(|ray| ray.origin.truncate())
        {
            let x = cursor_world_pos.x / grid_options.cell_width.floor();
            let y = cursor_world_pos.y / grid_options.cell_height.floor();

            player_flag_event_writer.send(CellFlaggedEvent(Position {
                x: x as i32,
                y: y as i32,
            }));
        }
    }
}

fn toggle_cell_flag(
    grid_params: Res<GridParams>, // todo: plutot utiliser le composant grid
    mut game_data: ResMut<GameData>,
    mut cell_flagged_event: EventReader<CellFlaggedEvent>,
    mut cells_entities: Query<(&Position, &mut Sprite, &CellKind, &mut CellState)>,
) {
    if game_data.remaining_flags == 0 {
        return;
    }

    for CellFlaggedEvent(flagged_pos) in cell_flagged_event.iter() {
        if is_out_of_bounds(
            *flagged_pos,
            grid_params.cell_count_per_row,
            grid_params.row_count,
        ) {
            continue;
        }

        for (entity_pos, mut sprite, cell_kind, mut cell_state) in cells_entities.iter_mut() {
            if *flagged_pos != *entity_pos {
                continue;
            }

            match *cell_state {
                CellState::Covered => {
                    *cell_state = CellState::Flagged;
                    sprite.color = Color::BLUE;
                    game_data.remaining_flags -= 1;

                    if *cell_kind == CellKind::Mine {
                        game_data.remaining_mines -= 1;
                    }
                }
                CellState::Uncovered => continue,
                CellState::Flagged => {
                    *cell_state = CellState::Covered;
                    sprite.color = Color::WHITE;
                    game_data.remaining_flags += 1;

                    if *cell_kind == CellKind::Mine {
                        game_data.remaining_mines += 1;
                    }
                }
            };
        }
    }
}

fn flood_fill(
    grid_params: Res<GridParams>,
    mut cell_clicked_event: EventReader<CellClickedEvent>,
    mut neighbor_uncovered_event: EventWriter<MineNeighborUncoveredEvent>,
    mut cells: Query<(Entity, &Position, &mut CellState, &CellKind, &mut Sprite)>,
) {
    for CellClickedEvent(clicked_pos) in cell_clicked_event.iter() {
        if is_out_of_bounds(
            *clicked_pos,
            grid_params.cell_count_per_row,
            grid_params.row_count,
        ) {
            return;
        }

        let mut uncoved_queue: VecDeque<Position> = VecDeque::with_capacity(20);
        uncoved_queue.push_back(*clicked_pos);
        while let Some(uncovered_pos) = uncoved_queue.pop_front() {
            for (entity, cell_pos, mut cell_state, cell_kind, mut sprite) in cells.iter_mut() {
                if *cell_state != CellState::Covered {
                    continue;
                }

                if *cell_pos == *clicked_pos && *cell_kind == CellKind::Mine {
                    sprite.color = Color::BLACK;
                    *cell_state = CellState::Uncovered;
                    println!("It's a mine ! Game Over !"); // todo game over
                    return;
                }

                if *cell_pos == uncovered_pos {
                    sprite.color = Color::ANTIQUE_WHITE;
                    *cell_state = CellState::Uncovered;
                    match *cell_kind {
                        CellKind::Empty => {
                            let neighbors_pos = calc_pos_neighbors(*cell_pos)
                                .into_iter()
                                .filter(|neighbor_pos| {
                                    !is_out_of_bounds(
                                        *neighbor_pos,
                                        grid_params.cell_count_per_row,
                                        grid_params.row_count,
                                    )
                                })
                                .collect::<Vec<Position>>();

                            for neighbor_pos in neighbors_pos.into_iter() {
                                if !uncoved_queue.contains(&neighbor_pos) {
                                    uncoved_queue.push_back(neighbor_pos);
                                }
                            }
                        }
                        CellKind::MineNeighbor { .. } => {
                            neighbor_uncovered_event.send(MineNeighborUncoveredEvent(entity));
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

fn is_out_of_bounds(pos: Position, max_x: u32, max_y: u32) -> bool {
    pos.x < 0 || pos.x > max_x as i32 - 1 || pos.y < 0 || pos.y > max_y as i32 - 1
}

fn display_uncovered_neighbor_count(
    mut neighbor_uncovered_event: EventReader<MineNeighborUncoveredEvent>,
    mut texts: Query<(&mut Visibility, &Parent), With<Text>>,
) {
    for MineNeighborUncoveredEvent(neighbor_entity) in neighbor_uncovered_event.iter() {
        for (mut visility, parent) in texts.iter_mut() {
            if *neighbor_entity == parent.get() {
                *visility = Visibility::Visible;
            }
        }
    }
}
