use std::f32::consts::PI;

use engine::{
    physics::EARTH_ACCELERATION_M_PER_S,
    simulator::{Draw, Simulation, Tick, TickDraw},
};

use macroquad::prelude as mq;

const BALL_RADIUS: f32 = 25.;
const BALL_COLOUR: mq::Color = mq::WHITE;
const TICK_LEN_SECONDS: f64 = 0.0167;
const DAMPENING_RATIO: f32 = 0.9;

fn draw_arrow(
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    thickness: f32,
    color: mq::Color,
    head_ratio: f32,
) {
    mq::draw_line(x1, y1, x2, y2, thickness, color);
    // arrow head
    let tip_theta: f32 = PI / 6. - PI;
    let tail_pos = glam::vec2(x1, y1);
    let tip_pos = glam::vec2(x2, y2);
    let tip_from_origin = tip_pos - tail_pos;
    let a_unit = glam::Vec2::from_angle(tip_theta);
    let b_unit = glam::Vec2::from_angle(-tip_theta);

    let a = a_unit.rotate(tip_from_origin) * head_ratio + tip_pos;
    let b = b_unit.rotate(tip_from_origin) * head_ratio + tip_pos;

    mq::draw_triangle(
        mq::vec2(x2, y2),
        mq::vec2(a.x, a.y),
        mq::vec2(b.x, b.y),
        color,
    );
}
mod engine {
    pub mod simulator {
        pub trait Tick {
            /// Handle a tick
            fn on_tick(&mut self, tick_len_seconds: f64) -> ();
        }

        pub trait Draw {
            fn on_draw(&self) -> ();
        }

        pub trait TickDraw: Tick + Draw {}

        pub struct Simulation<'a> {
            seconds_per_tick: f64,
            objects: Vec<Box<&'a mut dyn TickDraw>>,
            tick_count: usize,
        }

        impl<'a> Simulation<'a> {
            pub fn new(seconds_per_tick: f64) -> Self {
                Self {
                    seconds_per_tick,
                    objects: Vec::new(),
                    tick_count: 0,
                }
            }

            pub fn get_tick_count(&self) -> usize {
                self.tick_count
            }

            pub fn do_tick(&mut self, time: f64) -> () {
                let expected_tick_count = (time / self.seconds_per_tick).floor() as usize;
                let ticks_to_perform = expected_tick_count - self.tick_count;
                for _ in 0..(ticks_to_perform + 1) {
                    self.objects
                        .iter_mut()
                        .for_each(|o| o.on_tick(self.seconds_per_tick));
                }
                self.tick_count += ticks_to_perform;
            }

            pub fn do_draw(&self) -> () {
                self.objects.iter().for_each(|o| o.on_draw())
            }

            pub fn add_object(&mut self, object: &'a mut dyn TickDraw) -> () {
                self.objects.push(Box::from(object));
            }
        }
    }

    pub mod physics {
        pub const EARTH_ACCELERATION_M_PER_S: f64 = 9.8;
    }
}

struct Ball {
    pos: glam::Vec2,
    velocity: glam::Vec2,
}

impl Tick for Ball {
    fn on_tick(&mut self, tick_len_seconds: f64) -> () {
        // update velocity
        self.velocity.y += (tick_len_seconds * EARTH_ACCELERATION_M_PER_S * 3.) as f32;
        self.pos += self.velocity * tick_len_seconds as f32;
        if self.pos.y >= 500. {
            self.velocity.y *= -DAMPENING_RATIO;
        }
    }
}

impl Draw for Ball {
    fn on_draw(&self) -> () {
        mq::draw_circle(self.pos.x, self.pos.y, BALL_RADIUS, BALL_COLOUR);
        let circle_center = self.pos;
        draw_arrow(
            circle_center.x,
            circle_center.y,
            circle_center.x + self.velocity.x,
            circle_center.y + self.velocity.y,
            1.,
            mq::BLUE,
            0.2,
        );
    }
}

impl TickDraw for Ball {}

fn draw_dbg_text(time: f64, ticks_so_far: usize, frames_so_far: usize) -> () {
    mq::draw_text(
            &format!("Time elapsed: {:.3}\nTPS: {:.3} (expected {:.3})\nTicks: {}\nFPS: {:.3} (expected {:.3})\nFrames: {}",
                time,
                ticks_so_far as f64/time,
                1. / TICK_LEN_SECONDS,
                ticks_so_far,
                frames_so_far as f64 / time,
                mq::get_fps(),
                frames_so_far),
            5.,
            20.,
            16.,
            mq::WHITE,
        );
}

#[macroquad::main("Fixed Timestep")]
async fn main() {
    let mut ball = Ball {
        pos: glam::Vec2 { x: 400., y: 100. },
        velocity: glam::Vec2::ZERO,
    };
    let mut simulation = Simulation::new(TICK_LEN_SECONDS);
    simulation.add_object(&mut ball);

    let mut frames_so_far = 0;

    loop {
        let time = mq::get_time();
        simulation.do_tick(time);

        let ticks_so_far = simulation.get_tick_count();

        mq::clear_background(mq::BLACK);
        draw_dbg_text(time, ticks_so_far, frames_so_far);
        simulation.do_draw();

        frames_so_far += 1;
        mq::next_frame().await
    }
}
