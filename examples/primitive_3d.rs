use std::f32::consts::{FRAC_PI_2, PI};

use bevy::{color::palettes, math::vec2, prelude::*, window::PrimaryWindow};
use polyanya::Triangulation;
use rand::Rng;
use vleue_navigator::prelude::*;

#[path = "helpers/agent3d.rs"]
mod agent3d;
#[path = "helpers/ui.rs"]
mod ui;

const MESH_WIDTH: u32 = 150;
const MESH_HEIGHT: u32 = 100;

fn main() {
    App::new()
        .insert_resource(ClearColor(palettes::css::BLACK.into()))
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Navmesh with Polyanya".to_string(),
                    fit_canvas_to_parent: true,
                    ..default()
                }),
                ..default()
            }),
            VleueNavigatorPlugin,
            // Auto update the navmesh.
            // Obstacles will be entities with the `Obstacle` marker component,
            // and use the `Aabb` component as the obstacle data source.
            // NavmeshUpdaterPlugin::<Obstacle, Aabb>::default(),
            NavmeshUpdaterPlugin::<PrimitiveObstacle>::default(),
        ))
        .add_systems(
            Startup,
            (
                setup,
                ui::setup_stats::<true>,
                ui::setup_settings::<false>,
                agent3d::setup_agent::<100>,
            ),
        )
        .add_systems(
            Update,
            (
                spawn_obstacle_on_click.after(ui::update_settings::<10>),
                ui::update_stats::<PrimitiveObstacle>,
                remove_obstacles,
                ui::display_settings,
                ui::update_settings::<10>,
                agent3d::give_target_to_navigator::<MESH_WIDTH, MESH_HEIGHT>,
                agent3d::move_navigator::<100>,
                agent3d::display_navigator_path,
                agent3d::refresh_path::<MESH_WIDTH, MESH_HEIGHT>,
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(
            MESH_WIDTH as f32 / 2.0,
            MESH_WIDTH.min(MESH_HEIGHT) as f32,
            -1.0,
        )
        .looking_at(
            Vec3::new(
                MESH_WIDTH as f32 / 2.0,
                0.0,
                MESH_HEIGHT as f32 / 2.0 - MESH_HEIGHT as f32 / 12.0,
            ),
            Vec3::Y,
        ),
    ));
    // light
    for (x, y) in [(0.25, 0.25), (0.75, 0.25), (0.25, 0.75), (0.75, 0.75)] {
        commands.spawn((
            PointLight {
                shadows_enabled: true,
                intensity: MESH_WIDTH.min(MESH_HEIGHT) as f32 * 300_000.0,
                range: MESH_WIDTH.min(MESH_HEIGHT) as f32 * 10.0,
                ..default()
            },
            Transform::from_xyz(
                MESH_WIDTH as f32 * x,
                MESH_WIDTH.min(MESH_HEIGHT) as f32 / 3.0,
                MESH_HEIGHT as f32 * y,
            ),
        ));
    }

    // Spawn a new navmesh that will be automatically updated.
    commands.spawn((
        NavMeshSettings {
            // Define the outer borders of the navmesh.
            fixed: Triangulation::from_outer_edges(&[
                vec2(0.0, 0.0),
                vec2(MESH_WIDTH as f32, 0.0),
                vec2(MESH_WIDTH as f32, MESH_HEIGHT as f32),
                vec2(0.0, MESH_HEIGHT as f32),
            ]),
            simplify: 0.05,
            merge_steps: 0,
            agent_radius: 1.0,
            ..default()
        },
        // Mark it for update as soon as obstacles are changed.
        // Other modes available are debounced or manual trigger.
        NavMeshUpdateMode::Direct,
        NavMeshDebug(palettes::tailwind::SLATE_800.into()),
        Transform::from_rotation(Quat::from_rotation_x(FRAC_PI_2)),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(
            Vec3::Z,
            Vec2::new(MESH_WIDTH as f32 / 2.0, MESH_HEIGHT as f32 / 2.0),
        ))),
        Transform::from_translation(Vec3::new(
            MESH_WIDTH as f32 / 2.0,
            0.0,
            MESH_HEIGHT as f32 / 2.0,
        ))
        .with_rotation(Quat::from_rotation_x(-FRAC_PI_2)),
        MeshMaterial3d(materials.add(StandardMaterial::from(Color::Srgba(
            palettes::tailwind::BLUE_800,
        )))),
    ));

    // Spawn a few obstacles to start with.
    // They need
    // - the `Obstacle` marker component
    // - the `Aabb` component to define the obstacle's shape
    // - the `Transform` component to define the obstacle's position
    // - the `GlobalTransform` so that it's correctly handled by Bevy
    let mut rng = rand::rng();
    let mat = materials.add(StandardMaterial::from(Color::Srgba(
        palettes::tailwind::RED_700,
    )));
    for _ in 0..40 {
        let transform = Transform::from_translation(Vec3::new(
            rng.random_range(0.0..(MESH_WIDTH as f32)),
            0.0,
            rng.random_range(0.0..(MESH_HEIGHT as f32)),
        ))
        .with_rotation(Quat::from_rotation_y(rng.random_range(0.0..PI)));
        new_obstacle(&mut commands, &mut rng, transform, &mut meshes, &mat);
    }
}

fn new_obstacle(
    commands: &mut Commands,
    rng: &mut impl Rng,
    transform: Transform,
    meshes: &mut Assets<Mesh>,
    mat: &Handle<StandardMaterial>,
) {
    match rng.random_range(0..8) {
        0 => {
            let primitive = Rectangle {
                half_size: vec2(rng.random_range(1.0..5.0), rng.random_range(1.0..5.0)),
            };
            commands
                .spawn((
                    transform,
                    Visibility::Visible,
                    PrimitiveObstacle::Rectangle(primitive),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Mesh3d(meshes.add(Extrusion::new(primitive, rng.random_range(5.0..15.0)))),
                        MeshMaterial3d(mat.clone()),
                        Transform::from_rotation(Quat::from_rotation_x(FRAC_PI_2)),
                    ));
                });
        }
        1 => {
            let primitive = Circle {
                radius: rng.random_range(1.0..5.0),
            };
            commands
                .spawn((
                    transform,
                    Visibility::Visible,
                    PrimitiveObstacle::Circle(primitive),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Mesh3d(meshes.add(Extrusion::new(primitive, rng.random_range(5.0..15.0)))),
                        MeshMaterial3d(mat.clone()),
                        Transform::from_rotation(Quat::from_rotation_x(FRAC_PI_2)),
                    ));
                });
        }
        2 => {
            let primitive = Ellipse {
                half_size: vec2(rng.random_range(1.0..5.0), rng.random_range(1.0..5.0)),
            };
            commands
                .spawn((
                    transform,
                    Visibility::Visible,
                    PrimitiveObstacle::Ellipse(primitive),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Mesh3d(meshes.add(Extrusion::new(primitive, rng.random_range(5.0..15.0)))),
                        MeshMaterial3d(mat.clone()),
                        Transform::from_rotation(Quat::from_rotation_x(FRAC_PI_2)),
                    ));
                });
        }
        3 => {
            let primitive =
                CircularSector::new(rng.random_range(1.5..5.0), rng.random_range(0.5..FRAC_PI_2));
            commands
                .spawn((
                    transform,
                    Visibility::Visible,
                    PrimitiveObstacle::CircularSector(primitive),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Mesh3d(meshes.add(Extrusion::new(primitive, rng.random_range(5.0..15.0)))),
                        MeshMaterial3d(mat.clone()),
                        Transform::from_rotation(Quat::from_rotation_x(FRAC_PI_2)),
                    ));
                });
        }
        4 => {
            let primitive =
                CircularSegment::new(rng.random_range(1.5..5.0), rng.random_range(1.0..PI));
            commands
                .spawn((
                    transform,
                    Visibility::Visible,
                    PrimitiveObstacle::CircularSegment(primitive),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Mesh3d(meshes.add(Extrusion::new(primitive, rng.random_range(5.0..15.0)))),
                        MeshMaterial3d(mat.clone()),
                        Transform::from_rotation(Quat::from_rotation_x(FRAC_PI_2)),
                    ));
                });
        }
        5 => {
            let primitive = Capsule2d::new(rng.random_range(1.0..3.0), rng.random_range(1.5..5.0));
            commands
                .spawn((
                    transform,
                    Visibility::Visible,
                    PrimitiveObstacle::Capsule(primitive),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Mesh3d(meshes.add(Extrusion::new(primitive, rng.random_range(5.0..15.0)))),
                        MeshMaterial3d(mat.clone()),
                        Transform::from_rotation(Quat::from_rotation_x(FRAC_PI_2)),
                    ));
                });
        }
        6 => {
            let primitive = RegularPolygon::new(rng.random_range(1.0..5.0), rng.random_range(3..8));
            commands
                .spawn((
                    transform,
                    Visibility::Visible,
                    PrimitiveObstacle::RegularPolygon(primitive),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Mesh3d(meshes.add(Extrusion::new(primitive, rng.random_range(5.0..15.0)))),
                        MeshMaterial3d(mat.clone()),
                        Transform::from_rotation(Quat::from_rotation_x(FRAC_PI_2)),
                    ));
                });
        }
        7 => {
            let primitive = Rhombus::new(rng.random_range(3.0..6.0), rng.random_range(2.0..3.0));
            commands
                .spawn((
                    transform,
                    Visibility::Visible,
                    PrimitiveObstacle::Rhombus(primitive),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Mesh3d(meshes.add(Extrusion::new(primitive, rng.random_range(5.0..15.0)))),
                        MeshMaterial3d(mat.clone()),
                        Transform::from_rotation(Quat::from_rotation_x(FRAC_PI_2)),
                    ));
                });
        }
        _ => unreachable!(),
    }
}

fn spawn_obstacle_on_click(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    primary_window: Single<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut commands: Commands,
    settings: Single<Ref<NavMeshSettings>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Click was on a UI button that triggered a settings change, ignore it.
    if settings.is_changed() {
        return;
    }
    if mouse_button_input.just_pressed(MouseButton::Left) {
        let Ok((camera, camera_transform)) = camera_q.single() else {
            return;
        };
        let window = *primary_window;
        if let Some(position) = window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
            .and_then(|ray| {
                ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y))
                    .map(|d| (ray, d))
            })
            .map(|(ray, d)| ray.get_point(d))
        {
            let mut rng = rand::rng();
            let mat = materials.add(StandardMaterial::from(Color::Srgba(
                palettes::tailwind::RED_700,
            )));
            let transform = Transform::from_translation(position)
                .with_rotation(Quat::from_rotation_y(rng.random_range(0.0..PI)));
            new_obstacle(&mut commands, &mut rng, transform, &mut meshes, &mat);
            info!("spawning an obstacle at {}", position);
        }
    }
}

fn remove_obstacles(
    obstacles: Query<Entity, With<PrimitiveObstacle>>,
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        for entity in obstacles.iter() {
            commands.entity(entity).despawn();
        }
    }
}
