use bevy::{prelude::*, sprite::Anchor};
use bevy_rapier2d::{
    control::KinematicCharacterController,
    dynamics::RigidBody,
    geometry::{Collider, Sensor},
    pipeline::{CollisionEvent, QueryFilterFlags},
    plugin::{NoUserData, RapierConfiguration, RapierContext, RapierPhysicsPlugin},
    render::RapierDebugRenderPlugin,
};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        // .insert_resource(RapierConfiguration {
        //     gravity: Vec2::new(0., -300.),
        //     ..Default::default()
        // })
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_startup_system(setup)
        .add_systems((
            handle_inputs,
            handle_velocity,
            enemy_movement,
            handle_health_change,
            handle_update_money_text,
            handle_collisions
                .before(handle_velocity)
                .before(handle_inputs),
            move_projectiles
        ));

    app.run();
}
const GRAVITY: f32 = -200.0;
const PLATFORM_SIZE: Vec2 = Vec2::new(20000.0, 50.0);
const SMALL_PLATFORM_SIZE: Vec2 = Vec2::new(300.0, 50.0);
const MAX_PLAYER_VEL: f32 = 500.;
#[derive(Component)]
pub struct Player;

#[derive(Component)]
struct Projectile {
    direction: Vec2,
}
#[derive(Component)]
pub struct MaxHealth(pub f32);

#[derive(Component)]
pub struct CurrentHealth(pub f32);
#[derive(Component)]
pub struct CollidedThisFrame(pub Timer);

#[derive(Component)]

pub struct Gravity(pub Vec2);

#[derive(Component, Default)]
pub struct Velocity(pub Vec2);

// ENEMY STUFF
#[derive(Component)]
pub struct Enemy;

const ENEMY_SPEED: f32 = 200.0;
const ENEMY_X_RANGE: f32 = 100.0;
const ENEMY_JUMP_FORCE: f32 = 300.0;

#[derive(Component)]
pub struct EnemyDirection(f32);

#[derive(Component)]
pub struct JumpTimer(Timer);

#[derive(Component)]
pub struct HPBar;
#[derive(Component)]

pub struct MoneyText(pub i32);

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
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
        // .insert(RigidBody::Dynamic)
        .insert(Collider::cuboid(25.0, 50.0))
        .insert(Velocity::default())
        .insert(Gravity(Vec2::new(0., GRAVITY)))
        .insert(Player)
        .insert(MaxHealth(100.0))
        .insert(CurrentHealth(100.0));

    // PLAYER HP BAR
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.9, 0.1, 0.1),
                custom_size: Some(Vec2::new(200.0, 10.0)),
                anchor: Anchor::CenterLeft,
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(-490., 340., 0.)),
            ..default()
        })
        .insert(HPBar);

    commands.spawn((
        // Create a TextBundle that has a Text with a single section.
        TextBundle::from_section(
            // Accepts a `String` or any type that converts into a `String`, such as `&str`
            "Credit Score",
            TextStyle {
                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font_size: 30.0,
                color: Color::WHITE,
            },
        ) // Set the alignment of the Text
        .with_text_alignment(TextAlignment::Left)
        // Set the style of the TextBundle itself.
        .with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                top: Val::Px(5.0),
                left: Val::Px(5.0),
                ..default()
            },
            ..default()
        }),
    ));
    // money text
    // commands.spawn((
    //     // Create a TextBundle that has a Text with a single section.
    //     TextBundle::from_section(
    //         // Accepts a `String` or any type that converts into a `String`, such as `&str`
    //         "$",
    //         TextStyle {
    //             font: asset_server.load("fonts/FiraSans-Bold.ttf"),
    //             font_size: 20.0,
    //             color: Color::GOLD,
    //         },
    //     ) // Set the alignment of the Text
    //     .with_text_alignment(TextAlignment::Left)
    //     // Set the style of the TextBundle itself.
    //     .with_style(Style {
    //         position_type: PositionType::Absolute,
    //         position: UiRect {
    //             top: Val::Px(5.0),
    //             right: Val::Px(30.0),
    //             ..default()
    //         },
    //         ..default()
    //     }),
    //     MoneyText(0),
    // ));
    commands.spawn((
        // Create a TextBundle that has a Text with a single section.
        TextBundle::from_section(
            // Accepts a `String` or any type that converts into a `String`, such as `&str`
            "$0",
            TextStyle {
                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font_size: 20.0,
                color: Color::GOLD,
            },
        ) // Set the alignment of the Text
        .with_text_alignment(TextAlignment::Left)
        // Set the style of the TextBundle itself.
        .with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                top: Val::Px(5.0),
                right: Val::Px(10.0),
                ..default()
            },
            ..default()
        }),
        MoneyText(0),
    ));

    // Rectangle - Enemy
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.5, 0.0, 0.0),
                custom_size: Some(Vec2::new(100.0, 100.0)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, -125., 0.)),
            ..default()
        })
        // .insert(RigidBody::Dynamic)
        .insert(Collider::cuboid(50.0, 50.0))
        .insert(Velocity::default())
        .insert(Enemy)
        .insert(Sensor)
        .insert(EnemyDirection(1.0))
        .insert(JumpTimer(Timer::from_seconds(10.0, TimerMode::Repeating)));

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
        .insert(Collider::cuboid(PLATFORM_SIZE.x / 2., PLATFORM_SIZE.y / 2.))
        .insert(Sensor);
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
        ))
        .insert(Sensor);
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
        ))
        .insert(Sensor);
}

pub fn handle_inputs(
    mut player_query: Query<(Entity, &mut Velocity, &mut Gravity, &Transform), (With<Player>,)>,
    time: Res<Time>,
    key_input: ResMut<Input<KeyCode>>,
    mut commands: Commands,
) {
    let (player_e, mut vel, mut grav, transform) = player_query.single_mut();
    let mut d = Vec2::ZERO;
    let speed = 300.;
    let s = speed * time.delta_seconds();

    if key_input.pressed(KeyCode::A) {
        d.x -= 1.;
    }
    if key_input.pressed(KeyCode::D) {
        d.x += 1.;
    }
    if key_input.pressed(KeyCode::W) {
        println!("W");
        // d.y += 1.;
    }
    if key_input.pressed(KeyCode::S) {
        d.y -= 1.;
    }
    if key_input.just_pressed(KeyCode::L) {
        // Create a small entity used as a projectile
        let direction = if key_input.pressed(KeyCode::A) {
            Vec2::new(-1.0, 0.0)
        } else {
            Vec2::new(1.0, 0.0)
        };

        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(1.0, 0.0, 0.0),
                    custom_size: Some(Vec2::new(10.0, 10.0)),
                    ..default()
                },
                transform: Transform {
                    translation: transform.translation,
                    ..default()
                },
                ..default()
            },
            Projectile { direction },
            Velocity(direction * 300.0), // Set the projectile direction and speed
        ));
    }
    if key_input.just_pressed(KeyCode::Space) {
        vel.0.y = 150.;
    }

    if d.x != 0. || d.y != 0. {
        d = d.normalize() * s;
    }

    if vel.0.y > 0. {
        grav.0 = Vec2::new(0., GRAVITY);
    }
    vel.0 += d;

    if vel.0.x >= MAX_PLAYER_VEL {
        vel.0.x = MAX_PLAYER_VEL;
    }
    if vel.0.x <= -MAX_PLAYER_VEL {
        vel.0.x = -MAX_PLAYER_VEL;
    }
    if vel.0.y >= MAX_PLAYER_VEL {
        vel.0.y = MAX_PLAYER_VEL;
    }
    // println!("{s:?} ||| {d:?}, {:?}", vel.0);

    // if d.x != 0. || d.y != 0. {
    //     player_kcc.translation = Some(vel.0);
    // }
}

pub fn handle_velocity(
    mut player_query: Query<
        (
            Entity,
            &mut KinematicCharacterController,
            &mut Velocity,
            &mut Gravity,
            Option<&mut CollidedThisFrame>,
        ),
        (With<Player>,),
    >,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (e, mut transform, mut vel, mut grav, mut collided) in player_query.iter_mut() {
        if let Some(mut collider) = collided {
            if collider.0.percent() == 0. {
                grav.0 = Vec2::ZERO;
                collider.0.tick(time.delta());
                // continue;
            }
            collider.0.tick(time.delta());
            if collider.0.finished() {
                grav.0 = Vec2::new(0., GRAVITY);

                commands.entity(e).remove::<CollidedThisFrame>();
            }
        }
        let mut new_vel = vel.0 + grav.0 * time.delta_seconds();
        if new_vel.y <= grav.0.y * 2. {
            new_vel.y = grav.0.y * 2.;
        }
        vel.0 = new_vel;
        // println!("MOVING {:?} {:?}", new_vel, grav.0);
        transform.translation = Some(new_vel * time.delta_seconds());
    }
}

pub fn handle_collisions(
    mut collisions: EventReader<CollisionEvent>,
    mut player: Query<
        (Entity, &mut Velocity, &MaxHealth, &mut CurrentHealth),
        (With<Player>, Without<CollidedThisFrame>),
    >,
    colliders: Query<Entity, With<Collider>>,
    enemies: Query<Entity, With<Enemy>>,
    mut commands: Commands,
    context: ResMut<RapierContext>,
) {
    let Ok((player, mut vel, max_hp, mut curr_hp)) = player.get_single_mut() else {
        return;
    };

    let hits_this_frame = context
        .intersection_pairs()
        .filter(|c| (c.0 == player) || (c.1 == player));
    for (e1, e2, _) in hits_this_frame {
        // println!("{:?}", hit);
        // vel.0 = Vec2::ZERO;
        if (e1 == player && enemies.get(e2).is_ok()) || (e2 == player && enemies.get(e1).is_ok()) {
            curr_hp.0 -= 1.;
            println!("HIT ENEMY {:?}", curr_hp.0);
        }
        commands
            .entity(player)
            .insert(CollidedThisFrame(Timer::from_seconds(0.1, TimerMode::Once)));
    }
}

pub fn enemy_movement(
    time: Res<Time>,
    mut query: Query<
        (
            &mut Transform,
            &mut Velocity,
            &mut EnemyDirection,
            &mut JumpTimer,
        ),
        With<Enemy>,
    >,
) {
    for (mut transform, mut velocity, mut direction, mut jump_timer) in query.iter_mut() {
        let delta_move = ENEMY_SPEED * direction.0 * time.delta_seconds();
        transform.translation.x += delta_move;

        if transform.translation.x >= ENEMY_X_RANGE || transform.translation.x <= -ENEMY_X_RANGE {
            direction.0 *= -1.0;
        }

        jump_timer.0.tick(time.delta());

        if jump_timer.0.finished() {
            velocity.0.y = ENEMY_JUMP_FORCE;

            jump_timer.0.reset();
        }
    }
}

// Add the system to move projectiles
pub fn move_projectiles(
    mut projectile_query: Query<(&mut Transform, &Velocity, &Projectile), With<Projectile>>,
    time: Res<Time>,
) {
    for (mut transform, velocity, projectile) in projectile_query.iter_mut() {
        transform.translation += Vec3::new(projectile.direction.x, projectile.direction.y, 0.0) * velocity.0.length() * time.delta_seconds();
    }
}

pub fn handle_health_change(
    query: Query<(Entity, &MaxHealth, &CurrentHealth, Option<&Player>), Changed<CurrentHealth>>,
    mut hp_bar: Query<&mut Sprite, With<HPBar>>,
    mut money: Query<&mut MoneyText>,
) {
    for (entity, max_hp, curr_hp, player_option) in query.iter() {
        println!("HP: {}/{}", curr_hp.0, max_hp.0);
        if player_option.is_some() {
            for mut sprite in hp_bar.iter_mut() {
                sprite.custom_size = Some(Vec2::new(200.0 * (curr_hp.0 / max_hp.0), 10.));
            }
        } else {
            if curr_hp.0 <= 0. {
                println!("KILL!");
                money.single_mut().0 += 100;
            }
        }
    }
}

pub fn handle_update_money_text(
    query: Query<&MoneyText, Changed<MoneyText>>,
    mut text: Query<&mut Text>,
) {
    for money in query.iter() {
        for mut text in text.iter_mut() {
            text.sections[0].value = format!("${}", money.0);
        }
    }
}
