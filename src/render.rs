use crate::map::{Vector2D, BlockMap};
use winit_input_helper::WinitInputHelper;
use winit::event::VirtualKeyCode;
use rayon::prelude::*;
use std::f64::consts::PI;

pub const RENDER_WIDTH: u32 = 320 * 2;
pub const RENDER_WIDTH_MID: u32 = (RENDER_WIDTH as f64 / 2.0) as u32;
pub const RENDER_HEIGHT: u32 = 240 * 2;
pub const RENDER_HEIGHT_MID: u32 = (RENDER_HEIGHT as f64 / 2.0) as u32;

const FOV: f64 = 60.0 * PI / 180.0;
const ROTATE_SPEED: f64 = PI;
const FORWARD_SPEED: f64 = 3.0;

type ColumnRange = std::ops::RangeInclusive<u32>;
struct ColumnData {
    column: ColumnRange,
    light: f64
}

pub struct Entity {
    dir: Vector2D,
    pos: Vector2D
}

pub struct RenderState {
    player: Entity,
    room: BlockMap
}

fn compute_light(light: f64) -> [u8; 4]
{
    let mut r: u8 = 0xf9;
    let mut g: u8 = 0xd4;
    let mut b: u8 = 0xa4;
    r = (r as f64 * light).floor() as u8;
    g = (g as f64 * light).floor() as u8;
    b = (b as f64 * light).floor() as u8;
    [r, g, b, 0xff]
}

impl RenderState {
    pub fn new() -> Self {
        let mut map = BlockMap::new(8, 8);
        map.populate_map();
        Self {
            room: map,
            player: Entity {
                pos: Vector2D::new(2.0, 2.0),
                dir: Vector2D::new(1.0, 0.0)
            }
        }
    }

    pub fn update(&mut self, input: &WinitInputHelper, delta: f64) {
        let rotation: f64 = {
            if input.key_held(VirtualKeyCode::Right) {
                delta * ROTATE_SPEED * -1.0
            } else if input.key_held(VirtualKeyCode::Left) {
                delta * ROTATE_SPEED
            } else {
                0.0
            }
        };
        let pos_d: f64 = {
            if input.key_held(VirtualKeyCode::Up) {
                delta * FORWARD_SPEED
            } else if input.key_held(VirtualKeyCode::Down) {
                delta * FORWARD_SPEED * -1.0
            } else {
                0.0
            }
        };
        let old = self.player.dir;
        let new = Vector2D::new(old.x * rotation.cos() - old.y * rotation.sin(),
                                old.x * rotation.sin() + old.y * rotation.cos());
        self.player.dir = new.normalize();
        if pos_d.abs() > f64::EPSILON {
            let move_dir = (self.player.dir * pos_d).normalize();
            let dv = match self.room.cast_ray(self.player.pos, move_dir, 1.0) {
                None => {
                    pos_d
                },
                Some(x) => {
                    let diff = ((x.hit - self.player.pos).magnitude() - 0.1).clamp(0.0, 1.0);
                    let pos_clamp = pos_d.abs().clamp(0.0, diff);
                    pos_clamp * pos_d.signum()
                }
            };
            self.player.pos += self.player.dir * dv;
        }
        self.player.pos.x = self.player.pos.x.clamp(0.0, self.room.width as f64);
        self.player.pos.y = self.player.pos.y.clamp(0.0, self.room.height as f64);
    }

    fn compute_column(&self, hit: Vector2D) -> ColumnRange {
        let diff = hit - self.player.pos; // Hit relative to player
        let beta = diff.angle(&self.player.dir);
        let dist = diff.magnitude() * beta.cos();
        let height = RENDER_HEIGHT as f64 / dist;
        let half_height = height / 2.0;
        let top = RENDER_HEIGHT_MID as f64 - half_height;
        let bot = RENDER_HEIGHT_MID as f64 + half_height;
        (top.round() as u32)..=(bot.round() as u32)
    }

    pub fn render(&self, frame: &mut [u8]) {
        let view_dist = (1.0 / (FOV / 2.0).tan()) * RENDER_WIDTH_MID as f64;

        let col_iter = (0..RENDER_WIDTH).into_par_iter().map(|x| {
            let screenx = x as f64 - (RENDER_WIDTH_MID as f64);
            let angle = view_dist.atan2(screenx) - std::f64::consts::FRAC_PI_2;
            let dir = self.player.dir;
            let ray = Vector2D::new(dir.x * angle.cos() - dir.y * angle.sin(),
                                    dir.x * angle.sin() + dir.y * angle.cos()).normalize();
            let hit = self.room.cast_ray(self.player.pos, ray, f64::INFINITY);
            match hit { // Move wall color computation here while we have ray info
                Some(x) => ColumnData {
                    column: self.compute_column(x.hit),
                    light: x.light
                },
                None => ColumnData {
                    column: u32::MAX..=u32::MAX,
                    light: 1.0
                }
            }
        });
        let columns: Vec<ColumnData> = col_iter.collect();

        let frame_iter = frame.par_chunks_exact_mut(4).enumerate();
        frame_iter.for_each(|(i, pixel)| {
            let x = (i % RENDER_WIDTH as usize) as u32;
            let y = (i / RENDER_WIDTH as usize) as u32;
            let cd = &columns[x as usize];
            let col = {
                if cd.column.contains(&y) {
                    if x == RENDER_WIDTH_MID {
                        [0xff, 0x00, 0x00, 0xff]
                    } else {
                        compute_light(cd.light)
                    }
                } else if y <= RENDER_HEIGHT_MID {
                    [0xff, 0xff, 0xff, 0xff]
                }  else {
                    [0xbc, 0x78, 0xa2, 0xff]
                }
            };

            pixel.copy_from_slice(&col);
        });
    }
}
