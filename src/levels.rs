use super::{PIXEL_SIZE, TILE_PIXELS, TILE_SIZE};
use macroquad::prelude::*;
use std::collections::HashMap;

fn draw_rect_i32(x: i32, y: i32, w: i32, h: i32, c: Color) {
    draw_rectangle(x as f32, y as f32, w as f32, h as f32, c)
}

#[derive(Copy, Clone, Debug)]
pub enum Tile {
    Empty,
    Player,
    BackWall,
    Wall,

    DoorGeneric,
    Door(usize),

    ExitAnchor,
}

impl Tile {
    pub fn is_solid(&self) -> bool {
        match self {
            Self::Wall => true,
            _ => false,
        }
    }

    pub fn from_string(s: &str) -> Self {
        match s {
            "wall" => Self::Wall,
            "player" => Self::Player,
            "backwall" => Self::BackWall,
            "door" => Self::DoorGeneric,
            "exit_anchor" => Self::ExitAnchor,
            _ => Self::Empty,
        }
    }

    pub fn draw(&self, x: i32, y: i32) {
        match self {
            Self::Empty => (),
            Self::Wall => draw_rect_i32(x, y, TILE_PIXELS, TILE_PIXELS, BLACK),
            Self::BackWall => draw_rect_i32(x, y, TILE_PIXELS, TILE_PIXELS, GRAY),
            // _ => draw_rect_i32(x, y, TILE_PIXELS, TILE_PIXELS, MAGENTA),
            _ => (),
        }
    }
}

pub struct AABB {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

impl AABB {
    fn intersect(&self, other: &Self) -> bool {
        self.x <= other.x + other.w
            && self.x + self.w >= other.x
            && self.y <= other.y + other.h
            && self.y + self.h >= other.w
    }
}

fn check_tilemap_collision(c_box: AABB, map: &Vec<Vec<Vec<Tile>>>) -> bool {
    let extra_x = (c_box.x + c_box.w) % TILE_SIZE != 0;
    let extra_y = (c_box.y + c_box.h) % TILE_SIZE != 0;

    let xi: Box<[i32]> = if extra_x {
        (0..=c_box.w).step_by(TILE_SIZE as usize).collect()
    } else {
        (0..c_box.w).step_by(TILE_SIZE as usize).collect()
    };
    let yi: Box<[i32]> = if extra_y {
        (0..=c_box.h).step_by(TILE_SIZE as usize).collect()
    } else {
        (0..c_box.h).step_by(TILE_SIZE as usize).collect()
    };

    for x in xi.iter() {
        for y in yi.iter() {
            let (tx, ty) = ((c_box.x + x) / TILE_SIZE, (c_box.y + y) / TILE_SIZE);
            if tx < 0 || tx >= map[0][0].len() as i32 || ty < 0 || ty >= map[0].len() as i32 {
                continue;
            }
            let (tx, ty) = (tx as usize, ty as usize);
            for l in map {
                if l[ty][tx].is_solid() {
                    return true;
                }
            }
        }
    }

    false
}

pub trait Object {
    fn get_type(&self) -> &'static str;

    fn get_aabb(&self) -> AABB;

    fn update(&mut self, _tiles: &Vec<Vec<Vec<Tile>>>) {}

    fn draw(&self, _off_x: i32, _off_y: i32) {}

    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

pub struct Player {
    pub x: i32,
    pub y: i32,

    pub vx: i32,
    pub vy: i32,

    pub grounded: bool,
}

impl Object for Player {
    fn get_type(&self) -> &'static str {
        "PLAYER"
    }

    fn get_aabb(&self) -> AABB {
        AABB {
            x: self.x,
            y: self.y,
            w: TILE_SIZE,
            h: TILE_SIZE,
        }
    }

    fn update(&mut self, tiles: &Vec<Vec<Vec<Tile>>>) {
        // accelerate left and right
        if is_key_down(KeyCode::Left) {
            self.vx -= TILE_SIZE / 16;
            if self.vx < -TILE_SIZE * 3 / 16 {
                self.vx = -TILE_SIZE * 3 / 16
            }
        } else if is_key_down(KeyCode::Right) {
            self.vx += TILE_SIZE / 16;
            if self.vx > TILE_SIZE * 3 / 16 {
                self.vx = TILE_SIZE * 3 / 16
            }
        }
        if !is_key_down(KeyCode::Left) && !is_key_down(KeyCode::Right) {
            self.vx *= 11;
            self.vx /= 16;
        }

        if is_key_down(KeyCode::Z) {
            self.vy += TILE_SIZE / 16 / 5;
        } else {
            self.vy += TILE_SIZE / 16 / 2;
        }

        if self.grounded && is_key_down(KeyCode::Z) {
            self.vy = -TILE_SIZE * 5 / 16;
            self.grounded = false;
        }

        // cap vx and vy at one tile per game step
        // in practice this will never be hit
        self.vx = self.vx.clamp(-TILE_SIZE, TILE_SIZE);
        self.vy = self.vy.clamp(-TILE_SIZE, TILE_SIZE);

        // horizontal movement
        // move to tile boundary if we are moving too fast

        let remaining_movement = if (self.x + self.vx) / TILE_SIZE != self.x / TILE_SIZE {
            let old_pos = self.x;
            if self.vx < 0 {
                self.x = self.x / TILE_SIZE * TILE_SIZE;
            } else {
                self.x = (self.x / TILE_SIZE + 1) * TILE_SIZE;
            }
            // difference between target position and current is remaining movement
            (old_pos + self.vx) - (self.x)
        } else {
            self.vx
        };
        // now we are aligned at tile boundary, do remaining movement,
        // then step back if we are then colliding
        self.x += remaining_movement;
        if check_tilemap_collision(self.get_aabb(), tiles) {
            self.x -= remaining_movement;
            self.vx = 0;
        }

        // same but vertical

        let remaining_movement = if (self.y + self.vy) / TILE_SIZE != self.y / TILE_SIZE {
            let old_pos = self.y;
            if self.vy < 0 {
                self.y = self.y / TILE_SIZE * TILE_SIZE;
            } else {
                self.y = (self.y / TILE_SIZE + 1) * TILE_SIZE;
            }
            (old_pos + self.vy) - (self.y)
        } else {
            self.vy
        };
        self.y += remaining_movement;
        if check_tilemap_collision(self.get_aabb(), tiles) {
            self.y -= remaining_movement;
            if self.vy > 0 {
                // going down, we have just landed
                self.grounded = true;
                self.vy = 1;
            } else {
                self.vy = PIXEL_SIZE;
            }
        }
    }

    fn draw(&self, off_x: i32, off_y: i32) {
        draw_rect_i32(
            self.x / PIXEL_SIZE + off_x,
            self.y / PIXEL_SIZE + off_y,
            TILE_PIXELS,
            TILE_PIXELS,
            RED,
        );
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[derive(Clone)]
pub struct SideExits {
    pub left: Option<usize>,
    pub right: Option<usize>,
    pub up: Option<usize>,
    pub down: Option<usize>,
}

#[derive(Clone)]
pub struct SideOffsets {
    pub left: Option<i32>,
    pub right: Option<i32>,
    pub up: Option<i32>,
    pub down: Option<i32>,
}

#[derive(Clone)]
pub struct LevelRaw {
    name: String,
    tiles: Vec<Vec<Vec<Tile>>>,
    exits: SideExits,
    door_exits: Vec<usize>,
}

impl LevelRaw {
    pub fn side_offsets(&self) -> SideOffsets {
        let mut side_offsets = SideOffsets {
            left: None,
            right: None,
            up: None,
            down: None,
        };

        for layer in self.tiles.iter() {
            for (y, row) in layer.iter().enumerate() {
                for (x, tile) in row.iter().enumerate() {
                    match tile {
                        Tile::ExitAnchor => {
                            if x == 0 {
                                side_offsets.left = Some(y as i32 * TILE_SIZE)
                            } else if x == row.len() - 1 {
                                side_offsets.right = Some(y as i32 * TILE_SIZE)
                            } else if y == 0 {
                                side_offsets.up = Some(x as i32 * TILE_SIZE)
                            } else if y == layer.len() - 1 {
                                side_offsets.down = Some(x as i32 * TILE_SIZE)
                            }
                        }
                        _ => (),
                    }
                }
            }
        }
        side_offsets
    }
    pub fn draw(&self, off_x: i32, off_y: i32) {
        for layer in self.tiles.iter() {
            for (y, row) in layer.iter().enumerate() {
                for (x, tile) in row.iter().enumerate() {
                    tile.draw(
                        x as i32 * TILE_PIXELS + off_x,
                        y as i32 * TILE_PIXELS + off_y,
                    )
                }
            }
        }
    }
    pub fn propagate_draw(
        &self,
        off_x: i32,
        off_y: i32,
        levels: &Vec<LevelRaw>,
        seen: &mut Vec<usize>,
        skip_actually_drawing: bool,
    ) {
        if !skip_actually_drawing {
            self.draw(off_x, off_y)
        }
        if self.exits.left.is_some() && !seen.contains(&self.exits.left.expect("balls")) {
            let ind = self.exits.left.expect("balls");
            seen.push(ind);

            let offset = levels[ind].tiles[0][0].len() as i32 * TILE_PIXELS;
            let perp_offset = (self.side_offsets().left.expect("balls")
                - levels[ind].side_offsets().right.expect("balls"))
                / PIXEL_SIZE;

            levels[ind].propagate_draw(off_x - offset, off_y + perp_offset, levels, seen, false);
        };
        if self.exits.right.is_some() && !seen.contains(&self.exits.right.expect("balls")) {
            let ind = self.exits.right.expect("balls");
            seen.push(ind);

            let offset = self.tiles[0][0].len() as i32 * TILE_PIXELS;
            let perp_offset = (self.side_offsets().right.expect("balls")
                - levels[ind].side_offsets().left.expect("balls"))
                / PIXEL_SIZE;

            levels[ind].propagate_draw(off_x + offset, off_y + perp_offset, levels, seen, false);
        };
        if self.exits.up.is_some() && !seen.contains(&self.exits.up.expect("balls")) {
            let ind = self.exits.up.expect("balls");
            seen.push(ind);

            let offset = levels[ind].tiles[0].len() as i32 * TILE_PIXELS;
            let perp_offset = (self.side_offsets().up.expect("balls")
                - levels[ind].side_offsets().down.expect("balls"))
                / PIXEL_SIZE;

            levels[ind].propagate_draw(off_x + perp_offset, off_y - offset, levels, seen, false);
        };
        if self.exits.down.is_some() && !seen.contains(&self.exits.down.expect("balls")) {
            let ind = self.exits.down.expect("balls");
            seen.push(ind);

            let offset = self.tiles[0].len() as i32 * TILE_PIXELS;
            let perp_offset = (self.side_offsets().down.expect("balls")
                - levels[ind].side_offsets().up.expect("balls"))
                / PIXEL_SIZE;

            levels[ind].propagate_draw(off_x + perp_offset, off_y + offset, levels, seen, false);
        };
    }
}

pub struct Level {
    name: String,
    tiles: Vec<Vec<Vec<Tile>>>,
    objects: Vec<Box<dyn Object>>,
    pub side_exits: SideExits,
    pub side_offsets: SideOffsets,
}

impl Level {
    pub fn dimensions(&self) -> (i32, i32) {
        (self.tiles[0][0].len() as i32, self.tiles[0].len() as i32)
    }
    pub fn focus_position(&self) -> (i32, i32) {
        for obj in self.objects.iter() {
            if obj.get_type() == "PLAYER" {
                let aabb = obj.get_aabb();
                return (aabb.x + aabb.w / 2, aabb.y + aabb.h / 2);
            }
        }
        (0, 0)
    }
    pub fn player_vel(&self) -> (i32, i32) {
        for obj in self.objects.iter() {
            if obj.get_type() == "PLAYER" {
                let player = obj
                    .as_any()
                    .downcast_ref::<Player>()
                    .expect("it is a player");
                return (player.vx, player.vy);
            }
        }
        (0, 0)
    }
    pub fn player_obj(&mut self) -> &mut Player {
        for obj in self.objects.iter_mut() {
            if obj.get_type() == "PLAYER" {
                let player = obj
                    .as_any_mut()
                    .downcast_mut::<Player>()
                    .expect("it is a player");
                return player;
            }
        }
        panic!("erm")
    }
    pub fn draw(&self, off_x: i32, off_y: i32) {
        for layer in self.tiles.iter() {
            for (y, row) in layer.iter().enumerate() {
                for (x, tile) in row.iter().enumerate() {
                    tile.draw(
                        x as i32 * TILE_PIXELS + off_x,
                        y as i32 * TILE_PIXELS + off_y,
                    )
                }
            }
        }
        for o in self.objects.iter() {
            o.draw(off_x, off_y)
        }
    }
    pub fn update(&mut self) {
        for o in self.objects.iter_mut() {
            o.update(&self.tiles)
        }
    }

    pub fn from_level_raw(l: LevelRaw) -> Self {
        let mut tiles = vec![];
        let mut objects: Vec<Box<dyn Object>> = vec![];
        let mut side_offsets = SideOffsets {
            left: None,
            right: None,
            up: None,
            down: None,
        };

        for layer in l.tiles.iter() {
            let mut l_tiles = vec![];

            for (y, row) in layer.iter().enumerate() {
                let mut row_tiles = vec![];
                for (x, tile) in row.iter().enumerate() {
                    match tile {
                        Tile::Player => {
                            let obj = Player {
                                x: x as i32 * TILE_SIZE,
                                y: y as i32 * TILE_SIZE,
                                vx: 0,
                                vy: 0,
                                grounded: false,
                            };
                            objects.push(Box::new(obj));
                            row_tiles.push(Tile::Empty);
                        }
                        Tile::ExitAnchor => {
                            if x == 0 {
                                side_offsets.left = Some(y as i32 * TILE_SIZE)
                            } else if x == row.len() - 1 {
                                side_offsets.right = Some(y as i32 * TILE_SIZE)
                            } else if y == 0 {
                                side_offsets.up = Some(x as i32 * TILE_SIZE)
                            } else if y == layer.len() - 1 {
                                side_offsets.down = Some(x as i32 * TILE_SIZE)
                            }
                            row_tiles.push(Tile::Empty);
                        }
                        t => row_tiles.push(*t),
                    }
                }
                l_tiles.push(row_tiles);
            }

            tiles.push(l_tiles);
        }

        Level {
            name: l.name,
            tiles,
            objects,
            side_exits: l.exits,
            side_offsets,
        }
    }
}

fn load_level(path: &str) -> LevelRaw {
    let level_contents = std::fs::read_to_string(path).unwrap();
    let level_contents = level_contents.trim().replace("\r\n", "\n");

    let mut parts = level_contents.split("\n===\n");

    let name = parts.next().expect("balls").to_string();

    let tilemap: HashMap<char, Tile> = parts
        .next()
        .expect("balls")
        .lines()
        .map(|a| {
            // println!("{}", a);
            let mut halves = a.split(":");
            let left_half = halves
                .next()
                .expect("should have two halves")
                .trim()
                .chars()
                .nth(0)
                .unwrap_or(' ');

            let right_half = halves.next().expect("should have two halves").trim();
            let right_half = Tile::from_string(right_half);

            (left_half, right_half)
        })
        .collect();

    let (exits, door_exits) = {
        let mut exits = SideExits {
            left: None,
            right: None,
            up: None,
            down: None,
        };
        let mut door_exits = vec![];

        for l in parts.next().expect("balls").lines() {
            let mut halves = l.split(":");
            let left_half = halves.next().expect("should have two halves").trim();

            let right_half = halves.next().expect("should have two halves").trim();
            let right_half: usize = right_half.parse().expect("should have valid index");

            match left_half {
                "left" => exits.left = Some(right_half),
                "right" => exits.right = Some(right_half),
                "up" => exits.up = Some(right_half),
                "down" => exits.down = Some(right_half),
                "door" => door_exits.push(right_half),
                _ => (),
            }
        }

        (exits, door_exits)
    };

    let mut tiles = vec![];
    while let Some(layer_content) = parts.next() {
        let mut layer = vec![];
        for row in layer_content.lines() {
            let row = row.chars();
            let row = row
                .map(|c| *tilemap.get(&c).unwrap_or(&Tile::Empty))
                .collect();
            layer.push(row);
        }
        tiles.push(layer)
    }

    // println!("{:?}", tiles);
    // let tiles: Vec<Vec<Tile>> = parts
    //     .next()
    //     .expect("balls")
    //     .lines()
    //     .map(|l| {
    //         l.chars().collect::<Vec<char>>()[..]
    //             .chunks(2)
    //             .map(|pair| Tile::from_chars(pair[0], pair[1]))
    //             .collect()
    //     })
    //     .collect();

    LevelRaw {
        name,
        tiles,
        exits,
        door_exits,
    }
}

pub struct Levelset {
    pub levels: Vec<LevelRaw>,
}

pub fn load_levelset(path: &str) -> Levelset {
    let levelset_file = std::fs::read_to_string(format!("{}/levels.levelset", path)).unwrap();
    let levelset_file = levelset_file.trim();

    println!("{}", levelset_file);

    let mut levels = vec![];
    for l in levelset_file.lines() {
        let l = l.trim();

        println!("reading {}/{}.lvl", path, l);

        levels.push(load_level(&format!("{}/{}.lvl", path, l)));
    }

    Levelset { levels }
}
