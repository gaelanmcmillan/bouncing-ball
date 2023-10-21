use std::f32::consts::PI;

use engine::{
    physics::EARTH_ACCELERATION_M_PER_S,
    simulator::{Draw, Expire, Simulation, Tick, TickDrawExpire},
};

use macroquad::prelude as mq;

const BALL_EXPIRY_TIME: f64 = 2.;
const FLOOR_Y: f32 = 500.;
const TICK_LEN_SECONDS: f64 = 0.0167 / 2.;
const GRAVITY_MULTIPLIER: f64 = 40.;
const DAMPENING_MULTIPLIER: f32 = 0.8;
const ARROW_LEN_MULTIPLIER: f32 = 0.2;

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
    let tail_pos = mq::vec2(x1, y1);
    let tip_pos = mq::vec2(x2, y2);
    let tip_from_origin = tip_pos - tail_pos;
    let a_unit = mq::Vec2::from_angle(tip_theta);
    let b_unit = mq::Vec2::from_angle(-tip_theta);

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
            fn on_tick(&mut self, tick_len_seconds: f64);
        }

        pub trait Draw {
            fn on_draw(&self);
        }

        pub trait Expire {
            fn is_expired(&self) -> bool;
        }

        pub trait TickDrawExpire: Tick + Draw + Expire {}

        pub struct Simulation {
            seconds_per_tick: f64,
            objects: Vec<Box<dyn TickDrawExpire>>,
            tick_count: usize,
        }

        impl Simulation {
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

            pub fn get_object_count(&self) -> usize {
                self.objects.len()
            }

            pub fn do_tick(&mut self, time: f64) {
                let expected_tick_count = (time / self.seconds_per_tick).floor() as usize;
                let ticks_to_perform = expected_tick_count - self.tick_count;
                for _ in 0..(ticks_to_perform + 1) {
                    self.objects
                        .iter_mut()
                        .for_each(|o| o.on_tick(self.seconds_per_tick));
                }
                self.tick_count += ticks_to_perform;
            }

            pub fn do_draw(&self) {
                self.objects.iter().for_each(|o| o.on_draw())
            }

            pub fn do_handle_expiry(&mut self) {
                self.objects.retain(|o| !o.is_expired());
            }

            pub fn add_object(&mut self, boxed: Box<dyn TickDrawExpire>) {
                self.objects.push(boxed);
            }
        }
    }

    pub mod physics {
        pub const EARTH_ACCELERATION_M_PER_S: f64 = 9.8;
    }
}

struct Ball {
    pos: mq::Vec2,
    velocity: mq::Vec2,
    radius: f32,
    color: mq::Color,
    time_on_floor: f64,
}

impl Tick for Ball {
    fn on_tick(&mut self, tick_len_seconds: f64) {
        // update velocity
        self.velocity.y +=
            (tick_len_seconds * EARTH_ACCELERATION_M_PER_S * GRAVITY_MULTIPLIER) as f32;
        self.pos += self.velocity * tick_len_seconds as f32;
        if self.pos.y > FLOOR_Y {
            self.pos.y = FLOOR_Y;
            self.velocity.y *= -DAMPENING_MULTIPLIER;
            self.time_on_floor += tick_len_seconds;
        }

        if self.pos.x > 500. || self.pos.x < 200. {
            self.pos.x = self.pos.x.clamp(200., 500.);
            self.velocity.x *= -DAMPENING_MULTIPLIER;
        }
    }
}

fn color_with_alpha(color: mq::Color, a: f32) -> mq::Color {
    mq::Color {
        r: color.r,
        g: color.g,
        b: color.b,
        a,
    }
}

impl Ball {
    fn get_alpha(&self) -> f32 {
        ((BALL_EXPIRY_TIME - self.time_on_floor) / BALL_EXPIRY_TIME) as f32
    }
}
impl Draw for Ball {
    fn on_draw(&self) {
        let alpha = self.get_alpha();
        mq::draw_circle(
            self.pos.x,
            self.pos.y,
            self.radius,
            color_with_alpha(self.color, alpha),
        );
        let circle_center = self.pos;
        let scaled_velocity = self.velocity * ARROW_LEN_MULTIPLIER;
        draw_arrow(
            circle_center.x,
            circle_center.y,
            circle_center.x + scaled_velocity.x,
            circle_center.y + scaled_velocity.y,
            1.,
            color_with_alpha(mq::BLUE, alpha),
            0.2,
        );

        mq::draw_text(
            &format!("v: <{:.2},{:.2}>", self.velocity.x, self.velocity.y),
            10.,
            50.,
            15.,
            mq::RED,
        );
    }
}

impl Expire for Ball {
    fn is_expired(&self) -> bool {
        self.time_on_floor >= BALL_EXPIRY_TIME
    }
}

impl TickDrawExpire for Ball {}

fn draw_dbg_text(time: f64, ticks_so_far: usize, frames_so_far: usize, object_count: usize) {
    mq::draw_text(
            &format!("Time elapsed {:.2}\nTPS: {:.2} (expected {:.2})\nTicks: {}\nFPS: {:.2} (expected {:.2})\nFrames: {}\nObjects: {}",
                time,
                ticks_so_far as f64/time,
                1. / TICK_LEN_SECONDS,
                ticks_so_far,
                frames_so_far as f64 / time,
                mq::get_fps(),
                frames_so_far,
            object_count),
            5.,
            20.,
            16.,
            mq::WHITE,
        );
}

fn handle_click<T: FnMut()>(mut callback: T) {
    if mq::is_mouse_button_down(mq::MouseButton::Left) {
        callback();
    }
}

fn rand_vec2(xlow: f32, xhigh: f32, ylow: f32, yhigh: f32) -> mq::Vec2 {
    mq::vec2(
        mq::rand::gen_range(xlow, xhigh),
        mq::rand::gen_range(ylow, yhigh),
    )
}

#[macroquad::main("Bouncing Balls")]
async fn main() {
    let ball = Ball {
        pos: mq::Vec2 { x: 400., y: 100. },
        velocity: mq::Vec2::X * 80.,
        radius: 15.0,
        color: mq::WHITE,
        time_on_floor: 0.,
    };
    let mut simulation = Simulation::new(TICK_LEN_SECONDS);
    simulation.add_object(Box::from(ball));

    let mut frames_so_far = 0;

    loop {
        // Handle Inputs
        handle_click(|| {
            let b = Ball {
                pos: rand_vec2(200., 400., 200., 400.),
                velocity: rand_vec2(5., 50., 0., 0.),
                radius: mq::rand::gen_range(10., 30.),
                color: mq::Color::from_rgba(
                    mq::rand::gen_range(100, 255),
                    mq::rand::gen_range(100, 255),
                    mq::rand::gen_range(100, 255),
                    255,
                ),
                time_on_floor: 0.,
            };
            simulation.add_object(Box::from(b));
        });
        // Handle Ticks
        let time = mq::get_time();
        simulation.do_tick(time);

        // Handle Expiry
        simulation.do_handle_expiry();

        // Handle Drawing
        mq::clear_background(mq::BLACK);
        draw_dbg_text(
            time,
            simulation.get_tick_count(),
            frames_so_far,
            simulation.get_object_count(),
        );
        simulation.do_draw();

        frames_so_far += 1;
        mq::next_frame().await
    }
}
