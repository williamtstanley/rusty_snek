use bevy::prelude::*;
use bevy::render::pass::ClearColor;
use rand::prelude::random;
use std::time::Duration;

const ARENA_WIDTH: u32 = 20;
const ARENA_HEIGHT: u32 = 20;

#[derive(Debug, PartialEq, Clone, Copy)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
}
impl Direction {
    fn opposite(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Up => Self::Down,
            Self::Down => Self::Up,
        }
    }
}

#[derive(Default, Clone, Copy, Eq, PartialEq, Hash)]
struct Position {
    x: i32,
    y: i32,
}

struct Size {
    width: f32,
    heigth: f32,
}
impl Size {
    pub fn square(x: f32) -> Self {
        Self {
            width: x,
            heigth: x,
        }
    }
}

struct FoodSpawnerTimer(Timer);
impl Default for FoodSpawnerTimer {
    fn default() -> Self {
        Self(Timer::new(Duration::from_millis(1000), true))
    }
}

struct SnekMoveTimer(Timer);

struct Materials {
    head_material: Handle<ColorMaterial>,
    food_material: Handle<ColorMaterial>,
    segment_material: Handle<ColorMaterial>,
}
struct SnekHead {
    direction: Direction,
}
struct Food;
struct SnekSegment;
struct GrowthEvent;
struct GameOverEvent;

#[derive(Default)]
struct SnekSegments(Vec<Entity>);
#[derive(Default)]
struct LastTailPosition(Option<Position>);
#[derive(Default)]
struct PreviousDirection(Option<Direction>);

fn setup(commands: &mut Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn(Camera2dBundle::default());
    commands.insert_resource(Materials {
        head_material: materials.add(Color::rgb(0.54, 0.21, 0.06).into()),
        food_material: materials.add(Color::rgb(1.0, 0.0, 1.0).into()),
        segment_material: materials.add(Color::rgba(0.54, 0.21, 0.06, 0.7).into()),
    });
}

fn spawn_segment(
    commands: &mut Commands,
    material: &Handle<ColorMaterial>,
    position: Position,
) -> Entity {
    commands
        .spawn(SpriteBundle {
            material: material.clone(),
            ..Default::default()
        })
        .with(SnekSegment)
        .with(position)
        .with(Size::square(0.75))
        .current_entity()
        .unwrap()
}

fn spawn_snek(
    commands: &mut Commands,
    materials: Res<Materials>,
    mut segments: ResMut<SnekSegments>,
) {
    segments.0 = vec![
        commands
            .spawn(SpriteBundle {
                material: materials.head_material.clone(),
                sprite: Sprite::new(Vec2::new(10.0, 10.0)),
                ..Default::default()
            })
            .with(SnekHead {
                direction: Direction::Up,
            })
            .with(SnekSegment)
            .with(Position { x: 10, y: 6 })
            .with(Size::square(0.9))
            .current_entity()
            .unwrap(),
        spawn_segment(
            commands,
            &materials.segment_material,
            Position { x: 10, y: 5 },
        ),
    ]
}

fn snek_movement(
    keyboard_input: Res<Input<KeyCode>>,
    snek_timer: ResMut<SnekMoveTimer>,
    segments: ResMut<SnekSegments>,
    mut last_tail_position: ResMut<LastTailPosition>,
    mut previous_direction: ResMut<PreviousDirection>,
    mut game_over_events: ResMut<Events<GameOverEvent>>,
    mut heads: Query<(Entity, &mut SnekHead)>,
    mut positions: Query<&mut Position>,
) {
    if let Some((head_entity, mut head)) = heads.iter_mut().next() {
        let segment_positions = segments
            .0
            .iter()
            .map(|e| *positions.get_mut(*e).unwrap())
            .collect::<Vec<Position>>();

        let mut head_position = positions.get_mut(head_entity).unwrap();
        let direction: Direction = if keyboard_input.pressed(KeyCode::Left) {
            Direction::Left
        } else if keyboard_input.pressed(KeyCode::Right) {
            Direction::Right
        } else if keyboard_input.pressed(KeyCode::Down) {
            Direction::Down
        } else if keyboard_input.pressed(KeyCode::Up) {
            Direction::Up
        } else {
            head.direction
        };

        
        if direction != head.direction.opposite() {
            let maybe_dir  = previous_direction.0;
            if maybe_dir.is_some() && maybe_dir.unwrap() != direction {
                head.direction = direction;                   
            }
        }

        if !snek_timer.0.finished() {
            return;
        }

        match &head.direction {
            Direction::Left => {
                head_position.x -= 1;
            }
            Direction::Right => {
                head_position.x += 1;
            }
            Direction::Up => {
                head_position.y += 1;
            }
            Direction::Down => {
                head_position.y -= 1;
            }
        };

        if head_position.x < 0
            || head_position.y < 0
            || head_position.x as u32 >= ARENA_WIDTH
            || head_position.y as u32 >= ARENA_HEIGHT
        {
            game_over_events.send(GameOverEvent);
        }

        if segment_positions.contains(&head_position) {
            game_over_events.send(GameOverEvent)
        }

        segment_positions
            .iter()
            .zip(segments.0.iter().skip(1))
            .for_each(|(position, segment)| *positions.get_mut(*segment).unwrap() = *position);

        last_tail_position.0 = Some(*segment_positions.last().unwrap());
        previous_direction.0 = Some(head.direction.clone());
    }
}

fn snek_eating(
    commands: &mut Commands,
    snek_timer: ResMut<SnekMoveTimer>,
    mut growth_events: ResMut<Events<GrowthEvent>>,
    food_positions: Query<(Entity, &Position), With<Food>>,
    head_positions: Query<&Position, With<SnekHead>>,
) {
    if !snek_timer.0.finished() {
        return;
    }

    for head_position in head_positions.iter() {
        for (entity, food_position) in food_positions.iter() {
            if food_position == head_position {
                commands.despawn(entity);
                growth_events.send(GrowthEvent);
            }
        }
    }
}

fn snek_growth(
    commands: &mut Commands,
    last_tail_position: Res<LastTailPosition>,
    growth_events: Res<Events<GrowthEvent>>,
    mut segments: ResMut<SnekSegments>,
    mut growth_reader: Local<EventReader<GrowthEvent>>,
    materials: Res<Materials>,
) {
    if growth_reader.iter(&growth_events).next().is_some() {
        segments.0.push(spawn_segment(
            commands,
            &materials.segment_material,
            last_tail_position.0.unwrap(),
        ))
    }
}

fn size_scaling(windows: Res<Windows>, mut query: Query<(&Size, &mut Sprite)>) {
    let window = windows.get_primary().unwrap();
    for (sprite_size, mut sprite) in query.iter_mut() {
        sprite.size = Vec2::new(
            sprite_size.width / ARENA_WIDTH as f32 * window.width() as f32,
            sprite_size.heigth / ARENA_HEIGHT as f32 * window.height() as f32,
        )
    }
}

fn position_translation(windows: Res<Windows>, mut query: Query<(&Position, &mut Transform)>) {
    fn convert(pos: f32, bound_window: f32, bound_game: f32) -> f32 {
        let tile_size = bound_window / bound_game;
        pos / bound_game * bound_window - (bound_window / 2.) + (tile_size / 2.)
    }
    let window = windows.get_primary().unwrap();
    for (position, mut transform) in query.iter_mut() {
        transform.translation = Vec3::new(
            convert(position.x as f32, window.width() as f32, ARENA_WIDTH as f32),
            convert(
                position.y as f32,
                window.height() as f32,
                ARENA_HEIGHT as f32,
            ),
            0.0,
        );
    }
}

fn food_spawner(
    commands: &mut Commands,
    materials: Res<Materials>,
    time: Res<Time>,
    segment_positions_res: Query<&Position, With<SnekSegment>>,
    foods: Query<Entity, With<Food>>,
    mut timer: Local<FoodSpawnerTimer>,
) {
    if timer.0.tick(time.delta_seconds()).finished() {
        let position: Position = Position {
            x: (random::<f32>() * ARENA_WIDTH as f32) as i32,
            y: (random::<f32>() * ARENA_HEIGHT as f32) as i32,
        };
        
        let segment_positions = segment_positions_res.iter().map(|position| position.clone()).collect::<Vec<Position>>();
        let count= foods.iter().collect::<Vec<Entity>>().len();

        if !segment_positions.contains(&position) && count <= 4 {
            commands
            .spawn(SpriteBundle {
                material: materials.food_material.clone(),
                ..Default::default()
            })
            .with(Food)
            .with(position)
            .with(Size::square(0.75));
        }
        
    }
}

fn snek_timer(time: Res<Time>, mut snek_timer: ResMut<SnekMoveTimer>) {
    snek_timer.0.tick(time.delta_seconds());
}

fn game_over(
    commands: &mut Commands,
    mut reader: Local<EventReader<GameOverEvent>>,
    game_over_events: Res<Events<GameOverEvent>>,
    materials: Res<Materials>,
    segments_res: ResMut<SnekSegments>,
    food: Query<Entity, With<Food>>,
    segments: Query<Entity, With<SnekSegment>>,
) {
    if reader.iter(&game_over_events).next().is_some() {
        for entity in food.iter().chain(segments.iter()) {
            commands.despawn(entity);
        }
        spawn_snek(commands, materials, segments_res);
    }
}

fn main() {
    App::build()
        .add_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .add_resource(WindowDescriptor {
            title: "Rusty Snek".to_string(),
            width: 800.0,
            height: 800.0,
            ..Default::default()
        })
        .add_resource(SnekMoveTimer(Timer::new(
            Duration::from_millis(200. as u64),
            true,
        )))
        .add_resource(SnekSegments::default())
        .add_resource(LastTailPosition::default())
        .add_resource(PreviousDirection::default())
        .add_event::<GrowthEvent>()
        .add_event::<GameOverEvent>()
        .add_startup_system(setup.system())
        .add_startup_stage("game_setup", SystemStage::single(spawn_snek.system()))
        .add_system(snek_timer.system())
        .add_system(snek_movement.system())
        .add_system(snek_eating.system())
        .add_system(snek_growth.system())
        .add_system(food_spawner.system())
        .add_system(game_over.system())
        .add_system(position_translation.system())
        .add_system(size_scaling.system())
        .add_plugins(DefaultPlugins)
        .run();
}
