use std::collections::VecDeque;

use bevy::{
    prelude::{
        apply_system_buffers, App, Camera, Camera2dBundle, ClearColor, Color, Commands, Component,
        Entity, EventReader, EventWriter, GlobalTransform, Input, IntoSystemAppConfigs,
        IntoSystemConfigs, MouseButton, Name, NextState, OnEnter, OnUpdate, PluginGroup, Query,
        Res, ResMut, States, Transform, With,
    },
    sprite::Sprite,
    window::{PrimaryWindow, Window, WindowPlugin},
    DefaultPlugins,
};

use grid::{
    calc_pos_neighbors, drop_mines, set_mines_neighbors_count, setup_grid, Cell, CellKind,
    CellState, GridParams, Position,
};

mod grid;

#[derive(States, PartialEq, Eq, Debug, Clone, Hash, Default)]
enum GameState {
    #[default]
    Loading,
    InGame,
}

fn main() {
    App::new()
        .add_state::<GameState>()
        .add_event::<CellClickedEvent>()
        .insert_resource(ClearColor(Color::DARK_GRAY))
        .insert_resource(GridParams::default()) // todo make it optional
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
        .add_systems((handle_click, flood_fill).in_set(OnUpdate(GameState::InGame)))
        .run();
}

fn launch_game(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::InGame);
}

fn handle_click(
    mut player_click_event_writer: EventWriter<CellClickedEvent>,
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
}

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

#[derive(Debug, Component)]
struct MainCamera;

struct CellClickedEvent(Position);

fn flood_fill(
    grid_params: Res<GridParams>,
    mut cell_clicked_event: EventReader<CellClickedEvent>,
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
                    match *cell_kind {
                        CellKind::Empty => {
                            sprite.color = Color::ANTIQUE_WHITE;
                            *cell_state = CellState::Uncovered;

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
                        CellKind::MineNeighbor { mines_count } => {
                            println!("discovering mine neighbor cell !");
                            let cell_color = match mines_count {
                                1 => Color::BLUE,
                                2 => Color::CYAN,
                                3 => Color::GREEN,
                                4 => Color::YELLOW,
                                5 => Color::ORANGE,
                                _ => Color::RED,
                            };
                            sprite.color = cell_color;
                            *cell_state = CellState::Uncovered;
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

// fn _flood_fill(
//     uncovered_pos: &Position,
//     empty_cells: &Query<
//         (&Position, &mut Sprite),
//         (With<Cell>, Without<Mine>, Without<MineNeighbor>),
//     >,
//     mine_cells: &Query<(&Position, &mut Sprite), (With<Cell>, With<Mine>, Without<MineNeighbor>)>,
//     mine_neighbors_cells: &Query<
//         (&Position, &mut Sprite, &MineNeighbor),
//         (With<Cell>, Without<Mine>),
//     >,
// ) -> Vec<Position> {
//     let mut cell_pos_to_uncover = vec![];
//     let uncovered_pos_neighbor = calc_pos_neighbors(uncovered_pos);

//     for (empty_cell_pos, _) in (*empty_cells).iter() {
//         if *uncovered_pos == *empty_cell_pos {
//             cell_pos_to_uncover.push((*empty_cell_pos).clone());
//         }

//         for neighbor_pos in uncovered_pos_neighbor.iter() {
//             // todo: doit être marqué découvert, sinon boucle à l'infini avec les voisins.
//             if neighbor_pos.x >= 0
//                 && neighbor_pos.x <= 19
//                 && neighbor_pos.y >= 0
//                 && neighbor_pos.y <= 19
//             {
//                 let mut neighbors_cells_to_uncover = _flood_fill(
//                     neighbor_pos,
//                     &empty_cells,
//                     &mine_cells,
//                     &mine_neighbors_cells,
//                 );
//                 cell_pos_to_uncover.append(&mut neighbors_cells_to_uncover);
//             }
//         }
//     }

//     // for (mine_cell_pos, mut sprite) in (*mine_cells).iter() {
//     //     if *uncovered_pos == *mine_cell_pos {
//     //         sprite.color = Color::BLACK;
//     //     }

//     //     for neighbor_pos in uncovered_pos_neighbor.iter() {
//     //         if *neighbor_pos == *mine_cell_pos {
//     //             sprite.color = Color::BLACK;
//     //         }
//     //     }
//     // }

//     for (neighbor_cell_pos, sprite, mine_neighbor) in (*mine_neighbors_cells).iter() {
//         // let sprite_color = match mine_neighbor.mines_count {
//         //     1 => Color::GREEN,
//         //     2 => Color::CYAN,
//         //     3 => Color::ALICE_BLUE,
//         //     4 => Color::YELLOW,
//         //     5 => Color::ORANGE,
//         //     _ => Color::RED,
//         // };

//         if *uncovered_pos == *neighbor_cell_pos {
//             println!("mine_neighbors_cells added");
//             cell_pos_to_uncover.push((*neighbor_cell_pos).clone());
//         }

//         // for neighbor_pos in uncovered_pos_neighbor.iter() {
//         //     if *neighbor_pos == *neighbor_cell_pos {
//         //         sprite.color = sprite_color;
//         //     }
//         // }
//     }

//     cell_pos_to_uncover
// }
