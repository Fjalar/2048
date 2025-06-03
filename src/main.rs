use bevy::asset::AssetMetaCheck;
use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowResolution};
use bevy_easings::{self, Ease, EaseMethod, EasingType, EasingsPlugin};
use bevy_rand::global::GlobalEntropy;
use bevy_rand::plugin::EntropyPlugin;
use bevy_rand::prelude::WyRand;
use itertools::Itertools;
use rand::seq::IteratorRandom;
use std::time::Duration;

const SQUARES_X: usize = 4;
const SQUARES_Y: usize = 4;

#[derive(Resource)]
struct Board([[Option<Entity>; SQUARES_Y]; SQUARES_X]);

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

#[derive(Component)]
struct AnimateMove;

#[derive(Component)]
struct AnimateSpawn;

#[derive(Component)]
struct AnimateMergeInto;

#[derive(Component)]
struct DespawnLater(Timer);

#[derive(Component)]
struct Value(u32);

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
        .add_plugins((
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
            EntropyPlugin::<WyRand>::default(),
            EasingsPlugin::default(),
        ))
        .add_systems(Startup, setup)
        .add_event::<Reset>()
        .add_event::<MoveDirection>()
        .add_systems(
            Update,
            (
                keyboard_system,
                new_board.before(update_visuals),
                make_move.before(update_visuals),
                despawn_later,
                update_visuals,
            ),
        )
        .insert_resource(Board([[None; SQUARES_Y]; SQUARES_X]))
        .insert_resource(Dims::default())
        .run();
}

fn setup(
    mut commands: Commands,
    // asset_server: Res<AssetServer>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut dims: ResMut<Dims>,
    mut events: EventWriter<Reset>,
) {
    commands.spawn(Camera2d);

    let window = window_query.single().unwrap();
    (dims.width, dims.height) = (window.width(), window.height());

    events.write(Reset);
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

fn make_move(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut events: EventReader<MoveDirection>,
    mut board: ResMut<Board>,
    mut index_query: Query<&mut Index>,
    mut value_query: Query<&mut Value>,
    mut rng: GlobalEntropy<WyRand>,
    dims: Res<Dims>,
) {
    for event in events.read() {
        let mut anything_changed = false;

        let (i_delta, j_delta): (i32, i32) = match event {
            MoveDirection::Up => (0, 1),
            MoveDirection::Down => (0, -1),
            MoveDirection::Left => (-1, 0),
            MoveDirection::Right => (1, 0),
        };

        // for i in 0..SQUARES_X {
        //     for j in 0..SQUARES_Y {
        //         if let Some(ent) = board.0[i][j] {
        //             commands.entity(ent).insert(AnimationThingy {
        //                 origin_i: i,
        //                 origin_j: j,
        //                 frames_left: 30,
        //             });
        //         }
        //     }
        // }

        let mut board_changed = true;

        // move all tiles the desired direction until no tiles move
        while board_changed {
            board_changed = false;
            for i in 0..SQUARES_X {
                for j in 0..SQUARES_Y {
                    if let Some(current) = board.0[i][j] {
                        let (neighbour_i, neighbour_j) = (i as i32 + i_delta, j as i32 + j_delta);
                        if (0..SQUARES_X).contains(&(neighbour_i as usize))
                            & (0..SQUARES_Y).contains(&(neighbour_j as usize))
                            && board.0[neighbour_i as usize][neighbour_j as usize].is_none()
                        {
                            board.0[neighbour_i as usize][neighbour_j as usize] = board.0[i][j];
                            if let Ok(mut idx) = index_query.get_mut(current) {
                                *idx = Index {
                                    i: neighbour_i as usize,
                                    j: neighbour_j as usize,
                                };
                                commands.entity(current).insert(AnimateMove);
                                board.0[i][j] = None;
                                board_changed = true;
                                anything_changed = true;
                            } else {
                                println!("couldn'd find the index entity in move");
                            }
                        }
                    }
                }
            }
        }

        // merge numbers, just duplicate of the above code
        for i in 0..SQUARES_X {
            for j in 0..SQUARES_Y {
                if let Some(current) = board.0[i][j] {
                    let (neighbour_i, neighbour_j) = (i as i32 + i_delta, j as i32 + j_delta);
                    if (0..SQUARES_X).contains(&(neighbour_i as usize))
                        & (0..SQUARES_Y).contains(&(neighbour_j as usize))
                    {
                        if let Some(neighbour) = board.0[neighbour_i as usize][neighbour_j as usize]
                        {
                            if let Ok([val1, mut val2]) =
                                value_query.get_many_mut([current, neighbour])
                            {
                                if val1.0 == val2.0 {
                                    val2.0 *= 2;
                                    board.0[i][j] = None;
                                    commands
                                        .entity(current)
                                        .insert(Index {
                                            i: neighbour_i as usize,
                                            j: neighbour_j as usize,
                                        })
                                        .insert(AnimateMergeInto)
                                        .insert(DespawnLater(Timer::from_seconds(
                                            0.2,
                                            TimerMode::Once,
                                        )));
                                    anything_changed = true;
                                }
                            } else {
                                println!("{:?}", board.0);
                                unreachable!(
                                    "couldn't find the value components of entities in merge"
                                )
                            }
                        }
                    }
                }
            }
        }
        if anything_changed {
            if let Some((i, j)) = (0..SQUARES_X)
                .cartesian_product(0..SQUARES_Y)
                .filter(|&(i, j)| board.0[i][j].is_none())
                .choose(&mut rng)
            {
                let (x, y) = (
                    ((i as f32) - ((SQUARES_X as f32 - 1.0) / 2.0)) * dims.width / SQUARES_X as f32,
                    ((j as f32) - ((SQUARES_Y as f32 - 1.0) / 2.0)) * dims.height
                        / SQUARES_Y as f32,
                );

                let square = Mesh2d(meshes.add(Rectangle {
                    half_size: Vec2::new(
                        dims.width / (2.0 * SQUARES_X as f32) - 10.0,
                        dims.height / (2.0 * SQUARES_Y as f32) - 10.0,
                    ),
                }));

                let white_material =
                    MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::WHITE)));

                board.0[i][j] = Some(
                    commands
                        .spawn((
                            Text2d::new(format!("{}", 0)),
                            TextFont::default(),
                            TextLayout::new_with_justify(JustifyText::Justified),
                            Transform::from_xyz(x, y, 0.0).with_scale(Vec3::ZERO),
                            TextColor::BLACK,
                            Index { i, j },
                            Value(1),
                            AnimateSpawn,
                            square,
                            white_material,
                        ))
                        .id(),
                );
            } else {
                // game over? not necessarily, could still merge some tiles
            }
        }
    }
}

fn new_board(
    mut commands: Commands,
    mut events: EventReader<Reset>,
    mut board: ResMut<Board>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut rng: GlobalEntropy<WyRand>,
    dims: Res<Dims>,
) {
    if !events.is_empty() {
        events.clear();

        board
            .0
            .iter()
            .flatten()
            .filter_map(|opt| *opt)
            .for_each(|e| commands.entity(e).despawn());

        board.0 = [[None; SQUARES_Y]; SQUARES_X];

        let first_ones = (0..SQUARES_X)
            .cartesian_product(0..SQUARES_Y)
            .choose_multiple(&mut rng, 2);

        for (i, j) in first_ones {
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

            board.0[i][j] = Some(
                commands
                    .spawn((
                        Text2d::new(format!("{}", 0)),
                        TextFont::default(),
                        TextLayout::new_with_justify(JustifyText::Justified),
                        Transform::from_xyz(x, y, 0.0),
                        TextColor::BLACK,
                        Index { i, j },
                        Value(1),
                        square,
                        white_material,
                    ))
                    .id(),
            );
        }
    }
}

fn update_visuals(
    mut commands: Commands,
    texts: Query<(&Value, &mut Text2d)>,
    animate_move: Query<
        (Entity, &Index, &mut Transform),
        (With<AnimateMove>, Without<AnimateSpawn>),
    >,
    animate_spawn: Query<(Entity, &Transform), (With<AnimateSpawn>, Without<AnimateMove>)>,
    animate_merge: Query<
        (Entity, &Index, &Transform),
        (With<AnimateMergeInto>, Without<AnimateMove>),
    >,
    dims: Res<Dims>,
) {
    for (value, mut text) in texts {
        text.0 = format!("{}", value.0);
    }

    for (ent, index, transform) in animate_move {
        let (x_destination, y_destination) = (
            ((index.i as f32) - ((SQUARES_X as f32 - 1.0) / 2.0)) * dims.width / SQUARES_X as f32,
            ((index.j as f32) - ((SQUARES_Y as f32 - 1.0) / 2.0)) * dims.height / SQUARES_Y as f32,
        );

        commands.entity(ent).insert(transform.ease_to(
            Transform::from_xyz(x_destination, y_destination, 0.0),
            EaseMethod::EaseFunction(bevy_easings::EaseFunction::QuadraticIn),
            EasingType::Once {
                duration: Duration::from_secs_f32(0.1),
            },
        ));

        commands.entity(ent).remove::<AnimateMove>();
    }

    for (ent, transform) in animate_spawn {
        commands
            .entity(ent)
            .insert(transform.with_scale(Vec3::ZERO).ease_to(
                transform.with_scale(Vec3::ONE),
                EaseMethod::EaseFunction(bevy_easings::EaseFunction::QuadraticIn),
                EasingType::Once {
                    duration: Duration::from_secs_f32(0.2),
                },
            ));

        commands.entity(ent).remove::<AnimateSpawn>();
    }

    for (ent, index, transform) in animate_merge {
        let (x_destination, y_destination) = (
            ((index.i as f32) - ((SQUARES_X as f32 - 1.0) / 2.0)) * dims.width / SQUARES_X as f32,
            ((index.j as f32) - ((SQUARES_Y as f32 - 1.0) / 2.0)) * dims.height / SQUARES_Y as f32,
        );

        commands.entity(ent).insert(transform.ease_to(
            Transform::from_xyz(x_destination, y_destination, 0.0).with_scale(Vec3::ZERO),
            EaseMethod::EaseFunction(bevy_easings::EaseFunction::QuadraticIn),
            EasingType::Once {
                duration: Duration::from_secs_f32(0.1),
            },
        ));

        commands.entity(ent).remove::<AnimateMergeInto>();
    }
}

fn despawn_later(
    mut commands: Commands,
    query: Query<(Entity, &mut DespawnLater)>,
    time: Res<Time>,
) {
    for (entity, mut timer) in query {
        timer.0.tick(time.delta());

        if timer.0.finished() {
            commands.entity(entity).despawn();
        }
    }
}
