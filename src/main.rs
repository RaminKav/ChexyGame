use bevy::{asset, prelude::*, sprite::Anchor, window::PrimaryWindow};
use bevy_rapier2d::{
    control::KinematicCharacterController,
    dynamics::RigidBody,
    geometry::{Collider, Sensor},
    na::Translation,
    pipeline::{CollisionEvent, QueryFilterFlags},
    plugin::{NoUserData, RapierConfiguration, RapierContext, RapierPhysicsPlugin},
    render::RapierDebugRenderPlugin,
};
use rand::Rng;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .insert_resource(CursorPos::default())
        .insert_resource(CurrentCard(0))
        .insert_resource(SpawnTimer(Timer::from_seconds(3., TimerMode::Repeating)))
        .insert_resource(MonthTimer(Timer::from_seconds(75., TimerMode::Repeating)))
        // .insert_resource(RapierConfiguration {
        //     gravity: Vec2::new(0., -300.),
        //     ..Default::default()
        // })
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        // .add_plugin(RapierDebugRenderPlugin::default())
        .add_startup_system(setup)
        .add_systems((
            handle_inputs,
            update_cursor_pos,
            handle_despawn_timers,
            handle_velocity,
            tick_month,
            handle_proj_collisions,
            spawn_random_enemies,
            enemy_movement,
            handle_health_change,
            handle_update_money_text,
            handle_collisions
                .before(handle_velocity)
                .before(handle_inputs),
            move_projectiles,
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
pub struct Projectile {
    direction: Vec2,
    damage: f32,
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
const ENEMY_JUMP_FORCE: f32 = 100.0;

#[derive(Component)]
pub struct EnemyDirection(f32, Timer);

#[derive(Component)]
pub struct JumpTimer(Timer);

#[derive(Component)]
pub struct HPBar;
#[derive(Component)]

pub struct MoneyText(pub i32, pub i32);
#[derive(Component)]

pub struct DayText;

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Camera
    commands.spawn(Camera2dBundle::default());
    // Background
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            // color: Color::rgb(0.25, 0.25, 0.75),
            custom_size: Some(Vec2::new(1290.0, 750.0)),
            ..default()
        },
        texture: asset_server.load("city-background.png"),
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, 0.0), // Ensure the background is behind all entities
            ..default()
        },
        ..default()
    });
    // Rectangle
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                // color: Color::rgb(0.25, 0.25, 0.75),
                custom_size: Some(Vec2::new(100.0, 100.0)),
                ..default()
            },
            texture: asset_server.load("chester.png"),

            transform: Transform::from_translation(Vec3::new(-50., -300., 1.)),
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
            transform: Transform::from_translation(Vec3::new(-490., 340., 1.)),
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
    commands.spawn((
        // Create a TextBundle that has a Text with a single section.
        TextBundle::from_section(
            // Accepts a `String` or any type that converts into a `String`, such as `&str`
            "Day 1",
            TextStyle {
                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font_size: 40.0,
                color: Color::ORANGE_RED,
            },
        ) // Set the alignment of the Text
        .with_text_alignment(TextAlignment::Left)
        // Set the style of the TextBundle itself.
        .with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                top: Val::Px(5.0),
                left: Val::Px(600.0),
                ..default()
            },
            ..default()
        }),
        DayText,
    ));

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
        MoneyText(0, 2800),
    ));

    // Rectangle - Enemy
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(100.0, 100.0)),
                ..default()
            },
            texture: asset_server.load("bill-asset.png"),
            transform: Transform::from_translation(Vec3::new(0.0, -305., 1.)),
            ..default()
        })
        // .insert(RigidBody::Dynamic)
        .insert(Collider::cuboid(50.0, 50.0))
        .insert(Velocity::default())
        .insert(Enemy)
        .insert(MaxHealth(50.0))
        .insert(CurrentHealth(50.0))
        .insert(Sensor)
        .insert(EnemyDirection(
            1.0,
            Timer::from_seconds(2., TimerMode::Repeating),
        ))
        .insert(JumpTimer(Timer::from_seconds(10.0, TimerMode::Repeating)));

    // Platform
    commands
        .spawn(Collider::cuboid(PLATFORM_SIZE.x / 2., PLATFORM_SIZE.y / 2.))
        .insert(TransformBundle::from_transform(
            Transform::from_translation(Vec3::new(-200., -380., 0.)),
        ))
        .insert(Sensor);
    // commands
    //     .spawn(SpriteBundle {
    //         sprite: Sprite {
    //             color: Color::rgb(0.75, 0.25, 0.25),
    //             custom_size: Some(SMALL_PLATFORM_SIZE),
    //             ..default()
    //         },
    //         transform: Transform::from_translation(Vec3::new(-200., 150., 0.)),
    //         ..default()
    //     })
    //     .insert(Collider::cuboid(
    //         SMALL_PLATFORM_SIZE.x / 2.,
    //         SMALL_PLATFORM_SIZE.y / 2.,
    //     ))
    //     .insert(Sensor);
    // commands
    //     .spawn(SpriteBundle {
    //         sprite: Sprite {
    //             color: Color::rgb(0.75, 0.25, 0.25),
    //             custom_size: Some(SMALL_PLATFORM_SIZE),
    //             ..default()
    //         },
    //         transform: Transform::from_translation(Vec3::new(200., 200., 0.)),
    //         ..default()
    //     })
    //     .insert(Collider::cuboid(
    //         SMALL_PLATFORM_SIZE.x / 2.,
    //         SMALL_PLATFORM_SIZE.y / 2.,
    //     ))
    //     .insert(Sensor);
}

#[derive(Resource)]
pub struct CurrentCard(pub i32);

pub fn handle_inputs(
    mut player_query: Query<
        (Entity, &mut Velocity, &mut Gravity, &mut Sprite, &Transform),
        With<Player>,
    >,
    time: Res<Time>,
    key_input: ResMut<Input<KeyCode>>,
    mouse_input: ResMut<Input<MouseButton>>,
    mut commands: Commands,
    cur: Res<CursorPos>,
    asset_server: Res<AssetServer>,
    mut money: Query<&mut MoneyText>,
    mut card: ResMut<CurrentCard>,
) {
    let (player_e, mut vel, mut grav, mut sprite, transform) = player_query.single_mut();
    let mut d = Vec2::ZERO;
    let speed = 200.;
    let s = speed * time.delta_seconds();

    if key_input.pressed(KeyCode::A) {
        d.x -= 1.;
        if vel.0.x > 0. {
            vel.0.x = 0.;
        }
        sprite.flip_x = false;
    }
    if key_input.pressed(KeyCode::D) {
        d.x += 1.;
        if vel.0.x < 0. {
            vel.0.x = 0.;
        }
        sprite.flip_x = true;
    }
    if key_input.pressed(KeyCode::W) {
        println!("W");
        // d.y += 1.;
    }
    if key_input.pressed(KeyCode::S) {
        d.y -= 1.;
    }

    if key_input.pressed(KeyCode::Q) {
        println!("UPGRADE");
        let mut money = money.single_mut();
        if money.0 >= 1000 {
            money.0 -= 1000;
            card.0 = 1;
        }
    }

    if !key_input.pressed(KeyCode::A) && !key_input.pressed(KeyCode::D) {
        vel.0.x = 0.;
    }
    if mouse_input.just_pressed(MouseButton::Left) {
        // Create a small entity used as a projectile
        let direction =
            (cur.world_coords.truncate() - transform.translation.truncate()).normalize();
        let (damage, asset) = if card.0 == 0 {
            (10., "credit-card-projectile.png")
        } else {
            (15., "credit-card-projectile2.png")
        };

        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    // color: Color::rgb(1.0, 0.0, 0.0),
                    custom_size: Some(Vec2::new(70.0, 50.0)),
                    ..default()
                },
                texture: asset_server.load(asset),

                transform: Transform {
                    translation: transform.translation,
                    ..default()
                },
                ..default()
            },
            Collider::cuboid(5.0, 5.0),
            Sensor,
            Projectile { direction, damage },
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
    mut player: Query<
        (Entity, &mut Velocity, &MaxHealth, &mut CurrentHealth),
        (With<Player>, Without<CollidedThisFrame>),
    >,
    enemies: Query<Entity, With<Enemy>>,
    proj: Query<Entity, With<Projectile>>,
    mut commands: Commands,
    context: ResMut<RapierContext>,
) {
    let Ok((player, mut vel, max_hp, mut curr_hp)) = player.get_single_mut() else {
        return;
    };

    let hits_this_frame = context.intersection_pairs().filter(|c| {
        ((c.0 == player) || (c.1 == player)) && proj.get(c.0).is_err() && proj.get(c.1).is_err()
    });
    for (e1, e2, _) in hits_this_frame {
        if (e1 == player && enemies.get(e2).is_ok()) || (e2 == player && enemies.get(e1).is_ok()) {
            curr_hp.0 -= 10.;
            println!("HIT ENEMY {:?}", curr_hp.0);
        }
        commands
            .entity(player)
            .insert(CollidedThisFrame(Timer::from_seconds(0.1, TimerMode::Once)));
    }
}

pub fn handle_proj_collisions(
    mut enemies: Query<(Entity, &mut CurrentHealth), With<Enemy>>,
    projs: Query<Entity, With<Projectile>>,
    mut commands: Commands,
    context: ResMut<RapierContext>,
    card: Res<CurrentCard>,
) {
    for (e1, e2, _) in context.intersection_pairs() {
        for (enemy, mut curr_hp) in enemies.iter_mut() {
            if (e1 == enemy && projs.get(e2).is_ok()) || (e2 == enemy && projs.get(e1).is_ok()) {
                if projs.get(e1).is_ok() {
                    commands.entity(e1).despawn();
                } else {
                    commands.entity(e2).despawn();
                };

                curr_hp.0 -= if card.0 == 0 { 10. } else { 15. };

                println!("HIT ENEMY {:?}", curr_hp.0);
            }
        }
    }
}

pub fn enemy_movement(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut EnemyDirection, &mut JumpTimer), With<Enemy>>,
) {
    for (mut transform, mut direction, mut jump_timer) in query.iter_mut() {
        let delta_move = ENEMY_SPEED * direction.0 * time.delta_seconds();
        transform.translation.x += delta_move;

        direction.1.tick(time.delta());

        if direction.1.finished() {
            direction.0 *= -1.0;
        }

        jump_timer.0.tick(time.delta());

        if jump_timer.0.finished() {
            transform.translation.y = ENEMY_JUMP_FORCE;

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
        transform.translation += Vec3::new(projectile.direction.x, projectile.direction.y, 0.0)
            * velocity.0.length()
            * time.delta_seconds();
    }
}

pub fn handle_health_change(
    query: Query<
        (
            Entity,
            &Transform,
            &MaxHealth,
            &CurrentHealth,
            Option<&Player>,
        ),
        Changed<CurrentHealth>,
    >,
    mut hp_bar: Query<&mut Sprite, With<HPBar>>,
    mut money: Query<&mut MoneyText>,
    mut commands: Commands,
    card: Res<CurrentCard>,
    asset_server: Res<AssetServer>,
) {
    for (entity, t, max_hp, curr_hp, player_option) in query.iter() {
        println!("HP: {}/{}", curr_hp.0, max_hp.0);
        if player_option.is_some() {
            for mut sprite in hp_bar.iter_mut() {
                sprite.custom_size = Some(Vec2::new(200.0 * (curr_hp.0 / max_hp.0), 10.));
            }
        } else {
            if curr_hp.0 <= 0. {
                println!("KILL!");
                commands.entity(entity).despawn_recursive();
                money.single_mut().0 += if card.0 == 0 { 100 } else { 150 };

                commands
                    .spawn((
                        DespawnTimer(Timer::from_seconds(1., TimerMode::Once)),
                        // Create a TextBundle that has a Text with a single section.
                        TextBundle::from_section(
                            // Accepts a `String` or any type that converts into a `String`, such as `&str`
                            format!("+${}", if card.0 == 0 { 100 } else { 150 }),
                            TextStyle {
                                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 40.0,
                                color: Color::GREEN,
                            },
                        ) // Set the alignment of the Text
                        .with_text_alignment(TextAlignment::Left)
                        // Set the style of the TextBundle itself.
                        .with_style(Style {
                            position_type: PositionType::Absolute,
                            position: UiRect {
                                top: Val::Px(200.0),
                                left: Val::Px(300.0),
                                ..default()
                            },
                            ..default()
                        }),
                    ))
                    .insert(t.clone());
            }
        }
    }
}

pub fn handle_update_money_text(mut query: Query<(&MoneyText, &mut Text), Changed<MoneyText>>) {
    for (money, mut text) in query.iter_mut() {
        text.sections[0].value = format!("RENT: ${} Cash: ${}", money.1, money.0);
    }
}

#[derive(Default, Resource, Debug)]
pub struct CursorPos {
    pub world_coords: Vec3,
    pub screen_coords: Vec3,
    pub ui_coords: Vec3,
}

pub fn update_cursor_pos(
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Transform, &Camera)>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut cursor_pos: ResMut<CursorPos>,
) {
    for cursor_moved in cursor_moved_events.iter() {
        // To get the mouse's world position, we have to transform its window position by
        // any transforms on the camera. This is done by projecting the cursor position into
        // camera space (world space).
        for (cam_t, cam) in camera_q.iter() {
            *cursor_pos = CursorPos {
                world_coords: cursor_pos_in_world(&windows, cursor_moved.position, cam_t, cam),
                ui_coords: cursor_pos_in_ui(&windows, cursor_moved.position, cam),
                screen_coords: cursor_moved.position.extend(0.),
            };
        }
        // println!("cur {:?}", cursor_pos.screen_coords);
    }
}

pub fn cursor_pos_in_world(
    windows: &Query<&Window, With<PrimaryWindow>>,
    cursor_pos: Vec2,
    cam_t: &Transform,
    cam: &Camera,
) -> Vec3 {
    let window = windows.single();

    let window_size = Vec2::new(window.width(), window.height());

    // Convert screen position [0..resolution] to ndc [-1..1]
    // (ndc = normalized device coordinates)
    let ndc_to_world = cam_t.compute_matrix() * cam.projection_matrix().inverse();
    let ndc = (cursor_pos / window_size) * 2.0 - Vec2::ONE;
    ndc_to_world.project_point3(ndc.extend(0.0))
}
pub fn cursor_pos_in_ui(
    windows: &Query<&Window, With<PrimaryWindow>>,
    cursor_pos: Vec2,
    cam: &Camera,
) -> Vec3 {
    let window = windows.single();

    let window_size = Vec2::new(window.width(), window.height());

    // Convert screen position [0..resolution] to ndc [-1..1]
    // (ndc = normalized device coordinates)
    let t = Transform::from_translation(Vec3::new(0., 0., 0.));
    let ndc_to_world = t.compute_matrix() * cam.projection_matrix().inverse();
    let ndc = (cursor_pos / window_size) * 2.0 - Vec2::ONE;
    ndc_to_world.project_point3(ndc.extend(0.0))
}

#[derive(Resource)]
pub struct SpawnTimer(Timer);
#[derive(Resource)]
pub struct MonthTimer(Timer);

pub fn spawn_random_enemies(
    time: Res<Time>,
    mut spawn_timer: ResMut<SpawnTimer>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    spawn_timer.0.tick(time.delta());
    if spawn_timer.0.finished() {
        // Spawn an enemy
        let mut rng = rand::thread_rng();
        let pos = Vec3::new(
            rng.gen_range(-300.0..300.0),
            rng.gen_range(-200.0..300.0),
            1.,
        );
        commands
            .spawn(SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::new(100.0, 100.0)),
                    ..default()
                },
                texture: asset_server.load("bill-asset.png"),
                transform: Transform::from_translation(pos),
                ..default()
            })
            // .insert(RigidBody::Dynamic)
            .insert(Collider::cuboid(50.0, 50.0))
            // .insert(Velocity::default())
            .insert(Enemy)
            .insert(MaxHealth(50.))
            .insert(CurrentHealth(50.))
            .insert(Sensor)
            .insert(EnemyDirection(
                1.0,
                Timer::from_seconds(rng.gen_range(0.3_f32..1.2_f32), TimerMode::Repeating),
            ))
            .insert(JumpTimer(Timer::from_seconds(10.0, TimerMode::Repeating)));
    }
}

#[derive(Component)]
pub struct DespawnTimer(Timer);

pub fn tick_month(
    mut month: ResMut<MonthTimer>,
    mut text: Query<&mut Text, With<DayText>>,
    time: Res<Time>,
    mut rent_tracker: Query<&mut MoneyText>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut player: Query<&mut CurrentHealth, With<Player>>,
) {
    month.0.tick(time.delta());
    for mut t in text.iter_mut() {
        println!(
            "DAY {} {:?}",
            month.0.elapsed().as_secs(),
            f32::floor(month.0.elapsed().as_secs() as f32 / 2.5)
        );
        t.sections[0].value = format!(
            "Day {}",
            f32::floor(month.0.elapsed().as_secs() as f32 / 2.5)
        );
    }
    if month.0.finished() {
        let mut rent = rent_tracker.single_mut();
        if rent.0 < rent.1 {
            commands.spawn((
                DespawnTimer(Timer::from_seconds(5., TimerMode::Once)),
                // Create a TextBundle that has a Text with a single section.
                TextBundle::from_section(
                    // Accepts a `String` or any type that converts into a `String`, such as `&str`
                    "NOT ENOUGH MONEY FOR RENT: CREDIT SCORE LOWERED",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 50.0,
                        color: Color::ORANGE_RED,
                    },
                ) // Set the alignment of the Text
                .with_text_alignment(TextAlignment::Left)
                // Set the style of the TextBundle itself.
                .with_style(Style {
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        top: Val::Px(200.0),
                        left: Val::Px(100.0),
                        ..default()
                    },
                    ..default()
                }),
            ));
            player.single_mut().0 -= 30.;
        } else {
            rent.0 -= rent.1;
            player.single_mut().0 += 30.;
        }
        rent.1 += 200;
    }

    if player.single_mut().0 <= 0. {
        commands.spawn((
            DespawnTimer(Timer::from_seconds(5., TimerMode::Once)),
            // Create a TextBundle that has a Text with a single section.
            TextBundle::from_section(
                // Accepts a `String` or any type that converts into a `String`, such as `&str`
                "CREDIT SCORE TOO LOW: GAME OVER",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 50.0,
                    color: Color::RED,
                },
            ) // Set the alignment of the Text
            .with_text_alignment(TextAlignment::Left)
            // Set the style of the TextBundle itself.
            .with_style(Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: Val::Px(200.0),
                    left: Val::Px(100.0),
                    ..default()
                },
                ..default()
            }),
        ));
    }
}

pub fn handle_despawn_timers(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut DespawnTimer)>,
) {
    for (entity, mut timer) in query.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.finished() {
            println!("despawn");
            commands.entity(entity).despawn_recursive();
        }
    }
}
