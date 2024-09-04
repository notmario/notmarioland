use super::{MAX_PLAYER_SPEED, PIXEL_SIZE, PLAYER_ACCEL, TILE_PIXELS, TILE_SIZE};
use crate::{texture_cache, Adjacencies, Theme};
use macroquad::prelude::*;
use std::collections::HashMap;

fn draw_rect_i32(x: i32, y: i32, w: i32, h: i32, c: Color) {
    draw_rectangle(x as f32, y as f32, w as f32, h as f32, c)
}

pub struct GlobalState {
    pub changed_tiles: HashMap<(usize, usize, usize, usize), Tile>,
    pub keys: [i32; 6],
    pub timer: i32,
    pub secrets: i32,
    pub jumps: i32,
}

impl GlobalState {
    pub fn new() -> Self {
        GlobalState {
            changed_tiles: HashMap::new(),
            keys: [0; 6],
            timer: 0,
            secrets: 0,
            jumps: 0,
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}
impl Direction {
    fn h_vel(v: i32) -> Self {
        if v < 0 {
            Self::Left
        } else {
            Self::Right
        }
    }
    fn v_vel(v: i32) -> Self {
        if v <= 0 {
            Self::Up
        } else {
            Self::Down
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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

    OneWayLeft,
    OneWayRight,
    OneWayUp,
    OneWayDown,

    RedKey,
    YellowKey,
    GreenKey,
    CyanKey,
    BlueKey,
    MagentaKey,

    RedLock,
    YellowLock,
    GreenLock,
    CyanLock,
    BlueLock,
    MagentaLock,

    SawLauncherLeft,
    SawLauncherRight,
    SawLauncherUp,
    SawLauncherDown,

    SlowSawLauncherLeft,
    SlowSawLauncherRight,
    SlowSawLauncherUp,
    SlowSawLauncherDown,

    Secret,
    Goal,

    JumpArrow,
    JumpArrowOutline,
}

fn tilemap_draw(t: &Texture2D, x: i32, y: i32, touching: &Adjacencies) {
    let mut render_offset = (0, 0);
    if touching.up {
        render_offset.1 += 32
    };
    if touching.down {
        render_offset.1 += 16
    };
    if touching.left {
        render_offset.0 += 32
    };
    if touching.right {
        render_offset.0 += 16
    };

    draw_texture_ex(
        &t,
        x as f32,
        y as f32,
        WHITE,
        DrawTextureParams {
            source: Some(Rect {
                x: render_offset.0 as f32,
                y: render_offset.1 as f32,
                w: 16.,
                h: 16.,
            }),
            ..Default::default()
        },
    )
}

impl Tile {
    pub fn is_solid(
        &self,
        b_box: AABB,
        _c_box: AABB,
        my_aabb: AABB,
        direction: Direction,
        gs: &GlobalState,
    ) -> bool {
        match self {
            Self::Wall => true,
            Self::Wall2 => true,
            Self::Wall3 => true,
            Self::Wall4 => true,

            // note. the directions are seemingly reversed here
            // this is because "left" refers to it being on the left of the block
            // and it should thus block if you're going right
            Self::OneWayLeft => (b_box.x + b_box.w <= my_aabb.x) && direction == Direction::Right,
            Self::OneWayRight => (b_box.x >= my_aabb.x + my_aabb.w) && direction == Direction::Left,
            Self::OneWayUp => (b_box.y + b_box.h <= my_aabb.y) && direction == Direction::Down,
            Self::OneWayDown => (b_box.y >= my_aabb.y + my_aabb.h) && direction == Direction::Up,

            Self::RedLock => gs.keys[0] == 0,
            Self::YellowLock => gs.keys[1] == 0,
            Self::GreenLock => gs.keys[2] == 0,
            Self::CyanLock => gs.keys[3] == 0,
            Self::BlueLock => gs.keys[4] == 0,
            Self::MagentaLock => gs.keys[5] == 0,

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

            "onewayleft" => Self::OneWayLeft,
            "onewayright" => Self::OneWayRight,
            "onewaydown" => Self::OneWayDown,
            "onewayup" => Self::OneWayUp,

            "redkey" => Self::RedKey,
            "yellowkey" => Self::YellowKey,
            "greenkey" => Self::GreenKey,
            "cyankey" => Self::CyanKey,
            "bluekey" => Self::BlueKey,
            "magentakey" => Self::MagentaKey,

            "redlock" => Self::RedLock,
            "yellowlock" => Self::YellowLock,
            "greenlock" => Self::GreenLock,
            "cyanlock" => Self::CyanLock,
            "bluelock" => Self::BlueLock,
            "magentalock" => Self::MagentaLock,

            "sawlauncherleft" => Self::SawLauncherLeft,
            "sawlauncherright" => Self::SawLauncherRight,
            "sawlauncherup" => Self::SawLauncherUp,
            "sawlauncherdown" => Self::SawLauncherDown,

            "slowsawlauncherleft" => Self::SlowSawLauncherLeft,
            "slowsawlauncherright" => Self::SlowSawLauncherRight,
            "slowsawlauncherup" => Self::SlowSawLauncherUp,
            "slowsawlauncherdown" => Self::SlowSawLauncherDown,

            "secret" => Self::Secret,
            "goal" => Self::Goal,

            "jumparrow" => Self::JumpArrow,

            _ => Self::Empty,
        }
    }
    // pull this out into its own function because yes
    fn sprite(&self) -> Option<&'static str> {
        match self {
            Self::Door(_) | Self::DoorGeneric => Some("assets/door.png"),
            Self::Spikes => Some("assets/spike.png"),

            Self::RedKey => Some("assets/redkey.png"),
            Self::YellowKey => Some("assets/yellowkey.png"),
            Self::GreenKey => Some("assets/greenkey.png"),
            Self::CyanKey => Some("assets/cyankey.png"),
            Self::BlueKey => Some("assets/bluekey.png"),
            Self::MagentaKey => Some("assets/magentakey.png"),

            Self::RedLock => Some("assets/redlock.png"),
            Self::YellowLock => Some("assets/yellowlock.png"),
            Self::GreenLock => Some("assets/greenlock.png"),
            Self::CyanLock => Some("assets/cyanlock.png"),
            Self::BlueLock => Some("assets/bluelock.png"),
            Self::MagentaLock => Some("assets/magentalock.png"),

            Self::SawLauncherLeft => Some("assets/sawlauncherleft.png"),
            Self::SawLauncherRight => Some("assets/sawlauncherright.png"),
            Self::SawLauncherUp => Some("assets/sawlauncherup.png"),
            Self::SawLauncherDown => Some("assets/sawlauncherdown.png"),

            Self::SlowSawLauncherLeft => Some("assets/slowsawlauncherleft.png"),
            Self::SlowSawLauncherRight => Some("assets/slowsawlauncherright.png"),
            Self::SlowSawLauncherUp => Some("assets/slowsawlauncherup.png"),
            Self::SlowSawLauncherDown => Some("assets/slowsawlauncherdown.png"),

            Self::Secret => Some("assets/secret.png"),
            Self::Goal => Some("assets/goal.png"),

            Self::JumpArrow => Some("assets/jumparrow.png"),
            Self::JumpArrowOutline => Some("assets/jumparrowoutline.png"),

            _ => None,
        }
    }

    pub fn draw(
        &self,
        x: i32,
        y: i32,
        textures: &mut HashMap<String, Texture2D>,
        theme: &Theme,
        touching: &Adjacencies,
    ) {
        match self {
            Self::Empty => (),
            Self::Wall | Self::Wall2 | Self::Wall3 | Self::Wall4 => {
                let te = match self {
                    Self::Wall => &theme.wall_1,
                    Self::Wall2 => &theme.wall_2,
                    Self::Wall3 => &theme.wall_3,
                    Self::Wall4 => &theme.wall_4,
                    _ => unreachable!(),
                };
                if te.is_some() {
                    let t = texture_cache!(textures, te.as_ref().expect("it exists"));
                    tilemap_draw(&t, x, y, touching);
                } else {
                    draw_rect_i32(
                        x,
                        y,
                        TILE_PIXELS,
                        TILE_PIXELS,
                        if self == &Self::Wall4 { BLUE } else { BLACK },
                    )
                }
            }
            Self::BackWall | Self::BackWall2 | Self::BackWall3 | Self::BackWall4 => {
                let te = match self {
                    Self::BackWall => &theme.back_wall_1,
                    Self::BackWall2 => &theme.back_wall_2,
                    Self::BackWall3 => &theme.back_wall_3,
                    Self::BackWall4 => &theme.back_wall_4,
                    _ => unreachable!(),
                };
                if te.is_some() {
                    let t = texture_cache!(textures, te.as_ref().expect("it exists"));
                    tilemap_draw(&t, x, y, touching);
                } else {
                    draw_rect_i32(x, y, TILE_PIXELS, TILE_PIXELS, GRAY)
                }
            }
            Self::OneWayLeft => draw_rect_i32(x, y, TILE_PIXELS / 8, TILE_PIXELS, BLACK),
            Self::OneWayUp => draw_rect_i32(x, y, TILE_PIXELS, TILE_PIXELS / 8, BLACK),
            Self::OneWayRight => draw_rect_i32(
                x + TILE_PIXELS * 7 / 8,
                y,
                TILE_PIXELS / 8,
                TILE_PIXELS,
                BLACK,
            ),
            Self::OneWayDown => draw_rect_i32(
                x,
                y + TILE_PIXELS * 7 / 8,
                TILE_PIXELS,
                TILE_PIXELS / 8,
                BLACK,
            ),
            _ => {
                let t = self.sprite();
                if t.is_some() {
                    let t = texture_cache!(textures, t.expect("is some"));

                    draw_texture(&t, x as f32, y as f32, WHITE)
                }
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct AABB {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

// commented out so the compiler shuts up

impl AABB {
    fn intersect(&self, other: &Self) -> bool {
        self.x < other.x + other.w
            && self.x + self.w > other.x
            && self.y < other.y + other.h
            && self.y + self.h > other.y
    }
    fn smaller_by(&self, amt: i32) -> Self {
        AABB {
            x: self.x + amt,
            y: self.y + amt,
            w: self.w - amt * 2,
            h: self.h - amt * 2,
        }
    }
    // fn shift_by(&self, off: (i32, i32)) -> Self {
    //     AABB {
    //         x: self.x + off.0,
    //         y: self.y + off.1,
    //         w: self.w,
    //         h: self.h,
    //     }
    // }
}

fn check_tilemap_condition<F>(c_box: AABB, map: &Vec<Vec<Vec<Tile>>>, condition: F) -> bool
where
    F: Fn(Tile, usize, usize) -> bool,
{
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
                if condition(l[ty][tx], ty, tx) {
                    return true;
                }
            }
        }
    }

    false
}

fn check_tilemap_collision(
    b_box: AABB,
    c_box: AABB,
    map: &Vec<Vec<Vec<Tile>>>,
    direction: Direction,
    gs: &GlobalState,
) -> bool {
    check_tilemap_condition(c_box, map, |t, ty, tx| {
        let my_aabb = AABB {
            x: tx as i32 * TILE_SIZE,
            y: ty as i32 * TILE_SIZE,
            w: TILE_SIZE,
            h: TILE_SIZE,
        };

        t.is_solid(b_box, c_box, my_aabb, direction, gs)
    })
}

fn check_tilemap_wallslideable(c_box: AABB, map: &Vec<Vec<Vec<Tile>>>) -> bool {
    !check_tilemap_condition(c_box, map, |t, _, _| t == Tile::Wall4)
}

pub fn check_tilemap_death(c_box: AABB, map: &Vec<Vec<Vec<Tile>>>) -> bool {
    check_tilemap_condition(c_box, map, |t, _, _| t.is_deadly())
}

pub fn check_tilemap_win(c_box: AABB, map: &Vec<Vec<Vec<Tile>>>) -> bool {
    check_tilemap_condition(c_box, map, |t, _, _| t == Tile::Goal)
}

pub fn check_object_death(c_box: AABB, objects: &Vec<Box<dyn Object>>) -> bool {
    for o in objects {
        match o.get_type() {
            "SAW" => {
                if o.get_aabb().smaller_by(PIXEL_SIZE * 2).intersect(&c_box) {
                    return true;
                }
            }
            _ => (),
        }
    }

    false
}

pub fn flood_fill(
    x: usize,
    y: usize,
    layer: &mut Vec<Vec<Tile>>,
    with: Tile,
    subs: &mut HashMap<(usize, usize, usize, usize), Tile>,
    layer_pos: (usize, usize),
) {
    let t = layer[y][x];
    layer[y][x] = with;
    subs.insert((layer_pos.0, layer_pos.1, y, x), with);
    if y > 0 && layer[y - 1][x] == t {
        flood_fill(x, y - 1, layer, with, subs, layer_pos);
    }
    if y < layer.len() - 1 && layer[y + 1][x] == t {
        flood_fill(x, y + 1, layer, with, subs, layer_pos);
    }
    if x > 0 && layer[y][x - 1] == t {
        flood_fill(x - 1, y, layer, with, subs, layer_pos);
    }
    if x < layer[y].len() - 1 && layer[y][x + 1] == t {
        flood_fill(x + 1, y, layer, with, subs, layer_pos);
    }
}

pub fn collect_keys(
    c_box: AABB,
    my_screen: usize,
    map: &mut Vec<Vec<Vec<Tile>>>,
    gs: &mut GlobalState,
) -> bool {
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
            for (li, l) in map.iter_mut().enumerate() {
                match l[ty][tx] {
                    Tile::RedKey
                    | Tile::YellowKey
                    | Tile::GreenKey
                    | Tile::CyanKey
                    | Tile::BlueKey
                    | Tile::MagentaKey => {
                        gs.keys[match l[ty][tx] {
                            Tile::RedKey => 0,
                            Tile::YellowKey => 1,
                            Tile::GreenKey => 2,
                            Tile::CyanKey => 3,
                            Tile::BlueKey => 4,
                            Tile::MagentaKey => 5,
                            _ => unreachable!(),
                        }] += 1;
                        // println!("{:?}", gs.keys);
                        l[ty][tx] = Tile::Empty;
                        gs.changed_tiles
                            .insert((my_screen, li, ty, tx), Tile::Empty);
                    }
                    Tile::Secret => {
                        gs.secrets += 1;

                        l[ty][tx] = Tile::Empty;
                        gs.changed_tiles
                            .insert((my_screen, li, ty, tx), Tile::Empty);
                    }
                    Tile::JumpArrow => {
                        gs.jumps += 1;
                        l[ty][tx] = Tile::JumpArrowOutline;
                    }

                    _ => (),
                }
            }
        }
    }

    false
}
pub fn collect_doors(
    c_box: AABB,
    my_screen: usize,
    map: &mut Vec<Vec<Vec<Tile>>>,
    gs: &mut GlobalState,
) -> bool {
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
            for (li, l) in map.iter_mut().enumerate() {
                match l[ty][tx] {
                    Tile::RedLock
                    | Tile::YellowLock
                    | Tile::GreenLock
                    | Tile::CyanLock
                    | Tile::BlueLock
                    | Tile::MagentaLock => {
                        let i = match l[ty][tx] {
                            Tile::RedLock => 0,
                            Tile::YellowLock => 1,
                            Tile::GreenLock => 2,
                            Tile::CyanLock => 3,
                            Tile::BlueLock => 4,
                            Tile::MagentaLock => 5,
                            _ => unreachable!(),
                        };
                        if gs.keys[i] >= 1 {
                            gs.keys[i] -= 1;
                            flood_fill(
                                tx,
                                ty,
                                l,
                                Tile::Empty,
                                &mut gs.changed_tiles,
                                (my_screen, li),
                            );
                        };
                    }
                    _ => (),
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

    fn update(
        &mut self,
        _keys_pressed: &mut HashMap<KeyCode, bool>,
        _tiles: &Vec<Vec<Vec<Tile>>>,
        _global_state: &mut GlobalState,
    ) {
    }

    fn draw(
        &self,
        _off_x: i32,
        _off_y: i32,
        _texture: &mut HashMap<String, Texture2D>,
        _gs: &GlobalState,
    ) {
    }

    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    fn should_clear(&self) -> bool {
        false
    }
    fn spawn(&self, gs: &GlobalState) -> Option<Box<dyn Object>> {
        None
    }
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

    fn update(
        &mut self,
        keys_pressed: &mut HashMap<KeyCode, bool>,
        tiles: &Vec<Vec<Vec<Tile>>>,
        global_state: &mut GlobalState,
    ) {
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

        let before_aabb = self.get_aabb();

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
        if check_tilemap_collision(
            before_aabb,
            self.get_aabb(),
            tiles,
            Direction::h_vel(self.vx),
            &global_state,
        ) {
            let can_wallslide = check_tilemap_wallslideable(self.get_aabb(), tiles);
            self.x -= remaining_movement;
            self.freeze_timer = 0;
            if ((self.vx < 0 && is_key_down(KeyCode::Left))
                || (self.vx > 0 && is_key_down(KeyCode::Right)))
                && can_wallslide
            {
                self.wall_sliding = self.vx.signum();
            }
            if can_wallslide {
                self.vx = 0;
            } else if is_key_down(KeyCode::Left) && is_key_down(KeyCode::Right) {
                self.vx *= -1;
            }
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
            if *keys_pressed.entry(KeyCode::Z).or_insert(false) && self.air_frames != 0 {
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

        if (self.grounded || (global_state.jumps > 0 && self.freeze_timer <= 0))
            && *keys_pressed.entry(KeyCode::Z).or_insert(false)
        {
            self.vy = -TILE_SIZE * 5 / 16;
            if is_key_down(KeyCode::Up) {
                self.vy = -(self.vx.abs().max(TILE_SIZE * 5 / 16));
                self.vx /= 8;
            }
            if is_key_down(KeyCode::Left) && is_key_down(KeyCode::Right) {
                self.vx *= 9;
                self.vx /= 8;
            }
            if !self.grounded {
                global_state.jumps -= 1;
            }
            self.grounded = false;
        }

        // same but vertical

        // allow for the player to store grounded value while wall sliding
        if self.wall_sliding == 0 {
            self.air_frames += 1;
        } else {
            self.air_frames = self.air_frames.max(1)
        }
        if is_key_down(KeyCode::Down) {
            self.air_frames += 15
        }
        if self.air_frames > 15 {
            self.grounded = false
        }

        let before_aabb = self.get_aabb();

        let remaining_movement = if (self.y + self.vy) / TILE_SIZE != self.y / TILE_SIZE {
            let old_pos = self.y;
            if (self.y % TILE_SIZE) * 2 < TILE_SIZE {
                self.y = self.y / TILE_SIZE * TILE_SIZE;
            } else {
                self.y = (self.y / TILE_SIZE + 1) * TILE_SIZE;
            }
            (old_pos + self.vy) - (self.y)
        } else {
            self.vy
        };

        self.y += remaining_movement;
        if check_tilemap_collision(
            before_aabb,
            self.get_aabb(),
            tiles,
            Direction::v_vel(self.vy),
            &global_state,
        ) {
            self.y -= remaining_movement;
            if self.vy > 0 {
                // going down, we have just landed
                self.grounded = true;
                self.vy = 0;
                self.air_frames = 0;
            } else {
                self.vy = PIXEL_SIZE;
            }
            self.wall_sliding = 0;
        }
    }

    fn draw(
        &self,
        off_x: i32,
        off_y: i32,
        textures: &mut HashMap<String, Texture2D>,
        gs: &GlobalState,
    ) {
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
        );

        let rows = [("assets/arrowtiny.png", gs.jumps)]
            .into_iter()
            .filter(|k| k.1 != 0);
        let count: i32 = rows.clone().map(|k| k.1).sum();
        let angper = std::f32::consts::TAU / count as f32;

        let mut off = 0;
        for (i, r) in rows.enumerate() {
            let t = texture_cache!(textures, r.0);
            for j in 0..r.1 {
                let a = (off + j) as f32 * angper - gs.timer as f32 * 0.004;
                let (xoff, yoff) = a.sin_cos();
                draw_texture(
                    &t,
                    (self.x / PIXEL_SIZE + off_x + TILE_PIXELS / 4 + (xoff * 16.) as i32) as f32,
                    (self.y / PIXEL_SIZE + off_y + TILE_PIXELS / 4 + (yoff * 16.) as i32) as f32,
                    WHITE,
                )
            }
            off += r.1;
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

pub struct Saw {
    pub x: i32,
    pub y: i32,

    pub vx: i32,
    pub vy: i32,

    pub anim_timer: i32,

    pub should_remove: bool,
}

impl Object for Saw {
    fn get_type(&self) -> &'static str {
        "SAW"
    }

    fn get_aabb(&self) -> AABB {
        AABB {
            x: self.x,
            y: self.y,
            w: TILE_SIZE,
            h: TILE_SIZE,
        }
    }

    fn update(
        &mut self,
        _keys_pressed: &mut HashMap<KeyCode, bool>,
        tiles: &Vec<Vec<Vec<Tile>>>,
        global_state: &mut GlobalState,
    ) {
        // horizontal movement
        // move to tile boundary if we are moving too fast

        self.anim_timer += 1;

        let before_aabb = self.get_aabb();

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
        if check_tilemap_collision(
            before_aabb,
            self.get_aabb(),
            tiles,
            Direction::h_vel(self.vx),
            &global_state,
        ) {
            self.should_remove = true
        }

        let before_aabb = self.get_aabb();

        let remaining_movement = if (self.y + self.vy) / TILE_SIZE != self.y / TILE_SIZE {
            let old_pos = self.y;
            if (self.y % TILE_SIZE) * 2 < TILE_SIZE {
                self.y = self.y / TILE_SIZE * TILE_SIZE;
            } else {
                self.y = (self.y / TILE_SIZE + 1) * TILE_SIZE;
            }
            (old_pos + self.vy) - (self.y)
        } else {
            self.vy
        };

        self.y += remaining_movement;
        if check_tilemap_collision(
            before_aabb,
            self.get_aabb(),
            tiles,
            Direction::v_vel(self.vy),
            &global_state,
        ) {
            self.should_remove = true
        }
    }

    fn draw(
        &self,
        off_x: i32,
        off_y: i32,
        textures: &mut HashMap<String, Texture2D>,
        _gs: &GlobalState,
    ) {
        // draw_rect_i32(
        //     self.x / PIXEL_SIZE + off_x,
        //     self.y / PIXEL_SIZE + off_y,
        //     TILE_PIXELS,
        //     TILE_PIXELS,
        //     BLUE,
        // );

        let t = texture_cache!(textures, "assets/saw.png");

        let mut draw_offset = (0, 0);
        if self.anim_timer % 16 >= 8 {
            draw_offset = (16, 0)
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

    fn should_clear(&self) -> bool {
        self.should_remove
    }
}

pub struct SawLauncher {
    pub x: i32,
    pub y: i32,

    pub vx: i32,
    pub vy: i32,

    pub frames: i32,
}

impl Object for SawLauncher {
    fn get_type(&self) -> &'static str {
        "SAWLAUNCHER"
    }

    fn get_aabb(&self) -> AABB {
        AABB {
            x: self.x,
            y: self.y,
            w: TILE_SIZE,
            h: TILE_SIZE,
        }
    }

    fn update(
        &mut self,
        _keys_pressed: &mut HashMap<KeyCode, bool>,
        _tiles: &Vec<Vec<Vec<Tile>>>,
        _global_state: &mut GlobalState,
    ) {
    }

    fn draw(
        &self,
        off_x: i32,
        off_y: i32,
        textures: &mut HashMap<String, Texture2D>,
        _gs: &GlobalState,
    ) {
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn spawn(&self, gs: &GlobalState) -> Option<Box<dyn Object>> {
        if gs.timer % self.frames != 0 {
            return None;
        }

        Some(Box::new(Saw {
            x: self.x,
            y: self.y,
            vx: self.vx,
            vy: self.vy,
            anim_timer: 0,
            should_remove: false,
        }))
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
    theme: usize,
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
    pub fn secret_count(&self) -> i32 {
        let mut count = 0;

        for l in self.tiles.iter() {
            for row in l {
                for col in row {
                    if *col == Tile::Secret {
                        count += 1;
                    }
                }
            }
        }

        count
    }
    pub fn draw(
        &self,
        off_x: i32,
        off_y: i32,
        my_ind: usize,
        subs: &HashMap<(usize, usize, usize, usize), Tile>,
        textures: &mut HashMap<String, Texture2D>,
        theme: &Theme,
    ) {
        let max_y = self.tiles[0].len() - 1;
        let max_x = self.tiles[0][0].len() - 1;
        for (la, layer) in self.tiles.iter().enumerate() {
            for (y, row) in layer.iter().enumerate() {
                for (x, tile) in row.iter().enumerate() {
                    let adj = Adjacencies {
                        up: y == 0 || layer[y - 1][x] == *tile,
                        down: y == max_y || layer[y + 1][x] == *tile,
                        left: x == 0 || layer[y][x - 1] == *tile,
                        right: x == max_x || layer[y][x + 1] == *tile,
                    };
                    subs.get(&(my_ind, la, y, x)).unwrap_or(tile).draw(
                        x as i32 * TILE_PIXELS + off_x,
                        y as i32 * TILE_PIXELS + off_y,
                        textures,
                        theme,
                        &adj,
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
        my_ind: usize,
        subs: &HashMap<(usize, usize, usize, usize), Tile>,
        skip_actually_drawing: bool,
        textures: &mut HashMap<String, Texture2D>,
        themes: &[Theme],
    ) {
        if !skip_actually_drawing {
            self.draw(off_x, off_y, my_ind, subs, textures, &themes[self.theme])
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
                ind,
                subs,
                false,
                textures,
                themes,
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
                ind,
                subs,
                false,
                textures,
                themes,
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
                ind,
                subs,
                false,
                textures,
                themes,
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
                ind,
                subs,
                false,
                textures,
                themes,
            );
        };
    }
}

pub struct Level {
    pub name: String,
    pub tiles: Vec<Vec<Vec<Tile>>>,
    pub objects: Vec<Box<dyn Object>>,
    pub side_exits: SideExits,
    pub side_offsets: SideOffsets,
    pub theme: usize,
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
    pub fn draw(
        &self,
        off_x: i32,
        off_y: i32,
        textures: &mut HashMap<String, Texture2D>,
        theme: &Theme,
        gs: &GlobalState,
    ) {
        let max_y = self.tiles[0].len() - 1;
        let max_x = self.tiles[0][0].len() - 1;
        for layer in self.tiles.iter() {
            for (y, row) in layer.iter().enumerate() {
                for (x, tile) in row.iter().enumerate() {
                    let adj = Adjacencies {
                        up: y == 0 || layer[y - 1][x] == *tile,
                        down: y == max_y || layer[y + 1][x] == *tile,
                        left: x == 0 || layer[y][x - 1] == *tile,
                        right: x == max_x || layer[y][x + 1] == *tile,
                    };
                    tile.draw(
                        x as i32 * TILE_PIXELS + off_x,
                        y as i32 * TILE_PIXELS + off_y,
                        textures,
                        theme,
                        &adj,
                    )
                }
            }
        }
        for o in self.objects.iter() {
            o.draw(off_x, off_y, textures, gs)
        }
    }
    pub fn update(
        &mut self,
        keys_pressed: &mut HashMap<KeyCode, bool>,
        global_state: &mut GlobalState,
    ) {
        global_state.timer += 1;
        for o in self.objects.iter_mut() {
            o.update(keys_pressed, &self.tiles, global_state)
        }
        self.objects.retain(|o| !o.should_clear());
        let mut extra_objs = self
            .objects
            .iter()
            .filter_map(|o| o.spawn(&global_state))
            .collect();

        self.objects.append(&mut extra_objs);
    }

    pub fn from_level_raw(
        l: LevelRaw,
        my_ind: usize,
        subs: &HashMap<(usize, usize, usize, usize), Tile>,
    ) -> Self {
        let mut tiles = vec![];
        let mut objects: Vec<Box<dyn Object>> = vec![];
        let mut side_offsets = SideOffsets {
            left: None,
            right: None,
            up: None,
            down: None,
        };
        let mut door_exits = l.door_exits.iter();

        for (la, layer) in l.tiles.iter().enumerate() {
            let mut l_tiles = vec![];

            for (y, row) in layer.iter().enumerate() {
                let mut row_tiles = vec![];
                for (x, tile) in row.iter().enumerate() {
                    let newt = subs.get(&(my_ind, la, y, x)).unwrap_or(tile);
                    match newt {
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
                        Tile::SawLauncherLeft
                        | Tile::SawLauncherRight
                        | Tile::SawLauncherUp
                        | Tile::SawLauncherDown => {
                            let (vx, vy) = match &newt {
                                Tile::SawLauncherLeft => (-3 * TILE_SIZE / 16, 0),
                                Tile::SawLauncherRight => (3 * TILE_SIZE / 16, 0),
                                Tile::SawLauncherUp => (0, -3 * TILE_SIZE / 16),
                                Tile::SawLauncherDown => (0, 3 * TILE_SIZE / 16),
                                _ => unreachable!(),
                            };

                            objects.push(Box::new(SawLauncher {
                                x: x as i32 * TILE_SIZE,
                                y: y as i32 * TILE_SIZE,
                                vx,
                                vy,
                                frames: 45,
                            }));

                            row_tiles.push(*newt)
                        }
                        Tile::SlowSawLauncherLeft
                        | Tile::SlowSawLauncherRight
                        | Tile::SlowSawLauncherUp
                        | Tile::SlowSawLauncherDown => {
                            let (vx, vy) = match &newt {
                                Tile::SlowSawLauncherLeft => (-TILE_SIZE / 32, 0),
                                Tile::SlowSawLauncherRight => (TILE_SIZE / 32, 0),
                                Tile::SlowSawLauncherUp => (0, -TILE_SIZE / 32),
                                Tile::SlowSawLauncherDown => (0, TILE_SIZE / 32),
                                _ => unreachable!(),
                            };

                            objects.push(Box::new(SawLauncher {
                                x: x as i32 * TILE_SIZE,
                                y: y as i32 * TILE_SIZE,
                                vx,
                                vy,
                                frames: 32,
                            }));

                            row_tiles.push(*newt)
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
            theme: l.theme,
        }
    }
}

fn load_level(path: &str, level_inds: &HashMap<&str, usize>) -> LevelRaw {
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

    let (exits, door_exits, theme) = {
        let mut exits = SideExits {
            left: None,
            right: None,
            up: None,
            down: None,
        };
        let mut door_exits = vec![];
        let mut theme = 0;

        for l in parts.next().expect("should have part").lines() {
            let mut halves = l.split(":");
            let left_half = halves.next().expect("should have two halves").trim();

            let right_half = halves.next().expect("should have two halves").trim();
            let right_half: usize = match right_half.parse() {
                Ok(i) => i,
                Err(_) => *level_inds.get(right_half).expect("should exist"),
            };

            match left_half {
                "left" => exits.left = Some(right_half),
                "right" => exits.right = Some(right_half),
                "up" => exits.up = Some(right_half),
                "down" => exits.down = Some(right_half),
                "door" => door_exits.push(right_half),
                "theme" => theme = right_half,
                _ => (),
            }
        }

        (exits, door_exits, theme)
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
        theme,
    }
}

pub struct Levelset {
    pub name: String,
    pub levels: Vec<LevelRaw>,
    pub themes: Vec<Theme>,
    pub secret_count: i32,
}

pub fn load_levelset(path: &str) -> Levelset {
    let levelset_file = std::fs::read_to_string(format!("{}/levels.levelset", path)).unwrap();
    let levelset_file = levelset_file.trim().replace("\r\n", "\n");

    // println!("{}", levelset_file);

    let mut parts = levelset_file.split("\n===\n");

    let name = parts.next().expect("should have part").to_string();

    let mut secret_count = 0;

    let level_names: Vec<&str> = parts
        .next()
        .expect("should have part")
        .lines()
        .map(|n| n.trim())
        .collect();

    let level_inds: HashMap<&str, usize> = level_names
        .iter()
        .enumerate()
        .map(|(i, n)| (*n, i))
        .collect();

    let mut levels = vec![];
    for l in level_names {
        // println!("reading {}/{}.lvl", path, l);

        let lev = load_level(&format!("{}/{}.lvl", path, l), &level_inds);

        secret_count += lev.secret_count();

        levels.push(lev);
    }

    let mut themes = vec![];
    let np = parts.next();
    if np.is_some() {
        for l in np.expect("is some").lines() {
            let l = l.trim();

            // println!("reading {}/{}.nmltheme", path, l);

            themes.push(Theme::from_path(&format!("{}/{}.nmltheme", path, l)));
        }
    }

    Levelset {
        name,
        levels,
        themes,
        secret_count,
    }
}
