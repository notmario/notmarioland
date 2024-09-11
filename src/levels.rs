use super::{MAX_PLAYER_SPEED, PIXEL_SIZE, PLAYER_ACCEL, TILE_PIXELS, TILE_SIZE};
use crate::{
    texture_cache, Adjacencies, Theme, TransitionAnimationType, SCREEN_HEIGHT, SCREEN_WIDTH,
};
use macroquad::prelude::*;
use std::collections::{HashMap, VecDeque};

fn draw_rect_i32(x: i32, y: i32, w: i32, h: i32, c: Color) {
    draw_rectangle(x as f32, y as f32, w as f32, h as f32, c)
}

pub struct GlobalState {
    pub changed_tiles: HashMap<(usize, usize, usize, usize), Tile>,
    pub keys: [i32; 6],
    pub timer: i32,
    pub secrets: i32,
    pub jumps: i32,
    pub collected_jump_arrows: VecDeque<(usize, usize, usize)>,
    pub binocularing: bool,
    pub binocular_t: i32,
    pub binocular_rx: i32,
    pub binocular_ry: i32,
    pub modifiers: Modifiers,
    pub default_modifiers: Modifiers,
}

#[derive(Copy, Clone)]
pub struct Modifiers {
    pub superslippery: bool,
    pub game_speed: f32,
    pub invisiblelevel: bool,
    pub invisibleplayer: bool,
    pub nowalljump: bool,
    pub alwaysjumping: bool,
    pub uncapped_speed: bool,
}

impl Default for Modifiers {
    fn default() -> Self {
        Modifiers {
            superslippery: false,
            game_speed: 1.,
            invisiblelevel: false,
            invisibleplayer: false,
            nowalljump: false,
            alwaysjumping: false,
            uncapped_speed: false,
        }
    }
}

impl GlobalState {
    pub fn new() -> Self {
        GlobalState {
            changed_tiles: HashMap::new(),
            keys: [0; 6],
            timer: 0,
            secrets: 0,
            jumps: 0,
            collected_jump_arrows: VecDeque::new(),
            binocularing: false,
            binocular_t: 0,
            binocular_rx: 0,
            binocular_ry: 0,
            modifiers: Default::default(),
            default_modifiers: Default::default(),
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
    SecretDoorGeneric,
    SecretDoor(usize),

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

    Binocular,

    IceCube,
    PlayerVanish,
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
            "secretdoor" => Self::SecretDoorGeneric,
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

            "binocular" => Self::Binocular,

            "icecube" => Self::IceCube,
            "playervanish" => Self::PlayerVanish,

            _ => Self::Empty,
        }
    }
    // pull this out into its own function because yes
    fn sprite(&self) -> Option<&'static str> {
        match self {
            Self::Door(_) | Self::DoorGeneric => Some("assets/door.png"),
            Self::SecretDoor(_) | Self::SecretDoorGeneric => Some("assets/secretdoor.png"),
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

            Self::Binocular => Some("assets/binocular.png"),

            Self::IceCube => Some("assets/icecube.png"),
            Self::PlayerVanish => Some("assets/playervanish.png"),

            _ => None,
        }
    }

    fn minimap_col(&self) -> Color {
        match self {
            Self::Wall | Self::Wall2 | Self::Wall3 => WHITE,
            Self::Wall4 => BLUE,
            Self::Spikes => color_u8!(255, 104, 104, 255),
            _ => color_u8!(0, 0, 0, 0),
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
            Self::OneWayLeft | Self::OneWayRight | Self::OneWayUp | Self::OneWayDown => {
                let te = &theme.oneway;
                if te.is_some() {
                    let off = match self {
                        Self::OneWayLeft => (0, 16),
                        Self::OneWayRight => (16, 16),
                        Self::OneWayUp => (0, 0),
                        Self::OneWayDown => (16, 0),
                        _ => unreachable!(),
                    };
                    let t = texture_cache!(textures, te.as_ref().expect("it exists"));
                    draw_texture_ex(
                        &t,
                        x as f32,
                        y as f32,
                        WHITE,
                        DrawTextureParams {
                            source: Some(Rect {
                                x: off.0 as f32,
                                y: off.1 as f32,
                                w: 16.,
                                h: 16.,
                            }),
                            ..Default::default()
                        },
                    )
                } else {
                    match self {
                        Self::OneWayLeft => {
                            draw_rect_i32(x, y, TILE_PIXELS / 8, TILE_PIXELS, BLACK)
                        }
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
                        _ => unreachable!(),
                    }
                }
            }
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
    fn shift_by(&self, off: (i32, i32)) -> Self {
        AABB {
            x: self.x + off.0,
            y: self.y + off.1,
            w: self.w,
            h: self.h,
        }
    }
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
                        gs.collected_jump_arrows.push_back((li, ty, tx));
                    }
                    Tile::IceCube => {
                        gs.modifiers.superslippery = true;
                        l[ty][tx] = Tile::Empty;
                    }
                    Tile::PlayerVanish => {
                        gs.modifiers.invisibleplayer = true;
                        l[ty][tx] = Tile::Empty;
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
                if let Tile::SecretDoor(_) = l[ty][tx] {
                    return Some(l[ty][tx]);
                }
                if l[ty][tx] == Tile::Binocular {
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
                if let Tile::SecretDoor(i) = tile {
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
        _tiles: &mut Vec<Vec<Vec<Tile>>>,
        _global_state: &mut GlobalState,
    ) {
    }

    fn draw(
        &self,
        _off_x: i32,
        _off_y: i32,
        _texture: &mut HashMap<String, Texture2D>,
        _gs: &GlobalState,
        _t: &TransitionAnimationType,
    ) {
    }

    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    fn should_clear(&self) -> bool {
        false
    }
    fn spawn(&self, _gs: &GlobalState) -> Option<Box<dyn Object>> {
        None
    }
}

// used for death animation
fn trianglerad(theta: f32) -> f32 {
    let theta = theta % std::f32::consts::TAU;
    let pithree = std::f32::consts::FRAC_PI_3;
    let new_theta = if theta < 2. * pithree {
        theta - pithree
    } else if theta < 4. * pithree {
        theta + 3. * pithree
    } else {
        theta + pithree
    };

    let r = 0.5 * new_theta.cos().recip();
    if r.is_infinite() {
        1.
    } else {
        r
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
        tiles: &mut Vec<Vec<Vec<Tile>>>,
        global_state: &mut GlobalState,
    ) {
        if global_state.binocularing {
            return;
        }
        // accelerate left and right
        self.freeze_timer -= 1;
        let unslippy = check_tilemap_wallslideable(self.get_aabb().shift_by((0, 4)), &tiles);
        if self.freeze_timer <= 0 {
            if is_key_down(KeyCode::Left) && !is_key_down(KeyCode::Right) {
                if self.wall_sliding > 0 {
                    self.wall_sliding = 0
                }
                if self.vx >= -MAX_PLAYER_SPEED {
                    if global_state.modifiers.superslippery {
                        self.vx = (-MAX_PLAYER_SPEED).max(self.vx - PLAYER_ACCEL / 8);
                    } else if unslippy {
                        self.vx = (-MAX_PLAYER_SPEED).max(self.vx - PLAYER_ACCEL);
                    } else {
                        self.vx = (-MAX_PLAYER_SPEED).max(self.vx - PLAYER_ACCEL / 2);
                    }
                } else {
                    self.vx += TILE_SIZE / 128;
                }
            } else if is_key_down(KeyCode::Right) && !is_key_down(KeyCode::Left) {
                if self.wall_sliding > 0 {
                    self.wall_sliding = 0
                }
                if self.vx <= MAX_PLAYER_SPEED {
                    if global_state.modifiers.superslippery {
                        self.vx = (MAX_PLAYER_SPEED).min(self.vx + PLAYER_ACCEL / 8);
                    } else if unslippy {
                        self.vx = (MAX_PLAYER_SPEED).min(self.vx + PLAYER_ACCEL);
                    } else {
                        self.vx = (MAX_PLAYER_SPEED).min(self.vx + PLAYER_ACCEL / 2);
                    }
                } else {
                    self.vx -= TILE_SIZE / 128;
                }
            }
        }

        if !is_key_down(KeyCode::Left) && !is_key_down(KeyCode::Right) && self.freeze_timer <= 0 {
            if global_state.modifiers.superslippery {
            } else if !unslippy {
                self.vx *= 15;
                self.vx /= 16;
            } else {
                self.vx *= 11;
                self.vx /= 16;
            }
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
        if !global_state.modifiers.uncapped_speed {
            self.vx = self.vx.clamp(-TILE_SIZE, TILE_SIZE);
            self.vy = self.vy.clamp(-TILE_SIZE, TILE_SIZE);
        }
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

        let mod_movement = remaining_movement % TILE_SIZE;
        let mut temp_movement = remaining_movement;
        if mod_movement != temp_movement {
            loop {
                temp_movement -= TILE_SIZE * self.vx.signum();
                self.x += TILE_SIZE * self.vx.signum();
                if self.vx == 0 || temp_movement == mod_movement {
                    break;
                }

                if check_tilemap_collision(
                    before_aabb,
                    self.get_aabb(),
                    tiles,
                    Direction::h_vel(self.vx),
                    &global_state,
                ) {
                    self.x -= TILE_SIZE * self.vx.signum();
                    // continue;
                    break;
                }
            }
        } // now we are aligned at tile boundary, do remaining movement,
          // then step back if we are then colliding
        self.x += temp_movement;
        if check_tilemap_collision(
            before_aabb,
            self.get_aabb(),
            tiles,
            Direction::h_vel(self.vx),
            &global_state,
        ) {
            let can_wallslide = check_tilemap_wallslideable(self.get_aabb(), tiles)
                && !global_state.modifiers.nowalljump;
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
                global_state.collected_jump_arrows.pop_front();
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
        tt: &TransitionAnimationType,
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

        if let TransitionAnimationType::Door(_) = tt {
            draw_offset = (0, 96)
        }
        if gs.binocularing {
            draw_offset = (0, 96)
        }
        if let TransitionAnimationType::Death(frames) = tt {
            let frames = 80 - frames;

            if frames < 30 {
                let k = 45 - frames;
                draw_texture_ex(
                    &t,
                    (self.x / PIXEL_SIZE + off_x + rand::gen_range(-k, k) / 15) as f32,
                    (self.y / PIXEL_SIZE + off_y + rand::gen_range(-k, k) / 15) as f32,
                    WHITE,
                    DrawTextureParams {
                        source: Some(Rect {
                            x: 16.,
                            y: 96.,
                            w: 16.,
                            h: 16.,
                        }),
                        ..Default::default()
                    },
                );
            } else {
                let center_x = self.x / PIXEL_SIZE + off_x;
                let center_y = self.y / PIXEL_SIZE + off_y;

                let r = 40. - 40. / (frames - 29) as f32;
                let theta = 0.2 * ((frames - 30) as f32).sqrt();

                let k = 1. - ((frames as f32 - 45.) / 25.).clamp(0., 1.);

                let c = Color {
                    r: 1.,
                    g: 1. - 0.5960784314 * k,
                    b: 1. - 0.5960784314 * k,
                    a: 1.,
                };

                let t = texture_cache!(textures, "assets/deaththingy.png");

                for i in [
                    0., 0.125, 0.333, 0.5625, 0.666, 0.8, 1., 1.2, 1.333, 1.4375, 1.666, 1.875,
                ] {
                    let my_theta = i as f32 * std::f32::consts::PI;
                    let my_r = r * trianglerad(my_theta);
                    let (mut x, mut y) = (my_theta - theta).sin_cos();
                    x *= my_r;
                    y *= my_r;

                    draw_texture(
                        &t,
                        (center_x + x as i32 + 4) as f32,
                        (center_y + y as i32 + 4) as f32,
                        c,
                    );
                }
            }
        } else {
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

            let mut rows = [("assets/arrowtiny.png", gs.jumps)]
                .into_iter()
                .filter(|k| k.1 != 0)
                .map(|(img, count)| std::iter::once(img).cycle().take(count as usize))
                .flatten();
            let mut count = rows.clone().count() as i32;
            let mut shells = vec![];
            let mut step = 16;
            while count > 0 {
                shells.push(count.min(step));
                count -= step;
                step += 6;
            }

            for (i, s) in shells.iter().enumerate() {
                let angper = std::f32::consts::TAU / *s as f32;
                let rad = 16. + 6. * i as f32;
                for off in 0..*s {
                    let r = rows.next().expect(
                        "number of elements should match the number of items allotted in shells",
                    );
                    let t = texture_cache!(textures, r);
                    let a = off as f32 * angper - gs.timer as f32 * 0.004 * (1. + i as f32 / 2.);
                    let (xoff, yoff) = a.sin_cos();
                    draw_texture(
                        &t,
                        (self.x / PIXEL_SIZE + off_x + TILE_PIXELS / 4 + (xoff * rad) as i32)
                            as f32,
                        (self.y / PIXEL_SIZE + off_y + TILE_PIXELS / 4 + (yoff * rad) as i32)
                            as f32,
                        WHITE,
                    );
                }
            }
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
        tiles: &mut Vec<Vec<Vec<Tile>>>,
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
        _t: &TransitionAnimationType,
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
        _tiles: &mut Vec<Vec<Vec<Tile>>>,
        _global_state: &mut GlobalState,
    ) {
    }

    fn draw(
        &self,
        _off_x: i32,
        _off_y: i32,
        _textures: &mut HashMap<String, Texture2D>,
        _gs: &GlobalState,
        _t: &TransitionAnimationType,
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
pub struct ArrowRespawn {
    pub x: i32,
    pub y: i32,

    pub frames: i32,
    pub layer: usize,
    pub xi: usize,
    pub yi: usize,
}

impl Object for ArrowRespawn {
    fn get_type(&self) -> &'static str {
        "ARROWRESPAWN"
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
        tiles: &mut Vec<Vec<Vec<Tile>>>,
        gs: &mut GlobalState,
    ) {
        if tiles[self.layer][self.yi][self.xi] != Tile::JumpArrow
            && !gs
                .collected_jump_arrows
                .contains(&(self.layer, self.yi, self.xi))
        {
            self.frames += 1;
            if self.frames >= 180 {
                tiles[self.layer][self.yi][self.xi] = Tile::JumpArrow
            }
        } else {
            self.frames = 0;
        }
    }

    fn draw(
        &self,
        off_x: i32,
        off_y: i32,
        textures: &mut HashMap<String, Texture2D>,
        gs: &GlobalState,
        _t: &TransitionAnimationType,
    ) {
        let t = texture_cache!(textures, "assets/jumparrowfill.png");

        let k = if gs
            .collected_jump_arrows
            .contains(&(self.layer, self.yi, self.xi))
        {
            0
        } else {
            255
        };

        draw_texture_ex(
            &t,
            (self.x / PIXEL_SIZE + off_x) as f32,
            (self.y / PIXEL_SIZE + off_y) as f32,
            color_u8!(255, k, k, 63),
            DrawTextureParams {
                ..Default::default()
            },
        );

        let size = self.frames / 15;

        draw_texture_ex(
            &t,
            (self.x / PIXEL_SIZE + off_x) as f32,
            (self.y / PIXEL_SIZE + off_y + 14 - size) as f32,
            WHITE,
            DrawTextureParams {
                source: Some(Rect {
                    x: 0.,
                    y: 14. - size as f32,
                    w: 16.,
                    h: size as f32,
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
    pub tiles: Vec<Vec<Vec<Tile>>>,
    exits: SideExits,
    door_exits: Vec<usize>,
    theme: Option<usize>,
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
        if off_x > SCREEN_WIDTH
            || off_y > SCREEN_HEIGHT
            || off_x + (max_x + 1) as i32 * TILE_PIXELS < 0
            || off_y + (max_y + 1) as i32 * TILE_PIXELS < 0
        {
            return;
        }
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
        default_theme: usize,
    ) {
        if !skip_actually_drawing {
            self.draw(
                off_x,
                off_y,
                my_ind,
                subs,
                textures,
                &themes[self.theme.unwrap_or(default_theme)],
            )
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
                default_theme,
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
                default_theme,
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
                default_theme,
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
                default_theme,
            );
        };
    }

    pub fn minimap_draw(
        &self,
        off_x: i32,
        off_y: i32,
        levels: &Vec<LevelRaw>,
        seen: &mut Vec<usize>,
        my_ind: usize,
        subs: &HashMap<(usize, usize, usize, usize), Tile>,
    ) {
        let tile_size_thing = 2;
        let (ly, lx) = (self.tiles[0].len(), self.tiles[0][0].len());
        if seen.len() == 1 {
            draw_rectangle(
                off_x as f32 - 1.,
                off_y as f32 - 1.,
                (lx * 2 + 2) as f32,
                (ly * 2 + 2) as f32,
                color_u8!(255, 255, 255, 31),
            );
            draw_rectangle_lines(
                off_x as f32 - 1.,
                off_y as f32 - 1.,
                (lx * 2 + 2) as f32,
                (ly * 2 + 2) as f32,
                1.,
                WHITE,
            );
        }
        for l in self.tiles.iter() {
            for (y, row) in l.iter().enumerate() {
                for (x, tile) in row.iter().enumerate() {
                    draw_rectangle(
                        (off_x + x as i32 * tile_size_thing) as f32,
                        (off_y + y as i32 * tile_size_thing) as f32,
                        tile_size_thing as f32,
                        tile_size_thing as f32,
                        tile.minimap_col(),
                    )
                }
            }
        }
        if self.exits.left.is_some() && !seen.contains(&self.exits.left.expect("is some")) {
            let ind = self.exits.left.expect("is some");
            seen.push(ind);

            let offset = levels[ind].tiles[0][0].len() as i32 * tile_size_thing;
            let perp_offset = (self.side_offsets().left.expect("is some")
                - levels[ind]
                    .side_offsets()
                    .right
                    .expect("corresponding should have exit anchor"))
                / TILE_SIZE
                * tile_size_thing;

            levels[ind].minimap_draw(off_x - offset, off_y + perp_offset, levels, seen, ind, subs);
        };
        if self.exits.right.is_some() && !seen.contains(&self.exits.right.expect("is some")) {
            let ind = self.exits.right.expect("is some");
            seen.push(ind);

            let offset = self.tiles[0][0].len() as i32 * tile_size_thing;
            let perp_offset = (self.side_offsets().right.expect("is some")
                - levels[ind]
                    .side_offsets()
                    .left
                    .expect("corresponding should have exit anchor"))
                / TILE_SIZE
                * tile_size_thing;

            levels[ind].minimap_draw(off_x + offset, off_y + perp_offset, levels, seen, ind, subs);
        };
        if self.exits.up.is_some() && !seen.contains(&self.exits.up.expect("is some")) {
            let ind = self.exits.up.expect("is some");
            seen.push(ind);

            let offset = levels[ind].tiles[0].len() as i32 * tile_size_thing;
            let perp_offset = (self.side_offsets().up.expect("is some")
                - levels[ind]
                    .side_offsets()
                    .down
                    .expect("corresponding should have exit anchor"))
                / TILE_SIZE
                * tile_size_thing;

            levels[ind].minimap_draw(off_x + perp_offset, off_y - offset, levels, seen, ind, subs);
        };
        if self.exits.down.is_some() && !seen.contains(&self.exits.down.expect("is some")) {
            let ind = self.exits.down.expect("is some");
            seen.push(ind);

            let offset = self.tiles[0].len() as i32 * tile_size_thing;
            let perp_offset = (self.side_offsets().down.expect("is some")
                - levels[ind]
                    .side_offsets()
                    .up
                    .expect("corresponding should have exit anchor"))
                / TILE_SIZE
                * tile_size_thing;

            levels[ind].minimap_draw(off_x + perp_offset, off_y + offset, levels, seen, ind, subs);
        };
    }

    pub fn find_theme(
        &self,
        levels: &Vec<LevelRaw>,
        seen: &mut Vec<usize>,
        my_ind: usize,
    ) -> Option<(usize, (i32, i32))> {
        if self.theme.is_some() {
            return Some((self.theme.expect("is some"), (0, 0)));
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

            let t = levels[ind].find_theme(levels, seen, ind);

            if let Some((t, (x, y))) = t {
                return Some((t, (x - offset, y + perp_offset)));
            }
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

            let t = levels[ind].find_theme(levels, seen, ind);

            if let Some((t, (x, y))) = t {
                return Some((t, (x + offset, y + perp_offset)));
            }
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

            let t = levels[ind].find_theme(levels, seen, ind);

            if let Some((t, (x, y))) = t {
                return Some((t, (x + perp_offset, y - offset)));
            }
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

            let t = levels[ind].find_theme(levels, seen, ind);

            if let Some((t, (x, y))) = t {
                return Some((t, (x + perp_offset, y + offset)));
            }
        };
        None
    }
}

pub struct Level {
    pub name: String,
    pub tiles: Vec<Vec<Vec<Tile>>>,
    pub objects: Vec<Box<dyn Object>>,
    pub side_exits: SideExits,
    pub side_offsets: SideOffsets,
    pub theme: usize,
    pub theme_offset: (i32, i32),
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
        t: &TransitionAnimationType,
    ) {
        if !gs.modifiers.invisiblelevel {
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
        }
        for o in self.objects.iter() {
            let should_draw = if o.get_type() == "PLAYER" {
                !gs.modifiers.invisibleplayer
            } else {
                !gs.modifiers.invisiblelevel
            };
            if should_draw {
                o.draw(off_x, off_y, textures, gs, t)
            }
        }
    }
    pub fn update(
        &mut self,
        keys_pressed: &mut HashMap<KeyCode, bool>,
        global_state: &mut GlobalState,
    ) {
        global_state.timer += 1;
        for o in self.objects.iter_mut() {
            o.update(keys_pressed, &mut self.tiles, global_state)
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
        other_levels: &Vec<LevelRaw>,
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
                        Tile::SecretDoorGeneric => {
                            let ind = door_exits
                                .next()
                                .expect("should have a corresponding door entrance");
                            row_tiles.push(Tile::SecretDoor(*ind));
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
                        Tile::JumpArrow => {
                            row_tiles.push(Tile::JumpArrow);
                            objects.push(Box::new(ArrowRespawn {
                                x: x as i32 * TILE_SIZE,
                                y: y as i32 * TILE_SIZE,
                                frames: 0,
                                layer: la,
                                xi: x,
                                yi: y,
                            }));
                        }

                        t => row_tiles.push(*t),
                    }
                }
                l_tiles.push(row_tiles);
            }

            tiles.push(l_tiles);
        }

        let theme = match l.theme {
            Some(t) => (t, (0, 0)),
            None => l
                .find_theme(other_levels, &mut vec![my_ind], my_ind)
                .unwrap_or((0, (0, 0))),
        };

        Level {
            name: l.name,
            tiles,
            objects,
            side_exits: l.exits,
            side_offsets,
            theme: theme.0,
            theme_offset: theme.1,
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
        let mut theme = None;

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
                "theme" => theme = Some(right_half),
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

            if l == "null" {
                themes.push(Theme::default())
            } else {
                themes.push(Theme::from_path(&format!("{}/{}.nmltheme", path, l)));
            }
        }
    }

    Levelset {
        name,
        levels,
        themes,
        secret_count,
    }
}
