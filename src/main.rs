use bevy::{
    prelude::{
        App, Camera, Camera2dBundle, ClearColor, Color, Commands, Component, Entity, EventReader,
        EventWriter, GlobalTransform, Input, IntoSystemConfig, MouseButton, Name, PluginGroup,
        Query, Res, StartupSet, With, Without,
    },
    sprite::Sprite,
    window::{PrimaryWindow, Window, WindowPlugin},
    DefaultPlugins,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use grid::{setup_grid, Cell, GridOptions, GridPlugin, Position};
use rand::seq::IteratorRandom;

mod grid;

fn main() {
    App::new()
        .add_event::<PlayerClickEvent>()
        .insert_resource(ClearColor(Color::DARK_GRAY))
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
        // todo: voir pour pouvoir ajouter la resource peut importe l'ordre (Via Option ? Avec une method qui set une resource si None ?).
        .add_plugin(GridPlugin)
        .add_startup_system(setup_camera)
        .add_startup_system(setup_grid)
        .add_startup_system(drop_mines.in_base_set(StartupSet::PostStartup))
        .add_startup_system(
            set_mines_neighbors_count
                .in_base_set(StartupSet::PostStartupFlush)
                .after(drop_mines),
        )
        .add_system(handle_click)
        .add_system(display_clicked_cell)
        .run();
}

fn handle_click(
    mut player_click_event_writer: EventWriter<PlayerClickEvent>,
    mouse_buttons: Res<Input<MouseButton>>,
    grid_options: Res<GridOptions>,
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
            // todo refactor + decomposer et commenter
            let x = ((cursor_world_pos.x / grid_options.cell_width)
                + (grid_options.cell_count_per_row as f32 / 2.))
                .abs()
                .floor();
            let y = ((cursor_world_pos.y / grid_options.cell_height)
                - (grid_options.row_count as f32 / 2.))
                .abs()
                .floor();
            player_click_event_writer.send(PlayerClickEvent(Position {
                x: x as i32,
                y: y as i32,
            }));
        }
    }
}

fn display_clicked_cell(
    mut player_clicked_events: EventReader<PlayerClickEvent>,
    mut empty_cells: Query<
        (&Position, &mut Sprite),
        (With<Cell>, Without<Mine>, Without<MineNeighbor>),
    >,
    mut mine_cells: Query<
        (&Position, &mut Sprite),
        (With<Cell>, With<Mine>, Without<MineNeighbor>),
    >,
    mut mine_neighbors_cells: Query<
        (&Position, &mut Sprite, &MineNeighbor),
        (With<Cell>, Without<Mine>),
    >,
) {
    for PlayerClickEvent(clicked_pos) in player_clicked_events.iter() {
        let cells_to_uncover = flood_fill(
            clicked_pos,
            &empty_cells,
            &mine_cells,
            &mine_neighbors_cells,
        );

        println!("cell to uncover : {}", cells_to_uncover.len());

        'outer: for (empty_cell_pos, mut sprite) in empty_cells.iter_mut() {
            for pos_to_uncover in cells_to_uncover.iter() {
                if empty_cell_pos == pos_to_uncover {
                    sprite.color = Color::ANTIQUE_WHITE;
                    continue 'outer;
                }
            }
        }
    }
}

fn flood_fill(
    uncovered_pos: &Position,
    empty_cells: &Query<
        (&Position, &mut Sprite),
        (With<Cell>, Without<Mine>, Without<MineNeighbor>),
    >,
    mine_cells: &Query<(&Position, &mut Sprite), (With<Cell>, With<Mine>, Without<MineNeighbor>)>,
    mine_neighbors_cells: &Query<
        (&Position, &mut Sprite, &MineNeighbor),
        (With<Cell>, Without<Mine>),
    >,
) -> Vec<Position> {
    let mut cell_pos_to_uncover = vec![];
    let uncovered_pos_neighbor = calc_pos_neighbors(uncovered_pos);

    for (empty_cell_pos, _) in (*empty_cells).iter() {
        if *uncovered_pos == *empty_cell_pos {
            cell_pos_to_uncover.push((*empty_cell_pos).clone());
        }

        for neighbor_pos in uncovered_pos_neighbor.iter() {
            // todo: doit être marqué découvert, sinon boucle à l'infini avec les voisins.
            if neighbor_pos.x >= 0
                && neighbor_pos.x <= 19
                && neighbor_pos.y >= 0
                && neighbor_pos.y <= 19
            {
                let mut neighbors_cells_to_uncover = flood_fill(
                    neighbor_pos,
                    &empty_cells,
                    &mine_cells,
                    &mine_neighbors_cells,
                );
                cell_pos_to_uncover.append(&mut neighbors_cells_to_uncover);
            }
        }
    }

    // for (mine_cell_pos, mut sprite) in (*mine_cells).iter() {
    //     if *uncovered_pos == *mine_cell_pos {
    //         sprite.color = Color::BLACK;
    //     }

    //     for neighbor_pos in uncovered_pos_neighbor.iter() {
    //         if *neighbor_pos == *mine_cell_pos {
    //             sprite.color = Color::BLACK;
    //         }
    //     }
    // }

    for (neighbor_cell_pos, sprite, mine_neighbor) in (*mine_neighbors_cells).iter() {
        // let sprite_color = match mine_neighbor.mines_count {
        //     1 => Color::GREEN,
        //     2 => Color::CYAN,
        //     3 => Color::ALICE_BLUE,
        //     4 => Color::YELLOW,
        //     5 => Color::ORANGE,
        //     _ => Color::RED,
        // };

        if *uncovered_pos == *neighbor_cell_pos {
            println!("mine_neighbors_cells added");
            cell_pos_to_uncover.push((*neighbor_cell_pos).clone());
        }

        // for neighbor_pos in uncovered_pos_neighbor.iter() {
        //     if *neighbor_pos == *neighbor_cell_pos {
        //         sprite.color = sprite_color;
        //     }
        // }
    }

    cell_pos_to_uncover
}

fn setup_camera(mut commands: Commands) {
    commands
        .spawn(Name::new("main_camera"))
        .insert(Camera2dBundle::default())
        .insert(MainCamera);
}

fn drop_mines(mut commands: Commands, mut cell_entities: Query<Entity, With<Cell>>) {
    let mine_count = 30;
    let mut rng = rand::thread_rng();
    let bomb_cells = cell_entities
        .iter_mut()
        .choose_multiple(&mut rng, mine_count);
    for entity in bomb_cells {
        commands.entity(entity).insert(Mine);
    }
}

fn set_mines_neighbors_count(
    mut commands: Commands,
    mut cell_entities: Query<(Entity, &Position), (With<Cell>, Without<Mine>)>,
    mines_pos: Query<&Position, With<Mine>>,
) {
    for (entity, position) in cell_entities.iter_mut() {
        let mut neighbor_mines_count = 0;
        let neighbors_pos = calc_pos_neighbors(position);
        for mine_pos in mines_pos.iter() {
            for cell_neighbor_pos in neighbors_pos.iter() {
                if *mine_pos == *cell_neighbor_pos {
                    neighbor_mines_count += 1;
                }
            }
        }

        if neighbor_mines_count > 0 {
            commands.entity(entity).insert(MineNeighbor {
                mines_count: neighbor_mines_count,
            });
        }
    }
}

fn calc_pos_neighbors(pos: &Position) -> Vec<Position> {
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

#[derive(Debug, Component)]
pub struct Mine;

#[derive(Debug, Component, PartialEq, Eq)]
pub struct MineNeighbor {
    mines_count: u8,
}

#[derive(Debug, Component)]
struct MainCamera;

struct PlayerClickEvent(Position);
