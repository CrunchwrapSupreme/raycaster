extern crate nalgebra as na;
use na::Vector2;

pub type Vector2D = Vector2<f64>;

#[derive(Clone, Copy)]
pub enum BlockType {
    NONE = 0,
    WALL = 1,
}

pub struct BlockMap {
    blocks: Vec<BlockType>,
    width: usize,
    height: usize,
}

pub struct BlockData {
    pub hit: Vector2D,
    pub light: f64
}

impl BlockMap {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            blocks: vec![BlockType::NONE; width * height],
            width: width,
            height: height,
        }
    }

    pub fn populate_map(&mut self) {
        for x in 0..self.width {
            let tdx = self.xy_to_index(x, 0);
            let bdx = self.xy_to_index(x, self.height - 1);
            self.blocks[tdx] = BlockType::WALL;
            self.blocks[bdx] = BlockType::WALL;
        }
        for y in 0..self.height {
            let ldx = self.xy_to_index(0, y);
            let rdx = self.xy_to_index(self.width - 1, y);
            self.blocks[ldx] = BlockType::WALL;
            self.blocks[rdx] = BlockType::WALL;
        }
        let idx = self.xy_to_index(3, 3);
        self.blocks[idx] = BlockType::WALL;
    }

    #[allow(dead_code)]
    pub fn at(&self, x: usize, y: usize) -> Option<BlockType> {
        let idx = self.xy_to_index(x, y);
        if idx >= self.size() {
            return None;
        }
        Some(self.blocks[idx])
    }

    fn xy_to_index(&self, x: usize, y: usize) -> usize {
        let x = x;
        let y = y;
        x + y * self.width
    }

    #[allow(dead_code)]
    fn size(&self) -> usize {
        self.width * self.height
    }

    pub fn cast_ray(&self, pos: Vector2D, dir: Vector2D) -> Option<BlockData> {
        let dydx = dir.y / dir.x;
        let dxdy = dir.x / dir.y;
        let sx = (1.0 + dydx * dydx).sqrt();
        let sy = (1.0 + dxdy * dxdy).sqrt();
        let mut len = Vector2D::new(0.0, 0.0);
        let mut map_check = Vector2::new(pos.x.trunc() as i32, pos.y.trunc() as i32);
        let mut map_step = Vector2::new(0 as i32, 0 as i32);

        if dir.x < 0.0 {
            len.x = pos.x.fract() * sx;
            map_step.x = -1;
        } else {
            len.x = (1.0 - pos.x.fract()) * sx;
            map_step.x = 1;
        }

        if dir.y < 0.0 {
            len.y = pos.y.fract() * sy;
            map_step.y = -1;
        } else {
            len.y = (1.0 - pos.y.fract()) * sy;
            map_step.y = 1;
        }

        let mut f: f64;
        let mut found_block = false;
        loop {
            if len.y < len.x {
                map_check.y += map_step.y;
                f = len.y;
                len.y += sy;
            } else {
                map_check.x += map_step.x;
                f = len.x;
                len.x += sx;
            }

            if !(0..self.width as i32).contains(&map_check.x) || !(0..self.height as i32).contains(&map_check.y) {
                break;
            }

            let block_query = self.xy_to_index(map_check.x as usize, map_check.y as usize);
            let block_query = &self.blocks[block_query];
            if let BlockType::WALL = block_query {
                found_block = true;
                break;
            }
        }

        let hit = Vector2::new(pos.x + dir.x * f, pos.y + dir.y * f);
        if found_block {
            Some(BlockData {
                hit: hit,
                light: 1.0//(10.0 / hit.magnitude()).clamp(1.0, 0.1)
            })
        } else {
            None
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size() {
        let map = BlockMap::new(2, 2);
        assert_eq!(map.size(), 4);
    }

    #[test]
    fn at() {
        let mut map = BlockMap::new(3,3);
        map.blocks[0] = BlockType::WALL;
        assert!(matches!(map.at(0,0), Some(BlockType::WALL)))
    }

    #[test]
    fn populate_map() {
        let mut map = BlockMap::new(3, 3);
        map.populate_map();
        for x in 0..map.width {
            assert!(matches!(map.at(x, 0), Some(BlockType::WALL)));
            assert!(matches!(map.at(x, map.height - 1), Some(BlockType::WALL)));
        }

        for y in 0..map.height {
            assert!(matches!(map.at(0, y), Some(BlockType::WALL)));
            assert!(matches!(map.at(map.width - 1, y), Some(BlockType::WALL)));
        }
    }

    #[test]
    fn cast_ray() {
        let mut map = BlockMap::new(6, 6);
        let pos = Vector2D::new(2.0, 2.0);
        let dir = Vector2D::new(1.0, 0.0);
        let hit = map.cast_ray(pos, dir);
        match hit {
            None => assert!(true),
            Some(_) => assert!(false)
        }

        map.populate_map();
        let hit = map.cast_ray(pos, dir);
        match hit {
            None => assert!(false),
            Some(v) => {
                assert!(v.hit.x < 5.01);
                assert!(v.hit.x > 4.99);
                assert!(v.hit.y > 1.99);
                assert!(v.hit.y < 2.01);
            }
        }
    }
}
