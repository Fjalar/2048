use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowResolution};

#[derive(Resource)]
struct Board([[u32; 4]; 4]);

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
        .insert_resource(Board([[0; 4]; 4]))
        .run();
}

fn setup(
    mut commands: Commands,
    // asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut board: ResMut<Board>,
) {
    commands.spawn(Camera2d);

    let window = window_query.single().unwrap();
    let (width, height) = (window.width(), window.height());

    for i in 0..4 {
        for j in 0..4 {
            let (x, y) = (
                ((i as f32) - 1.5) * width / 4.0,
                ((j as f32) - 1.5) * height / 4.0,
            );

            let square = Mesh2d(meshes.add(Rectangle {
                half_size: Vec2::new(width / 8.0 - 10.0, height / 8.0 - 10.0),
            }));

            let white_material =
                MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::WHITE)));

            commands
                .spawn((square, white_material, Transform::from_xyz(x, y, 0.0)))
                .with_child((
                    Text2d::new(format!("{}", board.0[i][j])),
                    TextFont::default(),
                    TextLayout::new_with_justify(JustifyText::Justified),
                    Transform::from_scale(Vec3::new(1.0, 1.0, 1.0)),
                    TextColor::BLACK,
                ));
        }
    }
}
