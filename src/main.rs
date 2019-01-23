extern crate sdl2; 
extern crate specs;
#[macro_use]
extern crate specs_derive;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;

const gravitonal_constant: f64 = 6.67e-11f64;

use specs::{Builder, Component, DispatcherBuilder, ReadStorage, System, VecStorage, World, WriteStorage, Entities, RunNow};

#[derive(Component, Debug)]
#[storage(VecStorage)]
struct Position {
    x: f64,
    y: f64
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
struct Velocity {
    x: f64,
    y: f64,
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
struct Mass{
    mass: f64
}

struct UpdateVel;

impl<'a> System<'a> for UpdateVel {
    type SystemData = (Entities<'a>,
                        WriteStorage<'a, Velocity>,
                        ReadStorage<'a, Position>,
                        ReadStorage<'a, Mass>);

    fn run(&mut self, data: Self::SystemData) {
        use specs::Join;
        let (entities, mut vel_storage, pos_storeage, mass_storage) = data;
        
        for (vel, pos, mass) in (&mut vel_storage, &pos_storeage, &mass_storage).join() {
            // vel.x = 0.0;
            // vel.y = 0.0;

            for ent in entities.join(){
                let other_mass = mass_storage.get(ent);
                let other_pos = pos_storeage.get(ent);

                if let Some(other_mass) = other_mass {
                    if let Some(other_pos) = other_pos {
                        // calculate distance between the two objects
                        let dist = ((pos.x - other_pos.x).powf(2.0) + (pos.y - other_pos.y).powf(2.0)).sqrt();
                        if dist != 0.0 {
                            // calculate gravitonal force
                            let grav = (gravitonal_constant * mass.mass * other_mass.mass) / dist.powf(2.0);
                            // get angle between points
                            let angle = (other_pos.x- pos.x).atan2(other_pos.y - pos.y);
                            // deconstruct x and y values
                            let change = angle.sin_cos();
                            let x_change = change.0 * grav * 0.0000000001;
                            let y_change = change.1 * grav * 0.0000000001;

                            vel.x += x_change;
                            vel.y += y_change;
                        }
                    }
                }
            }
        }
    }
}

struct UpdatePos;

impl<'a> System<'a> for UpdatePos {
    type SystemData = (ReadStorage<'a, Velocity>,
                        WriteStorage<'a, Position>);

    fn run(&mut self, (vels, mut poses): Self::SystemData) {
        use specs::Join;
        for (vel, pos) in (&vels, &mut poses).join(){ 
            pos.x += vel.x * 1.0;
            pos.y += vel.y * 1.0;
        }
    }
}

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
 
    let window = video_subsystem.window("Astro-Physics sim", 800, 600)
        .position_centered()
        .build()
        .unwrap();
 
    let mut canvas = window.into_canvas().build().unwrap();
 
    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut world = World::new();
    world.register::<Position>();
    world.register::<Velocity>();
    world.register::<Mass>();

    let zoom = (0.1, 0.1);

    /*world
        .create_entity()
        .with(Position { x: 0.0, y: 0.0 })
        .with(Velocity { x: 0.0, y: 0.0 })
        .with(Mass{ mass: 1.9e+20f64})
        .build();*/

    for x in 0..15{
        for y in 0..15{
            world
                .create_entity()
                .with(Position { x: 50.0 * (0.0 + x as f64) - 375.0, y: 50.0 * (0.0 + y as f64) - 375.0 })
                .with(Velocity { x: 0.001, y: 0.001 })
                .with(Mass{ mass: 5.0e+10f64})
                .build();
        }
    }

    let mut dispatcher = DispatcherBuilder::new()
        .with(UpdateVel, "Update_vel", &[])
        .with(UpdatePos, "Update_pos", &["Update_vel"])
        //.with_thread_local(Render)
        .build();

    'running: loop {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }

        dispatcher.dispatch(&mut world.res);
        world.maintain();

        use specs::Join;
        let pos_storage = world.read_storage::<Position>();
        let vel_storage = world.read_storage::<Velocity>();
        for ent in world.entities().join(){
            let pos = pos_storage.get(ent);
            let vel = vel_storage.get(ent);

            canvas.set_draw_color(Color::RGB(255, 248, 231));
            if let Some(vel) = vel {
                let velocity = ((vel.x).powf(2.0) + (vel.y).powf(2.0)).sqrt();
                let red_color = 255.0 * (velocity / 25.0);
                let red_color_clamp: u8;
                if red_color > 255.0{
                    red_color_clamp = 255;
                } else if red_color < 0.0 {
                    red_color_clamp = 0;
                } else {
                    red_color_clamp = red_color as u8;
                }
                canvas.set_draw_color(Color::RGB(red_color_clamp, 248, 200));
            }

            if let Some(pos) = pos {
                canvas.draw_point(sdl2::rect::Point::new(((pos.x + (400.0 / zoom.0)) * zoom.0) as i32, ((pos.y +  (300.0 / zoom.1)) * zoom.1) as i32));
            }
        }

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}