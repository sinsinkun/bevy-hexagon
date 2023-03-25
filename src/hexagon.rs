use bevy::prelude::*;
use bevy::diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin};
use bevy::sprite::MaterialMesh2dBundle;
use bevy::sprite::Mesh2dHandle;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::render::mesh::Indices;

/*
  Quaternerion 2D rotation:

  q.x = sin(theta/2) * axis.x
  q.y = sin(theta/2) * axis.y
  q.z = sin(theta/2) * axis.z
  q.w = cos(theta/2)
*/

// ---- PLUGIN ----
pub struct HexagonPlugin;

impl Plugin for HexagonPlugin {
  fn build(&self, app: &mut App) {
    // imports app from main
    app.add_plugin(FrameTimeDiagnosticsPlugin)
      .insert_resource(MsTimer(Timer::from_seconds(0.01, TimerMode::Repeating))) // roughly 60 fps locked
      .insert_resource(SpawnTimer(Timer::from_seconds(1.0, TimerMode::Repeating))) 
      .add_startup_system(setup)
      .add_startup_system(create_bg)
      .add_system(spawn_walls)
      .add_system(print_fps)
      .add_system(rotate_camera)
      .add_system(move_walls)
      .add_system(player_control);
  }
}

// ---- CONSTANTS ----
const SPEED:f32 = 2.0;
const OFFSET_RADIUS:f32 = 60.0;
const DEG_TO_RAD:f32 = std::f32::consts::PI / 180.0;

// ---- RESOURCES ----
#[derive(Resource)]
struct SpawnTimer(Timer);

#[derive(Resource)]
struct MsTimer(Timer);

// ---- COMPONENTS ----
#[derive(Component)]
struct FpsText;

#[derive(Component)]
struct Player {
  angle: f32
}

#[derive(Component, Debug)]
struct Wall {
  distance: f32, // distance from center (45..450)
  direction: i32 // 0..5
}

// ---- SYSTEMS ----
fn setup(
  mut commands: Commands,
  asset_server: Res<AssetServer>,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<ColorMaterial>>,
) {
  // spawn camera
  commands.spawn(Camera2dBundle::default());

  // spawn UI
  commands.spawn(NodeBundle {
    style: Style {
      size: Size::new(Val::Percent(100.0), Val::Px(100.0)),
      padding: UiRect {
        top: Val::Px(5.0),
        left: Val::Px(5.0),
        ..default()
      },
      ..default()
    },
    // background_color: Color::rgba(0.1, 0.1, 0.1, 0.6).into(),
    ..default()
  }).with_children(|root| {
    // spawn FPS
    root.spawn((TextBundle::from_sections([
      TextSection::new(
        "FPS: ",
        TextStyle {
          font: asset_server.load("fonts/Roboto-Medium.ttf"),
          font_size: 15.0,
          color: Color::WHITE,
        }
      ),
      TextSection::from_style(TextStyle {
        font: asset_server.load("fonts/Roboto-Medium.ttf"),
        font_size: 15.0,
        color: Color::GREEN,
      }),
    ]), FpsText));
  });

  // spawns hexagon
  commands.spawn(MaterialMesh2dBundle {
    mesh: meshes.add(shape::RegularPolygon::new(50., 6).into()).into(),
    material: materials.add(ColorMaterial::from(Color::rgb(0.8, 0.8, 0.8))),
    transform: Transform::from_translation(Vec3::new(0.0, 0.0, 2.0)),
    ..default()
  });

  // spawns triangle (player)
  commands.spawn((
    MaterialMesh2dBundle {
      mesh: meshes.add(create_triangle()).into(),
      material: materials.add(ColorMaterial::from(Color::rgb(0.8, 0.8, 0.8))),
      transform: Transform::from_translation(Vec3::new(0.0, OFFSET_RADIUS, 99.0)),
      ..default()
    }, 
    Player { angle:0.0 }
  ));

}

fn create_bg(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>, 
  mut materials: ResMut<Assets<ColorMaterial>>
) {
  let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
  let x_val = 500.0 * f32::tan(30.0 * std::f32::consts::PI / 180.0);
  mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, 
    vec![
      [0.0, 0.0, 0.0], 
      [-x_val, 500.0, 0.0], 
      [x_val, 500.0, 0.0],
    ]
  );
  mesh.set_indices(Some(Indices::U32(vec![0,1,2,1,2,0])));
  
  let mut hex_collection = Vec::new();
  for i in 0..6 {
    let a = 30.0 + i as f32 * 60.0;

    let rotation = (-a / 2.0) * std::f32::consts::PI / 180.0;
    let mut color = Color::rgba(0.25, 0.25, 0.25, 1.0);
    if i % 2 == 1 {
      color = Color::rgba(0.2, 0.2, 0.2, 1.0);
    }
    hex_collection.push((rotation, color));
  }

  for hex in hex_collection {
    commands.spawn(MaterialMesh2dBundle {
      mesh: meshes.add(mesh.clone()).into(),
      material: materials.add(ColorMaterial::from(hex.1)),
      transform: Transform::from_rotation(Quat::from_xyzw(0.0, 0.0, hex.0.sin(), hex.0.cos())),
      ..default()
    });
  }

}

// create custom triangle mesh
fn create_triangle() -> Mesh {
  let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
  mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, 
    vec![[-8.0, 0.0, 0.0], [0.0, 8.0, 0.0], [8.0, 0.0, 0.0]]
  );
  mesh.set_indices(Some(Indices::U32(vec![0,1,2])));
  mesh
}

fn print_fps(
  diagnostics: Res<Diagnostics>, 
  mut query: Query<&mut Text, With<FpsText>>
) {
  for mut text in &mut query {
    if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
      if let Some(value) = fps.smoothed() {
        // Update the value of the second section
        text.sections[1].value = format!("{value:.2}");
        if value < 30.0 {
          text.sections[1].style.color = Color::RED;
        }
      }
    }
  }
}

fn player_control(
  time: Res<Time>, 
  mut timer: ResMut<MsTimer>,
  keyboard_input: Res<Input<KeyCode>>,
  mut query: Query<(&mut Transform, &mut Player), With<Player>>
) {
  let (mut player_transform, mut player) = query.single_mut();
  let mut direction = 0.0;

  // accept player input
  if keyboard_input.pressed(KeyCode::Left) {
    direction = -1.0;
  }
  if keyboard_input.pressed(KeyCode::Right) {
    direction = 1.0;
  }
  if direction == 0.0 { return };

  // perform movement on tick
  if timer.0.tick(time.delta()).just_finished() {
    // calculate translation
    let mut new_angle = player.angle + direction * SPEED;
    // clamp
    if new_angle > 360.0 {
      new_angle = new_angle - 360.0;
    }
    if new_angle < 0.0 {
      new_angle = new_angle + 360.0;
    }
    // update player component
    player.angle = new_angle;
    // calculate movement
    let angle_rad = new_angle * DEG_TO_RAD;
    player_transform.translation.x = OFFSET_RADIUS * f32::sin(angle_rad);
    player_transform.translation.y = OFFSET_RADIUS * f32::cos(angle_rad);
    // calculate rotation
    let rad_unit = -direction * SPEED * DEG_TO_RAD;
    // println!("Rotation {} -> {} rad -> rad_unit {}", &new_angle, &angle_rad, &rad_unit);
    player_transform.rotate_z(rad_unit);
  }
}

fn rotate_camera(
  time: Res<Time>, 
  mut timer: ResMut<MsTimer>,
  mut query: Query<&mut Transform, With<Camera>>
) {
  if timer.0.tick(time.delta()).just_finished() {
    let mut camera_t = query.single_mut();
    let rad_unit = 0.2 * SPEED * DEG_TO_RAD;
    camera_t.rotate_z(rad_unit);
  }
}

// spawn enemy shapes
fn spawn_walls(
  time: Res<Time>, 
  mut timer: ResMut<SpawnTimer>,
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<ColorMaterial>>,
) {

  // prevent spawn on timer
  if !timer.0.tick(time.delta()).just_finished() {
    return;
  }

  // INPUTS
  let distance = 450.0;
  // let direction = fastrand::i32(0..5);

  let num_of_walls = fastrand::i32(1..5);
  let mut directions = Vec::new();
  for _i in 0..num_of_walls {
    let mut dir = fastrand::i32(0..5);
    while directions.contains(&dir) {
      dir = fastrand::i32(0..5);
    }
    directions.push(dir);
  }
  println!("directions vec {:?}", directions);

  // create walls
  for direction in directions {
    let a = 30.0 + direction as f32 * 60.0;
    // translation vars
    let rotation = (-a / 2.0) * DEG_TO_RAD;
    let rotation_b = a * DEG_TO_RAD;
    let x = distance * rotation_b.sin();
    let y = distance * rotation_b.cos();
    // mesh vars
    let dist_2 = distance + 50.0;
    let x_width = distance * f32::tan(30.0 * DEG_TO_RAD);
    let x_width_2 = dist_2 * f32::tan(30.0 * DEG_TO_RAD);
    // create mesh
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, 
      vec![
        [-x_width, 0.0, 0.0], 
        [-x_width_2, 50.0, 0.0], 
        [x_width_2, 50.0, 0.0], 
        [x_width, 0.0, 0.0]
      ]
    );
    mesh.set_indices(Some(Indices::U32(vec![0,1,2,0,3,2])));
    // spawn mesh
    commands.spawn((MaterialMesh2dBundle {
      mesh: meshes.add(mesh).into(),
      material: materials.add(ColorMaterial::from(Color::rgb(0.96, 0.2, 0.2))),
      transform: Transform::from_translation(Vec3::new(x, y, 1.0))
        .with_rotation(Quat::from_xyzw(0.0, 0.0, rotation.sin(), rotation.cos())),
      ..default()
    }, Wall { distance:distance, direction:direction }));
  }
}

// move enemy shapes toward middle
fn move_walls(
  mut commands: Commands,
  time: Res<Time>, 
  mut timer: ResMut<MsTimer>,
  mut query: Query<(Entity, &mut Transform, &mut Wall, &Mesh2dHandle)>,
  mut meshes: ResMut<Assets<Mesh>>,
) {
  // on tick timer
  if timer.0.tick(time.delta()).just_finished() {
    // get all walls
    for (entity, mut transform, mut wall, handle) in &mut query {
      
      let new_distance = wall.distance - SPEED;
      // delete wall if too close
      if new_distance < 28.0 {
        commands.entity(entity).despawn();
        return;
      }

      // calculate transform vars
      let a = 30.0 + wall.direction as f32 * 60.0;
      let rotation = a * DEG_TO_RAD;

      // mesh transform vars
      let y_width = 10.0 + new_distance * 0.15;
      let dist_2 = new_distance + y_width;
      let x_width = new_distance * f32::tan(30.0 * DEG_TO_RAD);
      let x_width_2 = dist_2 * f32::tan(30.0 * DEG_TO_RAD);

      // fetch mesh
      let mesh = meshes.get_mut(&handle.0);
      // perform transforms
      if mesh.is_some() {
        mesh.unwrap().insert_attribute(Mesh::ATTRIBUTE_POSITION, 
          vec![
            [-x_width, 0.0, 0.0], 
            [-x_width_2, y_width, 0.0], 
            [x_width_2, y_width, 0.0], 
            [x_width, 0.0, 0.0]
          ]
        );
      }
      transform.translation.x = new_distance * rotation.sin();
      transform.translation.y = new_distance * rotation.cos();
      wall.distance = new_distance;
    }
  }
}

// TODO: end state