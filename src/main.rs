use bevy::asset::AssetMetaCheck;
use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowResolution};
use itertools::Itertools;
use rand::seq::IteratorRandom;

const SQUARES_X: usize = 4;
const SQUARES_Y: usize = 4;

#[derive(Resource)]
struct Board([[u32; SQUARES_Y]; SQUARES_X]);

#[derive(Resource, Default)]
struct Dims {
    width: f32,
    height: f32,
}

#[derive(Component)]
struct Index {
    i: usize,
    j: usize,
}

#[derive(Event)]
struct Reset;

#[derive(Event)]
enum MoveDirection {
    Up,
    Down,
    Left,
    Right,
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    // Wasm builds will check for meta files (that don't exist) if this isn't set.
                    // This causes errors and even panics in web builds on itch.
                    // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: WindowResolution::new(640.0, 640.0),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_systems(Startup, setup)
        .add_event::<Reset>()
        .add_event::<MoveDirection>()
        .add_systems(
            Update,
            (
                keyboard_system,
                new_board.before(update_visuals),
                make_move.before(update_visuals),
                update_visuals,
            ),
        )
        .insert_resource(Board([[0; SQUARES_Y]; SQUARES_X]))
        .insert_resource(Dims::default())
        .run();
}

fn setup(
    mut commands: Commands,
    // asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    board: ResMut<Board>,
    mut dims: ResMut<Dims>,
    mut events: EventWriter<Reset>,
) {
    commands.spawn(Camera2d);

    events.write(Reset);

    let window = window_query.single().unwrap();
    (dims.width, dims.height) = (window.width(), window.height());

    for i in 0..SQUARES_X {
        for j in 0..SQUARES_Y {
            let (x, y) = (
                ((i as f32) - ((SQUARES_X as f32 - 1.0) / 2.0)) * dims.width / SQUARES_X as f32,
                ((j as f32) - ((SQUARES_Y as f32 - 1.0) / 2.0)) * dims.height / SQUARES_Y as f32,
            );

            let square = Mesh2d(meshes.add(Rectangle {
                half_size: Vec2::new(
                    dims.width / (2.0 * SQUARES_X as f32) - 10.0,
                    dims.height / (2.0 * SQUARES_Y as f32) - 10.0,
                ),
            }));

            let white_material =
                MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::WHITE)));

            commands
                .spawn((
                    Text2d::new(format!("{}", board.0[i][j])),
                    TextFont::default(),
                    TextLayout::new_with_justify(JustifyText::Justified),
                    Transform::from_xyz(x, y, 0.0),
                    TextColor::BLACK,
                    Index { i, j },
                ))
                .with_child((square, white_material));
        }
    }
}

fn keyboard_system(
    mut keyboard_events: EventReader<KeyboardInput>,
    mut reset_event: EventWriter<Reset>,
    mut move_event: EventWriter<MoveDirection>,
) {
    for event in keyboard_events.read() {
        if event.state == ButtonState::Pressed {
            match event.key_code {
                KeyCode::ArrowUp | KeyCode::KeyW => {
                    move_event.write(MoveDirection::Up);
                }
                KeyCode::ArrowDown | KeyCode::KeyS => {
                    move_event.write(MoveDirection::Down);
                }
                KeyCode::ArrowLeft | KeyCode::KeyA => {
                    move_event.write(MoveDirection::Left);
                }
                KeyCode::ArrowRight | KeyCode::KeyD => {
                    move_event.write(MoveDirection::Right);
                }

                KeyCode::KeyR => {
                    reset_event.write(Reset);
                }
                _ => {}
            }
        }
    }
}

fn make_move(mut events: EventReader<MoveDirection>, mut board: ResMut<Board>) {
    for event in events.read() {
        let (i_delta, j_delta): (i32, i32) = match event {
            MoveDirection::Up => (0, 1),
            MoveDirection::Down => (0, -1),
            MoveDirection::Left => (-1, 0),
            MoveDirection::Right => (1, 0),
        };

        let mut board_changed = true;

        // move all tiles the desired direction until no tiles move
        while board_changed {
            board_changed = false;
            for i in 0..SQUARES_X {
                for j in 0..SQUARES_Y {
                    if board.0[i][j] != 0 {
                        let (neighbour_i, neighbour_j) = (i as i32 + i_delta, j as i32 + j_delta);
                        if (0..SQUARES_X).contains(&(neighbour_i as usize))
                            & (0..SQUARES_Y).contains(&(neighbour_j as usize))
                            && board.0[neighbour_i as usize][neighbour_j as usize] == 0
                        {
                            board.0[neighbour_i as usize][neighbour_j as usize] = board.0[i][j];
                            board.0[i][j] = 0;
                            board_changed = true;
                        }
                    }
                }
            }
        }

        // merge numbers, just duplicate of the above code
        for i in 0..SQUARES_X {
            for j in 0..SQUARES_Y {
                let first_value = board.0[i][j];
                if first_value != 0 {
                    let (neighbour_i, neighbour_j) = (i as i32 + i_delta, j as i32 + j_delta);
                    if (0..SQUARES_X).contains(&(neighbour_i as usize))
                        & (0..SQUARES_Y).contains(&(neighbour_j as usize))
                        && board.0[neighbour_i as usize][neighbour_j as usize] == first_value
                    {
                        board.0[neighbour_i as usize][neighbour_j as usize] = 2 * board.0[i][j];
                        board.0[i][j] = 0;
                    }
                }
            }
        }

        let rng = &mut rand::rng();

        if let Some((i, j)) = (0..SQUARES_X)
            .cartesian_product(0..SQUARES_Y)
            .filter(|&(i, j)| board.0[i][j] == 0)
            .choose(rng)
        {
            board.0[i][j] = 1;
        } else {
            // game over? not necessarily, could still merge some tiles
        }
    }
}

fn new_board(mut events: EventReader<Reset>, mut board: ResMut<Board>) {
    if !events.is_empty() {
        events.clear();
        let rng = &mut rand::rng();

        board.0 = [[0; SQUARES_Y]; SQUARES_X];

        let first_ones = (0..SQUARES_X)
            .cartesian_product(0..SQUARES_Y)
            .choose_multiple(rng, 2);

        for (i, j) in first_ones {
            board.0[i][j] = 1;
        }
    }
}

fn update_visuals(boxes: Query<(&Index, &mut Visibility, &mut Text2d)>, board: Res<Board>) {
    for (index, mut visibility, mut text) in boxes {
        let value = board.0[index.i][index.j];
        if value == 0 {
            *visibility = Visibility::Hidden;
        } else {
            text.0 = format!("{}", value);
            *visibility = Visibility::Visible;
        }
    }
}
