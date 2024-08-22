use super::{MAX_PLAYER_SPEED, PIXEL_SIZE, PLAYER_ACCEL, TILE_PIXELS, TILE_SIZE};
use crate::texture_cache;
use macroquad::prelude::*;
use std::collections::HashMap;

fn draw_rect_i32(x: i32, y: i32, w: i32, h: i32, c: Color) {
    draw_rectangle(x as f32, y as f32, w as f32, h as f32, c)
}

#[derive(Copy, Clone, Debug)]
pub enum Tile {
    Empty,
    Player,

    Wall,
    Wall2,
    Wall3,
    Wall4,

    BackWall,
    BackWall2,
    BackWall3,
    BackWall4,

    DoorGeneric,
    Door(usize),

    ExitAnchor,

    Spikes,
}

impl Tile {
    pub fn is_solid(&self) -> bool {
        match self {
            Self::Wall => true,
            Self::Wall2 => true,
            Self::Wall3 => true,
            Self::Wall4 => true,

            _ => false,
        }
    }

    pub fn is_deadly(&self) -> bool {
        match self {
            Self::Spikes => true,
            _ => false,
        }
    }

    pub fn from_string(s: &str) -> Self {
        match s {
            "wall" => Self::Wall,
            "wall2" => Self::Wall2,
            "wall3" => Self::Wall3,
            "wall4" => Self::Wall4,
            "backwall" => Self::BackWall,
            "backwall2" => Self::BackWall2,
            "backwall3" => Self::BackWall3,
            "backwall4" => Self::BackWall4,
            "door" => Self::DoorGeneric,
            "player" => Self::Player,
            "exit_anchor" => Self::ExitAnchor,
            "spikes" => Self::Spikes,
            _ => Self::Empty,
        }
    }

    pub fn draw(&self, x: i32, y: i32, _textures: &mut HashMap<String, Texture2D>) {
        match self {
            Self::Empty => (),
            Self::Wall => draw_rect_i32(x, y, TILE_PIXELS, TILE_PIXELS, BLACK),
            Self::BackWall => draw_rect_i32(x, y, TILE_PIXELS, TILE_PIXELS, GRAY),
            Self::Door(_) | Self::DoorGeneric => {
                draw_rect_i32(x, y, TILE_PIXELS, TILE_PIXELS, BROWN)
            }
            Self::Spikes => draw_rect_i32(x, y, TILE_PIXELS, TILE_PIXELS, RED),
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

pub fn check_tilemap_death(c_box: AABB, map: &Vec<Vec<Vec<Tile>>>) -> bool {
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
                if l[ty][tx].is_deadly() {
                    return true;
                }
            }
        }
    }

    false
}

pub fn check_door(c_box: AABB, map: &Vec<Vec<Vec<Tile>>>) -> Option<Tile> {
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
                if let Tile::Door(_) = l[ty][tx] {
                    return Some(l[ty][tx]);
                }
            }
        }
    }

    None
}

pub fn find_door(index: usize, map: &Vec<Vec<Vec<Tile>>>) -> Option<(i32, i32)> {
    for layer in map.iter() {
        for (y, row) in layer.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                if let Tile::Door(i) = tile {
                    if *i == index {
                        return Some((x as i32, y as i32));
                    }
                }
            }
        }
    }

    None
}

pub trait Object {
    fn get_type(&self) -> &'static str;

    fn get_aabb(&self) -> AABB;

    fn update(&mut self, _keys_pressed: &mut HashMap<KeyCode, bool>, _tiles: &Vec<Vec<Vec<Tile>>>) {
    }

    fn draw(&self, _off_x: i32, _off_y: i32, _texture: &mut HashMap<String, Texture2D>) {}

    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

pub struct Player {
    pub x: i32,
    pub y: i32,

    pub vx: i32,
    pub vy: i32,

    pub anim_timer: i32,

    pub grounded: bool,

    pub freeze_timer: i32,
    pub wall_sliding: i32,

    pub air_frames: i32,
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

    fn update(&mut self, keys_pressed: &mut HashMap<KeyCode, bool>, tiles: &Vec<Vec<Vec<Tile>>>) {
        // accelerate left and right
        self.freeze_timer -= 1;
        if self.freeze_timer <= 0 {
            if is_key_down(KeyCode::Left) && !is_key_down(KeyCode::Right) {
                if self.wall_sliding > 0 {
                    self.wall_sliding = 0
                }
                if self.vx >= -MAX_PLAYER_SPEED {
                    self.vx = (-MAX_PLAYER_SPEED).max(self.vx - PLAYER_ACCEL);
                } else {
                    self.vx += TILE_SIZE / 128;
                }
            } else if is_key_down(KeyCode::Right) && !is_key_down(KeyCode::Left) {
                if self.wall_sliding > 0 {
                    self.wall_sliding = 0
                }
                if self.vx <= MAX_PLAYER_SPEED {
                    self.vx = (MAX_PLAYER_SPEED).min(self.vx + PLAYER_ACCEL);
                } else {
                    self.vx -= TILE_SIZE / 128;
                }
            }
        }

        if !is_key_down(KeyCode::Left) && !is_key_down(KeyCode::Right) && self.freeze_timer <= 0 {
            self.vx *= 11;
            self.vx /= 16;
            self.anim_timer = 0;
            if self.wall_sliding != 0 {
                self.vx = self.wall_sliding;
            }
        } else {
            self.anim_timer += self.vx.abs() / PLAYER_ACCEL;
        }

        if is_key_down(KeyCode::Down) {
            self.vy += TILE_SIZE / 16;
        } else if is_key_down(KeyCode::Z) {
            self.vy += TILE_SIZE / 16 / 5;
        } else {
            self.freeze_timer -= 5;
            self.vy += TILE_SIZE / 16 / 2;
        }
        // cap vx and vy at one tile per game step
        // in practice this will never be hit
        self.vx = self.vx.clamp(-TILE_SIZE, TILE_SIZE);
        self.vy = self.vy.clamp(-TILE_SIZE, TILE_SIZE);
        if self.grounded {
            self.vy = self.vy.min(TILE_SIZE / 6)
        }

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
            self.freeze_timer = 0;
            if (self.vx < 0 && is_key_down(KeyCode::Left))
                || (self.vx > 0 && is_key_down(KeyCode::Right))
            {
                self.wall_sliding = self.vx.signum();
            }
            self.vx = 0;
        } else if remaining_movement.abs() > 0 {
            if self.wall_sliding != 0 {
                self.vx = -self.vx.signum() * 4;
            }
            self.wall_sliding = 0;
        }

        if self.wall_sliding != 0 {
            self.vx = -self.wall_sliding;
            if is_key_down(KeyCode::Down) {
                self.vy = self.vy.min(TILE_SIZE / 4);
            } else {
                self.vy = self.vy.min(TILE_SIZE / 32);
            }
            if *keys_pressed.entry(KeyCode::Z).or_insert(false) && !self.grounded {
                self.grounded = false;
                self.freeze_timer = 14;
                if self.wall_sliding < 0 {
                    self.vx = TILE_SIZE * 5 / 16;
                    self.vy = -TILE_SIZE * 4 / 16;
                } else if self.wall_sliding > 0 {
                    self.vx = -TILE_SIZE * 5 / 16;
                    self.vy = -TILE_SIZE * 4 / 16;
                }
                self.wall_sliding = 0;
            }
        }

        if self.grounded && *keys_pressed.entry(KeyCode::Z).or_insert(false) {
            self.vy = -TILE_SIZE * 5 / 16;
            if is_key_down(KeyCode::Left) && is_key_down(KeyCode::Right) {
                self.vx *= 9;
                self.vx /= 8;
            }
            self.grounded = false;
        }

        // same but vertical

        // allow for the player to store grounded value while wall sliding
        if self.wall_sliding == 0 {
            self.air_frames += 1;
        }
        if self.air_frames > 15 {
            self.grounded = false
        }

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
                self.air_frames = 0;
            } else {
                self.vy = PIXEL_SIZE;
            }
            self.wall_sliding = 0;
        }
    }

    fn draw(&self, off_x: i32, off_y: i32, textures: &mut HashMap<String, Texture2D>) {
        // draw_rect_i32(
        //     self.x / PIXEL_SIZE + off_x,
        //     self.y / PIXEL_SIZE + off_y,
        //     TILE_PIXELS,
        //     TILE_PIXELS,
        //     BLUE,
        // );

        let t = texture_cache!(textures, "assets/player.png");

        let mut draw_offset = (0, 0);

        if is_key_down(KeyCode::Left) && is_key_down(KeyCode::Right) {
            if self.vx < 4 {
                draw_offset = (16, 48)
            } else if self.vx > 4 {
                draw_offset = (0, 48)
            } else {
                draw_offset = (16, 0)
            }
        } else if !self.grounded {
            if self.vx < 4 {
                draw_offset = (16, 80)
            } else if self.vx > 4 {
                draw_offset = (0, 80)
            } else {
                draw_offset = (0, 0)
            }
        } else if is_key_down(KeyCode::Left) {
            if self.anim_timer % 64 < 43 {
                draw_offset = (0, 32)
            } else {
                draw_offset = (16, 32)
            }
        } else if is_key_down(KeyCode::Right) {
            if self.anim_timer % 64 < 43 {
                draw_offset = (0, 16)
            } else {
                draw_offset = (16, 16)
            }
        }
        if self.wall_sliding < 0 {
            draw_offset = (16, 64)
        } else if self.wall_sliding > 0 {
            draw_offset = (0, 64)
        }

        draw_texture_ex(
            &t,
            (self.x / PIXEL_SIZE + off_x) as f32,
            (self.y / PIXEL_SIZE + off_y) as f32,
            WHITE,
            DrawTextureParams {
                source: Some(Rect {
                    x: draw_offset.0 as f32,
                    y: draw_offset.1 as f32,
                    w: 16.,
                    h: 16.,
                }),
                ..Default::default()
            },
        )
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
    pub fn draw(&self, off_x: i32, off_y: i32, textures: &mut HashMap<String, Texture2D>) {
        for layer in self.tiles.iter() {
            for (y, row) in layer.iter().enumerate() {
                for (x, tile) in row.iter().enumerate() {
                    tile.draw(
                        x as i32 * TILE_PIXELS + off_x,
                        y as i32 * TILE_PIXELS + off_y,
                        textures,
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
        textures: &mut HashMap<String, Texture2D>,
    ) {
        if !skip_actually_drawing {
            self.draw(off_x, off_y, textures)
        }
        if self.exits.left.is_some() && !seen.contains(&self.exits.left.expect("is some")) {
            let ind = self.exits.left.expect("is some");
            seen.push(ind);

            let offset = levels[ind].tiles[0][0].len() as i32 * TILE_PIXELS;
            let perp_offset = (self.side_offsets().left.expect("is some")
                - levels[ind]
                    .side_offsets()
                    .right
                    .expect("corresponding should have exit anchor"))
                / PIXEL_SIZE;

            levels[ind].propagate_draw(
                off_x - offset,
                off_y + perp_offset,
                levels,
                seen,
                false,
                textures,
            );
        };
        if self.exits.right.is_some() && !seen.contains(&self.exits.right.expect("is some")) {
            let ind = self.exits.right.expect("is some");
            seen.push(ind);

            let offset = self.tiles[0][0].len() as i32 * TILE_PIXELS;
            let perp_offset = (self.side_offsets().right.expect("is some")
                - levels[ind]
                    .side_offsets()
                    .left
                    .expect("corresponding should have exit anchor"))
                / PIXEL_SIZE;

            levels[ind].propagate_draw(
                off_x + offset,
                off_y + perp_offset,
                levels,
                seen,
                false,
                textures,
            );
        };
        if self.exits.up.is_some() && !seen.contains(&self.exits.up.expect("is some")) {
            let ind = self.exits.up.expect("is some");
            seen.push(ind);

            let offset = levels[ind].tiles[0].len() as i32 * TILE_PIXELS;
            let perp_offset = (self.side_offsets().up.expect("is some")
                - levels[ind]
                    .side_offsets()
                    .down
                    .expect("corresponding should have exit anchor"))
                / PIXEL_SIZE;

            levels[ind].propagate_draw(
                off_x + perp_offset,
                off_y - offset,
                levels,
                seen,
                false,
                textures,
            );
        };
        if self.exits.down.is_some() && !seen.contains(&self.exits.down.expect("is some")) {
            let ind = self.exits.down.expect("is some");
            seen.push(ind);

            let offset = self.tiles[0].len() as i32 * TILE_PIXELS;
            let perp_offset = (self.side_offsets().down.expect("is some")
                - levels[ind]
                    .side_offsets()
                    .up
                    .expect("corresponding should have exit anchor"))
                / PIXEL_SIZE;

            levels[ind].propagate_draw(
                off_x + perp_offset,
                off_y + offset,
                levels,
                seen,
                false,
                textures,
            );
        };
    }
}

pub struct Level {
    pub name: String,
    pub tiles: Vec<Vec<Vec<Tile>>>,
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
        panic!("we should have a player! if we don't something has gone very wrong")
    }
    pub fn draw(&self, off_x: i32, off_y: i32, textures: &mut HashMap<String, Texture2D>) {
        for layer in self.tiles.iter() {
            for (y, row) in layer.iter().enumerate() {
                for (x, tile) in row.iter().enumerate() {
                    tile.draw(
                        x as i32 * TILE_PIXELS + off_x,
                        y as i32 * TILE_PIXELS + off_y,
                        textures,
                    )
                }
            }
        }
        for o in self.objects.iter() {
            o.draw(off_x, off_y, textures)
        }
    }
    pub fn update(&mut self, keys_pressed: &mut HashMap<KeyCode, bool>) {
        for o in self.objects.iter_mut() {
            o.update(keys_pressed, &self.tiles)
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
        let mut door_exits = l.door_exits.iter();

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
                                freeze_timer: 0,
                                wall_sliding: 0,
                                anim_timer: 0,
                                air_frames: 0,
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
                        Tile::DoorGeneric => {
                            let ind = door_exits
                                .next()
                                .expect("should have a corresponding door entrance");
                            row_tiles.push(Tile::Door(*ind));
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

    let name = parts.next().expect("should have part").to_string();

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

        for l in parts.next().expect("should have part").lines() {
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
    pub name: String,
    pub levels: Vec<LevelRaw>,
}

pub fn load_levelset(path: &str) -> Levelset {
    let levelset_file = std::fs::read_to_string(format!("{}/levels.levelset", path)).unwrap();
    let levelset_file = levelset_file.trim().replace("\r\n", "\n");

    println!("{}", levelset_file);

    let mut parts = levelset_file.split("\n===\n");

    let name = parts.next().expect("should have part").to_string();

    let mut levels = vec![];
    for l in parts.next().expect("should have part").lines() {
        let l = l.trim();

        println!("reading {}/{}.lvl", path, l);

        levels.push(load_level(&format!("{}/{}.lvl", path, l)));
    }

    Levelset { name, levels }
}
