extern crate sdl2; 
extern crate specs;
#[macro_use]
extern crate specs_derive;
extern crate rand;
extern crate num;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;
use rand::Rng;
use num::clamp;

const GRAVITONAL_CONSTANT: f64 = 6.67e-11f64;

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
        use specs::ParJoin;
        use specs::Join;
        use specs::prelude::ParallelIterator;

        let (entities, mut vel_storage, pos_storeage, mass_storage) = data;

        (&mut vel_storage, &pos_storeage, &mass_storage)
        .par_join()
        .for_each(|(vel, pos, mass)| {
            for ent in entities.join(){
                let other_mass = mass_storage.get(ent);
                let other_pos = pos_storeage.get(ent);

                if let Some(other_mass) = other_mass {
                    if let Some(other_pos) = other_pos {
                        // cy = y2 - y1
                        // cx = x2 - x1
                        let cx = other_pos.x - pos.x;
                        let cy = other_pos.y - pos.y;

                        // dist = 1.0/sqrt(cx^2 + cy^2 + min_dist)
                        let dist = 1.0 / ((cx * cx) + (cy * cy) + 0.0001).sqrt();

                        // f = G * m2 * m1 * dist * dist
                        let f = GRAVITONAL_CONSTANT * mass.mass * other_mass.mass * dist * dist;

                        // ay += cy * f
                        // ax += cx * f
                        vel.x += cx * f * 5.0f64;
                        vel.y += cy * f * 5.0f64;
                    }
                }
            }
        });
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
 
    let window = video_subsystem.window("Astro-Physics sim", (800.0*1.5) as u32, (600.0*1.5) as u32)
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

    let mut zoom = (0.01, 0.01);
    let mut offset = (0.0, 0.0);
    let mut pause = false;

    let mut rng = rand::thread_rng();

    /*world.create_entity()
            .with(Position { x: 0.0, y: 0.0})
            .with(Velocity { x: 0.0, y: 0.0})
            .with(Mass{ mass: 5.0e+15f64})
            .build();*/

    for _ in 0..4096{
        let _x = rng.gen_range(-1.0, 1.0) * 1.0e+10f64;
        let _y = rng.gen_range(-1.0, 1.0) * 1.0e+10f64;

        let _vx = rng.gen_range(-1.0, 1.0) * 1.0e+7f64;
        let _vy = rng.gen_range(-1.0, 1.0) * 1.0e+7f64;

        world.create_entity()
            .with(Position { x: _x, y: _y})
            .with(Velocity { x: _vx, y: _vy})
            .with(Mass{ mass: 5.0e+10f64})
            .build();
    }

    let mut dispatcher = DispatcherBuilder::new()
        .with(UpdateVel, "Update_vel", &[])
        .with(UpdatePos, "Update_pos", &["Update_vel"])
        //.with_thread_local(Render)
        .build();

    let mut move_down = false;
    let mut move_up = false;
    let mut move_right = false;
    let mut move_left = false;

    'running: loop {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::MouseWheel {y, ..} => {
                    zoom.0 = clamp::<f64>((y as f64 * 0.001) + zoom.0, 0.0, 5.0);
                    zoom.1 = clamp::<f64>((y as f64 * 0.001) + zoom.1, 0.0, 5.0);
                },
                Event::KeyDown { keycode: Some(Keycode::Down), ..} => {
                    move_down = true;
                },
                Event::KeyDown { keycode: Some(Keycode::Up), ..} => {
                    move_up = true;
                },
                Event::KeyDown { keycode: Some(Keycode::Right), ..} => {
                    move_right = true;
                },
                Event::KeyDown { keycode: Some(Keycode::Left), ..} => {
                    move_left = true;
                },
                Event::KeyUp { keycode: Some(Keycode::Down), ..} => {
                    move_down = false;
                },
                Event::KeyUp { keycode: Some(Keycode::Up), ..} => {
                    move_up = false;
                },
                Event::KeyUp { keycode: Some(Keycode::Right), ..} => {
                    move_right = false;
                },
                Event::KeyUp { keycode: Some(Keycode::Left), ..} => {
                    move_left = false;
                },
                Event::KeyDown { keycode: Some(Keycode::Space), ..} => {
                    pause = !pause;
                },
                _ => {}
            }
        }

        if move_down{
            offset.1 -= 10.0;
        }
        if move_up{
            offset.1 += 10.0;
        }
        if move_right{
            offset.0 -= 10.0;
        }
        if move_left{
            offset.0 += 10.0;
        }

        if !pause{
            dispatcher.dispatch(&mut world.res);
            world.maintain();
        }

        use specs::Join;
        let pos_storage = world.read_storage::<Position>();
        let vel_storage = world.read_storage::<Velocity>();
        for ent in world.entities().join(){
            let pos = pos_storage.get(ent);
            let vel = vel_storage.get(ent);

            canvas.set_draw_color(Color::RGB(255, 248, 231));
            if let Some(vel) = vel {
                let velocity = ((vel.x).powf(2.0) + (vel.y).powf(2.0)).sqrt();
                let red_color = 255.0 * (velocity / 1e+7f64);
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

            let copy_zoom = (zoom.0 * 0.000001, zoom.1 * 0.000001);

            if let Some(pos) = pos {
                canvas.draw_point(sdl2::rect::Point::new((offset.0 + (pos.x + (400.0 / copy_zoom.0)) * copy_zoom.0) as i32, (offset.1 + (pos.y +  (300.0 / copy_zoom.1)) * copy_zoom.1) as i32));
            }
        }

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}