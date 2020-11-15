use bevy::prelude::*;
use rand::Rng;


fn main() {
    let matches = clap::App::new("bevy_balls")
        .version("1.0")
        .author("James Bell")
        .about("Bouncing balls experiment for class")
        .arg(clap::Arg::with_name("room_size")
            .short("r")
            .long("room_size")
            .help("Sets the room size, number of balls is equal to room_size^2")
            .takes_value(true))
        .arg(clap::Arg::with_name("toggle_version")
            .short("t")
            .long("toggle_version")
            .help("If true, adds/removes components every collision")
            .takes_value(true))
        .get_matches();

    let room_size = matches.value_of("room_size").unwrap_or("10.0").parse::<f32>().unwrap_or(10.0);
    let toggle = matches.value_of("toggle_version").unwrap_or("true").parse::<bool>().unwrap_or(false);

    App::build()
        .add_resource(Room{x: room_size/2.0, y: room_size/2.0})
        .add_plugins(DefaultPlugins)
        .add_startup_system(start.system())
        .add_resource(Time::default())
        .add_system(update_positions.system())
        .add_system(if toggle {check_collisions_toggle.system()} else {check_collisions.system()})
        .run();
}

fn start(mut commands: Commands, room: Res<Room>){
    let mut rng = rand::thread_rng();
    for i in 0..(((room.x * 2.0) * (room.y * 2.0))as isize){
        commands.spawn((
            Position{
                x: rng.gen_range(-room.x, room.x),
                y: rng.gen_range(-room.y, room.y)
            },
            Velocity{
                x: rng.gen_range(-1.0, 1.0),
                y: rng.gen_range(-1.0, 1.0)
            }
        ));
    }
}

struct Position{
    x: f32,
    y: f32
}

struct Velocity{
    x: f32,
    y: f32
}

struct Room{
    x: f32,
    y: f32
}

struct Marker{}

fn update_positions(time: Res<Time>, mut balls: Query<(&mut Position, &Velocity)>){
    for (mut position, velocity) in balls.iter_mut(){
        position.x += velocity.x * time.delta_seconds;
        position.y += velocity.y * time.delta_seconds;
    }
}

fn collision_check(pos_1: &Position, pos_2: &Position) -> Option<Velocity>{
    if (pos_1.x - pos_2.x).powf(2.0) -  (pos_1.y - pos_2.y).powf(2.0) <= 4.0{
        return Some(Velocity{
            x: pos_1.x - pos_2.x,
            y: pos_1.y - pos_2.y
        })
    }
    None
}

fn check_collisions(room: Res<Room>, time: Res<Time>, mut balls: Query<(&mut Position, &mut Velocity)>){
    for (mut position, mut velocity) in balls.iter_mut(){
        //Unstuck
        if position.x.abs() > room.x{
            position.x = position.x.signum() * (room.x - 1.0);
        }
        if position.y.abs() > room.y{
            position.y = position.y.signum() * (room.y - 1.0);
        }
    }

    for (position, velocity) in unsafe{balls.iter_unsafe()}{
        //Collisions
        for (position_other, mut velocity_other) in unsafe{balls.iter_unsafe()}{
            if let Some(new_vel) = collision_check(&position_other, &position){
                *velocity_other = new_vel;
                break;
            }
        }
    }
    println!("{}", time.delta_seconds_f64);
}

fn check_collisions_toggle(mut commands: Commands, room: Res<Room>, time: Res<Time>, mut balls_without: Query<(Entity, &mut Position, &mut Velocity)>, mut balls_with: Query<(&Entity, &mut Position, &mut Velocity, &mut Marker)>){
    for (_ent, mut position, mut velocity) in balls_without.iter_mut(){
        //Unstuck
        if position.x.abs() > room.x{
            position.x = position.x.signum() * (room.x - 1.0);
        }
        if position.y.abs() > room.y{
            position.y = position.y.signum() * (room.y - 1.0);
        }
    }

    for (_ent, mut position, mut velocity, _marker) in balls_with.iter_mut(){
        //Unstuck
        if position.x.abs() > room.x{
            position.x = position.x.signum() * (room.x - 1.0);
        }
        if position.y.abs() > room.y{
            position.y = position.y.signum() * (room.y - 1.0);
        }
    }

    let mut ents_to_add = Vec::new();
    let mut ents_to_remove = Vec::new();

    for (_ent, position, velocity) in unsafe{balls_without.iter_unsafe()}{
        //Collisions
        for (ent, position_other, mut velocity_other) in unsafe{balls_without.iter_unsafe()}{
            if let Some(new_vel) = collision_check(&position_other, &position){
                *velocity_other = new_vel;
                ents_to_add.push(ent.clone());
                break;
            }
        }
        
        for (ent, position_other, mut velocity_other, mut marker) in unsafe{balls_with.iter_unsafe()}{
            if let Some(new_vel) = collision_check(&position_other, &position){
                *velocity_other = new_vel;
                ents_to_remove.push(ent.clone());
                break;
            }
        }
    }

    for (_ent, position, velocity, mut mark) in unsafe{balls_with.iter_unsafe()}{
        //Collisions
        for (ent, position_other, mut velocity_other) in unsafe{balls_without.iter_unsafe()}{
            if let Some(new_vel) = collision_check(&position_other, &position){
                *velocity_other = new_vel;
                ents_to_add.push(ent.clone());
                break;
            }
        }
        
        for (ent, position_other, mut velocity_other, mut marker) in unsafe{balls_with.iter_unsafe()}{
            if let Some(new_vel) = collision_check(&position_other, &position){
                *velocity_other = new_vel;
                ents_to_remove.push(ent.clone());
                break;
            }
        }
    }

    for ent in ents_to_add.drain(..){
        commands.insert_one(ent, Marker{});
    }

    for ent in ents_to_remove.drain(..){
        commands.remove_one::<Marker>(ent);
    }

    println!("{} TOGGLED", time.delta_seconds_f64);
}