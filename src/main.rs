use bevy::prelude::*;
use bevy_rapier2d::{
    control::KinematicCharacterController,
    dynamics::RigidBody,
    geometry::Collider,
    pipeline::QueryFilterFlags,
    plugin::{NoUserData, RapierConfiguration, RapierPhysicsPlugin},
    render::RapierDebugRenderPlugin,
};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .insert_resource(RapierConfiguration {
            gravity: Vec2::new(0., -300.),
            ..Default::default()
        })
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_startup_system(setup)
        .add_systems((handle_inputs, handle_velocity));

    app.run();
}
const PLATFORM_SIZE: Vec2 = Vec2::new(2000.0, 50.0);
const SMALL_PLATFORM_SIZE: Vec2 = Vec2::new(300.0, 50.0);
#[derive(Component)]
pub struct Player;

#[derive(Component, Default)]
pub struct Velocity(pub Vec2);

pub fn setup(mut commands: Commands) {
    // Camera
    commands.spawn(Camera2dBundle::default());

    // Rectangle
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.25, 0.25, 0.75),
                custom_size: Some(Vec2::new(50.0, 100.0)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(-50., 0., 0.)),
            ..default()
        })
        .insert(KinematicCharacterController {
            // The character offset is set to 0.01.
            // offset: CharacterLength::Absolute(0.01),
            filter_flags: QueryFilterFlags::EXCLUDE_SENSORS,
            ..default()
        })
        .insert(RigidBody::Dynamic)
        .insert(Collider::cuboid(25.0, 50.0))
        .insert(Velocity::default())
        .insert(Player);

    // Platform
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.75, 0.25, 0.25),
                custom_size: Some(PLATFORM_SIZE),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0., -200., 0.)),
            ..default()
        })
        .insert(Collider::cuboid(PLATFORM_SIZE.x / 2., PLATFORM_SIZE.y / 2.));
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.75, 0.25, 0.25),
                custom_size: Some(SMALL_PLATFORM_SIZE),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(-200., 150., 0.)),
            ..default()
        })
        .insert(Collider::cuboid(
            SMALL_PLATFORM_SIZE.x / 2.,
            SMALL_PLATFORM_SIZE.y / 2.,
        ));
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.75, 0.25, 0.25),
                custom_size: Some(SMALL_PLATFORM_SIZE),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(200., 200., 0.)),
            ..default()
        })
        .insert(Collider::cuboid(
            SMALL_PLATFORM_SIZE.x / 2.,
            SMALL_PLATFORM_SIZE.y / 2.,
        ));
}

pub fn handle_inputs(
    mut player_query: Query<(Entity, &mut Velocity), (With<Player>,)>,
    time: Res<Time>,
    key_input: ResMut<Input<KeyCode>>,
    mut commands: Commands,
) {
    let (player_e, mut vel) = player_query.single_mut();
    let mut d = Vec2::ZERO;
    let speed = 500.;
    let s = speed * time.delta_seconds();
    let grav = Vec2::new(0., -200.);

    if key_input.pressed(KeyCode::A) {
        d.x -= 1.;
    }
    if key_input.pressed(KeyCode::D) {
        d.x += 1.;
    }
    if key_input.pressed(KeyCode::W) {
        d.y += 1.;
    }
    if key_input.pressed(KeyCode::S) {
        d.y -= 1.;
    }
    if key_input.just_pressed(KeyCode::Space) {
        vel.0.y += 300.;
    }

    if d.x != 0. || d.y != 0. {
        d = d.normalize() * s;
    }
    println!("{s:?} ||| {:?}, {grav:?}, {:?}", d, vel.0);

    vel.0 += d + grav * time.delta_seconds();
    if vel.0.y <= grav.y * 2. {
        vel.0.y = grav.y * 2.;
    }

    // if d.x != 0. || d.y != 0. {
    //     player_kcc.translation = Some(vel.0);
    // }
}

pub fn handle_velocity(
    mut player_query: Query<(&mut KinematicCharacterController, &Velocity), (With<Player>,)>,
    time: Res<Time>,
) {
    for (mut transform, vel) in player_query.iter_mut() {
        transform.translation = Some(vel.0 * time.delta_seconds());
    }
}
