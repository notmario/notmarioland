#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use core::f32;
use std::collections::HashMap;

use macroquad::prelude::*;

const PIXEL_SIZE: i32 = 256;
const TILE_PIXELS: i32 = 16;
const TILE_SIZE: i32 = TILE_PIXELS * PIXEL_SIZE;

const MAX_PLAYER_SPEED: i32 = TILE_SIZE * 3 / 16;
const PLAYER_ACCEL: i32 = TILE_SIZE / 16;

const SCREEN_WIDTH: i32 = 640;
const SCREEN_HEIGHT: i32 = 368;

mod levels;
use levels::Object;

mod macros;

enum MenuState {
    Main(usize),
    LevelsetSelect(usize),
    EditSelect(usize),
}

enum State {
    Menu(MenuState),
    Game {
        levelset: Option<levels::Levelset>,
        current_ind: usize,
        level: levels::Level,
        global_state: levels::GlobalState,
        won: bool,
    },
    Edit {
        levelset: String,
        level: Option<String>,
    },
}

#[derive(Clone)]
struct BackgroundLayer {
    image: String,

    // all are stored with TILE_SIZE as 1 tile
    off_x: i32,
    off_y: i32,
    para_factor_x: i32,
    para_factor_y: i32,

    scroll_x: i32,
    scroll_y: i32,
    mod_x: i32,
    mod_y: i32,
}

#[derive(Default, Clone)]
struct Theme {
    bg: Vec<BackgroundLayer>,

    wall_1: Option<String>,
    wall_2: Option<String>,
    wall_3: Option<String>,
    wall_4: Option<String>,

    back_wall_1: Option<String>,
    back_wall_2: Option<String>,
    back_wall_3: Option<String>,
    back_wall_4: Option<String>,
}

impl Theme {
    fn from_path(path: &str) -> Self {
        let s = std::fs::read_to_string(path).unwrap();
        let s = s.trim().replace("\r\n", "\n");

        let mut theme = Theme {
            ..Default::default()
        };

        for part in s.split("\n===\n") {
            if part.starts_with("bglayer") {
                let mut lines = part.lines();
                // println!("{:?}", lines);
                lines.next();
                theme.bg.push(BackgroundLayer {
                    image: lines.next().expect("should exist").into(),

                    off_x: lines.next().expect("sh").parse().expect("sh"),
                    off_y: lines.next().expect("sh").parse().expect("sh"),

                    para_factor_x: lines.next().expect("sh").parse().expect("sh"),
                    para_factor_y: lines.next().expect("sh").parse().expect("sh"),

                    scroll_x: lines.next().expect("sh").parse().expect("sh"),
                    scroll_y: lines.next().expect("sh").parse().expect("sh"),
                    mod_x: lines.next().expect("sh").parse().expect("sh"),
                    mod_y: lines.next().expect("sh").parse().expect("sh"),
                })
            } else {
                // tilesets
                for line in part.lines() {
                    let mut parts = line.split(": ");
                    let (a, b) = (
                        parts.next().expect("should exist"),
                        parts.next().expect("should exist"),
                    );
                    match a.trim() {
                        "wall_1" => theme.wall_1 = Some(b.trim().into()),
                        "wall_2" => theme.wall_2 = Some(b.trim().into()),
                        "wall_3" => theme.wall_3 = Some(b.trim().into()),
                        "wall_4" => theme.wall_4 = Some(b.trim().into()),

                        "back_wall_1" => theme.back_wall_1 = Some(b.trim().into()),
                        "back_wall_2" => theme.back_wall_2 = Some(b.trim().into()),
                        "back_wall_3" => theme.back_wall_3 = Some(b.trim().into()),
                        "back_wall_4" => theme.back_wall_4 = Some(b.trim().into()),

                        _ => (),
                    }
                }
            }
        }

        theme
    }

    async fn load_textures(&self, textures: &mut HashMap<String, Texture2D>) {
        for t in self.bg.iter() {
            texture!(textures, &t.image);
        }
        if self.wall_1.is_some() {
            texture!(textures, self.wall_1.as_ref().expect("is some"));
        }
        if self.wall_2.is_some() {
            texture!(textures, self.wall_1.as_ref().expect("is some"));
        }
        if self.wall_3.is_some() {
            texture!(textures, self.wall_1.as_ref().expect("is some"));
        }
        if self.wall_4.is_some() {
            texture!(textures, self.wall_1.as_ref().expect("is some"));
        }

        if self.back_wall_1.is_some() {
            texture!(textures, self.back_wall_1.as_ref().expect("is some"));
        }
        if self.back_wall_2.is_some() {
            texture!(textures, self.back_wall_1.as_ref().expect("is some"));
        }
        if self.back_wall_3.is_some() {
            texture!(textures, self.back_wall_1.as_ref().expect("is some"));
        }
        if self.back_wall_4.is_some() {
            texture!(textures, self.back_wall_1.as_ref().expect("is some"));
        }
    }
}

struct Adjacencies {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}

fn draw_inverted_circle(x: f32, y: f32, r: f32, c: Color) {
    let k = 0.5;
    let j = 0.75_f32.sqrt();

    draw_rectangle(
        x - r * j - SCREEN_WIDTH as f32 * 2.,
        y - SCREEN_HEIGHT as f32 * 2.,
        SCREEN_WIDTH as f32 * 2.,
        SCREEN_HEIGHT as f32 * 4.,
        c,
    );
    draw_rectangle(
        x + r * j,
        y - SCREEN_HEIGHT as f32 * 2.,
        SCREEN_WIDTH as f32 * 2.,
        SCREEN_HEIGHT as f32 * 4.,
        c,
    );
    draw_rectangle(
        x - r * j,
        y - r - SCREEN_HEIGHT as f32 * 2.,
        r * 2. * j,
        SCREEN_HEIGHT as f32 * 2.,
        c,
    );

    draw_rectangle(x - r * j, y + r * k, r * 2., SCREEN_HEIGHT as f32 * 2., c);

    draw_triangle(
        Vec2 {
            x: x - r * j,
            y: y - r,
        },
        Vec2 { x, y: y - r },
        Vec2 {
            x: x - r * j,
            y: y + r * k,
        },
        c,
    );

    draw_triangle(
        Vec2 {
            x: x + r * j,
            y: y - r,
        },
        Vec2 { x, y: y - r },
        Vec2 {
            x: x + r * j,
            y: y + r * k,
        },
        c,
    );
}

fn draw_number_text(t: &Texture2D, text: &str, x: f32, y: f32, c: Color, timer: i32) {
    let mut stash_off = 0;
    let divs = [2, 3, 5, 7, 11, 13, 17, 19];
    for (i, (ch, div)) in text.chars().zip(divs.iter()).enumerate() {
        let off_x = match ch {
            '0' => 0,
            '1' => 1,
            '2' => 2,
            '3' => 3,
            '4' => 4,
            '5' => 5,
            '6' => 6,
            '7' => 7,
            '8' => 8,
            '9' => 9,
            '/' => 10,
            ':' => 11,
            _ => 11,
        } * 24;
        let off_y = (timer / div) % 4 * 32;
        if ch == ':' {
            stash_off -= 6;
        }

        draw_texture_ex(
            &t,
            x + i as f32 * 20. + stash_off as f32,
            y,
            c,
            DrawTextureParams {
                source: Some(Rect {
                    x: off_x as f32,
                    y: off_y as f32,
                    w: 24.,
                    h: 32.,
                }),
                ..Default::default()
            },
        );
        if ch == ':' {
            stash_off -= 6;
        }
    }
}

enum TransitionAnimationType {
    None,
    Death(i32),
    Door(i32),
}

fn window_conf() -> Conf {
    Conf {
        window_title: "notmarioland".to_owned(),
        fullscreen: false,
        window_width: SCREEN_WIDTH * 2,
        window_height: SCREEN_HEIGHT * 2,
        window_resizable: false,

        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut textures: HashMap<String, Texture2D> = HashMap::new();

    let levelsets: Vec<String> = std::fs::read_dir("levels/")
        .expect("directory should exist")
        .filter_map(|f| {
            if !f.is_ok() {
                return None;
            }
            let a = f.as_ref().expect("is ok").path();
            let a = a
                .to_str()
                .unwrap()
                .strip_prefix("levels/")
                .expect("we know it starts with levels/");

            let t_path = format!(
                "{}/levels.levelset",
                f.expect("is ok")
                    .path()
                    .to_str()
                    .expect("path should be string")
            );
            if std::fs::read(t_path.clone()).is_err() {
                // println!("{}", t_path);
                return None;
            }
            // println!("found levelset {:?}", a);
            Some(a.to_string())
        })
        .collect();

    let preload_textures = [
        "assets/player.png",
        "assets/redkey.png",
        "assets/yellowkey.png",
        "assets/greenkey.png",
        "assets/cyankey.png",
        "assets/bluekey.png",
        "assets/magentakey.png",
        "assets/redlock.png",
        "assets/yellowlock.png",
        "assets/greenlock.png",
        "assets/cyanlock.png",
        "assets/bluelock.png",
        "assets/magentalock.png",
        "assets/saw.png",
        "assets/sawlauncherleft.png",
        "assets/sawlauncherright.png",
        "assets/sawlauncherup.png",
        "assets/sawlauncherdown.png",
        "assets/slowsawlauncherleft.png",
        "assets/slowsawlauncherright.png",
        "assets/slowsawlauncherup.png",
        "assets/slowsawlauncherdown.png",
        "assets/secret.png",
        "assets/goal.png",
        "assets/door.png",
        "assets/secretdoor.png",
        "assets/spike.png",
        "assets/jumparrow.png",
        "assets/jumparrowoutline.png",
        "assets/jumparrowfill.png",
        "assets/arrowtiny.png",
        "assets/deaththingy.png",
        "assets/pausebottom.png",
        "assets/pauseleftbase.png",
        "assets/pauserightbase.png",
        "assets/pausetopbase.png",
        "assets/pauseresume.png",
        "assets/pauseresume-dull.png",
        "assets/pausereset.png",
        "assets/pausereset-dull.png",
        "assets/pauseexit.png",
        "assets/pauseexit-dull.png",
        "assets/winrightbase.png",
        "assets/numbers.png",
    ];

    for p in preload_textures {
        texture!(&mut textures, p);
    }

    let render_target = render_target(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32);
    render_target.texture.set_filter(FilterMode::Nearest);
    let mut cam = Camera2D::from_display_rect(Rect::new(
        0.,
        SCREEN_HEIGHT as f32,
        SCREEN_WIDTH as f32,
        -SCREEN_HEIGHT as f32,
    ));

    cam.render_target = Some(render_target.clone());
    let cam_true = Camera2D::from_display_rect(Rect::new(
        0.,
        SCREEN_HEIGHT as f32,
        SCREEN_WIDTH as f32,
        -SCREEN_HEIGHT as f32,
    ));

    let mut render_off_x = 0.;
    let mut render_off_y = 0.;

    let mut remaining_timer = 0.;
    let mut paused = false;
    let mut paused_frames = 0;
    let mut paused_selection = 0;

    let mut keys_pressed: HashMap<KeyCode, bool> = HashMap::new();

    let mut state = State::Menu(MenuState::Main(0));

    let mut themes = vec![];
    let mut deaths = 0;
    let mut secret_count = 0;
    let mut transition_ticks: i32 = 0;
    let mut next_ind: Option<usize> = None;
    let mut levelset_ind = 0;

    loop {
        clear_background(WHITE);

        set_default_camera();

        match &mut state {
            State::Menu(menu_state) => {
                clear_background(BLACK);

                set_camera(&cam_true);
                match menu_state {
                    MenuState::Main(ind) => {
                        draw_text("main menu (temporary)", 4., 12., 16., WHITE);

                        for (i, o) in ["play", "quit"].iter().enumerate() {
                            draw_text(
                                &format!("{}{}", if *ind == i { "> " } else { "  " }, o),
                                4.,
                                28. + 16. * i as f32,
                                16.,
                                WHITE,
                            );
                        }

                        if is_key_pressed(KeyCode::Down) && *ind < 2 {
                            *ind += 1
                        }
                        if is_key_pressed(KeyCode::Up) && *ind > 0 {
                            *ind -= 1
                        }

                        if is_key_pressed(KeyCode::Z) {
                            match ind {
                                0 => *menu_state = MenuState::LevelsetSelect(0),
                                // 1 => *menu_state = MenuState::EditSelect(0),
                                1 => panic!("user closed game"),
                                _ => (),
                            }
                        }
                    }
                    MenuState::LevelsetSelect(ind) => {
                        draw_text("select levelset", 4., 12., 16., WHITE);

                        for (i, l) in levelsets.iter().chain(["back".into()].iter()).enumerate() {
                            draw_text(
                                &format!("{}{}", if *ind == i { "> " } else { "  " }, l),
                                4.,
                                28. + 16. * i as f32,
                                16.,
                                WHITE,
                            );
                        }

                        if is_key_pressed(KeyCode::Down) && *ind < levelsets.len() {
                            *ind += 1
                        }
                        if is_key_pressed(KeyCode::Up) && *ind > 0 {
                            *ind -= 1
                        }

                        if is_key_pressed(KeyCode::Z) {
                            if *ind == levelsets.len() {
                                *menu_state = MenuState::Main(0);
                            } else {
                                let levelset =
                                    levels::load_levelset(&format!("levels/{}", levelsets[*ind]));
                                levelset_ind = *ind;
                                let current_ind = 0; // we assume the first level is index 0

                                let level_raw = levelset.levels[current_ind].clone();
                                let level = levels::Level::from_level_raw(
                                    level_raw,
                                    0,
                                    &levelset.levels,
                                    &HashMap::new(),
                                );

                                paused = false;
                                render_off_x = 0.;
                                render_off_y = 0.;

                                themes = levelset.themes.clone();
                                deaths = 0;
                                secret_count = levelset.secret_count;
                                transition_ticks = 0;
                                paused_frames = 0;

                                if themes.len() == 0 {
                                    themes.push(Theme {
                                        ..Default::default()
                                    })
                                }

                                for t in themes.iter() {
                                    t.load_textures(&mut textures).await;
                                }

                                state = State::Game {
                                    levelset: Some(levelset),
                                    current_ind,
                                    level,
                                    global_state: levels::GlobalState::new(),
                                    won: false,
                                }
                            }
                        } else if is_key_pressed(KeyCode::Escape) {
                            *menu_state = MenuState::Main(0);
                        }
                    }
                    MenuState::EditSelect(ind) => {
                        draw_text("select levelset to edit", 4., 12., 16., WHITE);

                        for (i, l) in levelsets
                            .iter()
                            .chain(["new".into(), "back".into()].iter())
                            .enumerate()
                        {
                            draw_text(
                                &format!("{}{}", if *ind == i { "> " } else { "  " }, l),
                                4.,
                                28. + 16. * i as f32,
                                16.,
                                WHITE,
                            );
                        }

                        if is_key_pressed(KeyCode::Down) && *ind < levelsets.len() + 1 {
                            *ind += 1
                        }
                        if is_key_pressed(KeyCode::Up) && *ind > 0 {
                            *ind -= 1
                        }

                        if is_key_pressed(KeyCode::Z) {
                            if *ind == levelsets.len() {
                            } else if *ind == levelsets.len() + 1 {
                                *menu_state = MenuState::Main(0);
                            } else {
                            }
                        } else if is_key_pressed(KeyCode::Escape) {
                            *menu_state = MenuState::Main(0)
                        }
                    }
                }
            }
            State::Game {
                levelset,
                current_ind,
                level,
                global_state,
                won,
            } => {
                set_camera(&cam);
                if is_key_pressed(KeyCode::Escape) {
                    paused = !paused;
                    paused_selection = 0;
                }
                if !paused && !*won {
                    let delta = get_frame_time();
                    remaining_timer += delta;

                    for (keycode, is_pressed) in keys_pressed.iter_mut() {
                        if is_key_pressed(*keycode) {
                            *is_pressed = true
                        }
                    }
                    paused_frames = (paused_frames - 3).clamp(0, 80);
                    if remaining_timer * 60. >= 1. {
                        transition_ticks += 1;
                    }

                    if transition_ticks == -1 && remaining_timer * 60. >= 1. {
                        if let Some(index) = next_ind {
                            let level_raw =
                                levelset.as_ref().expect("is some").levels[index].clone();

                            let old_ind = *current_ind;
                            *current_ind = index;
                            *level = levels::Level::from_level_raw(
                                level_raw,
                                *current_ind,
                                &levelset.as_ref().unwrap().levels,
                                &global_state.changed_tiles,
                            );

                            let p_pos = levels::find_door(old_ind, &level.tiles);
                            println!("{:?}", p_pos);
                            if let Some((x, y)) = p_pos {
                                let p_obj = level.player_obj();

                                (p_obj.x, p_obj.y) = (x * TILE_SIZE, y * TILE_SIZE);
                            }

                            let new_d = level.dimensions();

                            render_off_x = (SCREEN_WIDTH / 2 - new_d.0 * TILE_PIXELS / 2) as f32;
                            render_off_y = (SCREEN_HEIGHT / 2 - new_d.1 * TILE_PIXELS / 2) as f32;

                            next_ind = None;
                        } else {
                            let level_raw =
                                levelset.as_ref().expect("is some").levels[*current_ind].clone();
                            *level = levels::Level::from_level_raw(
                                level_raw,
                                *current_ind,
                                &levelset.as_ref().unwrap().levels,
                                &global_state.changed_tiles,
                            );
                        }

                        global_state.jumps = 0;
                        global_state.collected_jump_arrows = std::collections::VecDeque::new();
                    }

                    if remaining_timer * 60. >= 1. && transition_ticks >= 0 {
                        level.update(&mut keys_pressed, global_state);

                        let pbb = level.player_obj().get_aabb();

                        levels::collect_keys(pbb, *current_ind, &mut level.tiles, global_state);
                        levels::collect_doors(pbb, *current_ind, &mut level.tiles, global_state);

                        // check if we should exit!!
                        let player_pos = level.focus_position();
                        let player_vel = level.player_vel();
                        let d = level.dimensions();
                        if levelset.is_some() && player_pos.0 < 0 && player_vel.0 < 0 {
                            if level.side_exits.left.is_some() {
                                let old_sliding = level.player_obj().wall_sliding;
                                let old_freeze = level.player_obj().freeze_timer;

                                let r_off_y =
                                    level.side_offsets.left.expect("should have an exit anchor");
                                let off_y = player_pos.1 - TILE_SIZE / 2 - r_off_y;
                                let new_x = player_pos.0 - TILE_SIZE / 2;

                                let level_raw = levelset.as_ref().expect("is some").levels
                                    [level.side_exits.left.expect("is some")]
                                .clone();

                                *current_ind = level.side_exits.left.expect("is some");
                                *level = levels::Level::from_level_raw(
                                    level_raw,
                                    *current_ind,
                                    &levelset.as_ref().unwrap().levels,
                                    &global_state.changed_tiles,
                                );
                                global_state.jumps = 0;
                                global_state.collected_jump_arrows =
                                    std::collections::VecDeque::new();
                                let new_off_y = level
                                    .side_offsets
                                    .right
                                    .expect("should have an exit anchor");
                                let new_off_x = level.dimensions().0 * TILE_SIZE;

                                render_off_x -= (level.dimensions().0 * TILE_PIXELS) as f32;
                                render_off_y -= ((new_off_y - r_off_y) / PIXEL_SIZE) as f32;

                                // paused = true;
                                let p = level.player_obj();

                                (p.freeze_timer, p.wall_sliding) = (old_freeze, old_sliding);
                                p.x = new_x + new_off_x;
                                p.y = new_off_y + off_y;
                                (p.vx, p.vy) = player_vel
                            } else {
                                let p = level.player_obj();
                                p.x = -TILE_SIZE / 2
                            }
                        } else if levelset.is_some()
                            && player_pos.0 > d.0 * TILE_SIZE
                            && player_vel.0 > 0
                        {
                            if level.side_exits.right.is_some() {
                                let old_sliding = level.player_obj().wall_sliding;
                                let old_freeze = level.player_obj().freeze_timer;

                                let r_off_y = level
                                    .side_offsets
                                    .right
                                    .expect("should have an exit anchor");
                                let off_y = player_pos.1 - TILE_SIZE / 2 - r_off_y;
                                let new_x = player_pos.0 - d.0 * TILE_SIZE - TILE_SIZE / 2;

                                let level_raw = levelset.as_ref().expect("is some").levels
                                    [level.side_exits.right.expect("is some")]
                                .clone();

                                *current_ind = level.side_exits.right.expect("is some");
                                *level = levels::Level::from_level_raw(
                                    level_raw,
                                    *current_ind,
                                    &levelset.as_ref().unwrap().levels,
                                    &global_state.changed_tiles,
                                );
                                global_state.jumps = 0;
                                global_state.collected_jump_arrows =
                                    std::collections::VecDeque::new();
                                let new_off_y =
                                    level.side_offsets.left.expect("should have an exit anchor");

                                render_off_x += (d.0 * TILE_PIXELS) as f32;
                                render_off_y -= ((new_off_y - r_off_y) / PIXEL_SIZE) as f32;

                                let p = level.player_obj();

                                (p.freeze_timer, p.wall_sliding) = (old_freeze, old_sliding);
                                p.x = new_x;
                                p.y = new_off_y + off_y;
                                (p.vx, p.vy) = player_vel;
                            } else {
                                let p = level.player_obj();
                                p.x = d.0 * TILE_SIZE - TILE_SIZE / 2;
                            }
                        } else if levelset.is_some() && player_pos.1 < 0 && player_vel.1 < 0 {
                            if level.side_exits.up.is_some() {
                                let old_sliding = level.player_obj().wall_sliding;
                                let old_freeze = level.player_obj().freeze_timer;

                                let r_off_x =
                                    level.side_offsets.up.expect("should have an exit anchor");
                                let off_x = player_pos.0 - TILE_SIZE / 2 - r_off_x;
                                let new_y = player_pos.1 - TILE_SIZE / 2;

                                let level_raw = levelset.as_ref().expect("is some").levels
                                    [level.side_exits.up.expect("is some")]
                                .clone();

                                *current_ind = level.side_exits.up.expect("is some");
                                *level = levels::Level::from_level_raw(
                                    level_raw,
                                    *current_ind,
                                    &levelset.as_ref().unwrap().levels,
                                    &global_state.changed_tiles,
                                );
                                global_state.jumps = 0;
                                global_state.collected_jump_arrows =
                                    std::collections::VecDeque::new();
                                let new_off_x =
                                    level.side_offsets.down.expect("should have an exit anchor");
                                let new_off_y = level.dimensions().1 * TILE_SIZE;

                                render_off_y -= (level.dimensions().1 * TILE_PIXELS) as f32;
                                render_off_x -= ((new_off_x - r_off_x) / PIXEL_SIZE) as f32;

                                // paused = true;
                                let p = level.player_obj();

                                (p.freeze_timer, p.wall_sliding) = (old_freeze, old_sliding);
                                p.y = new_y + new_off_y;
                                p.x = new_off_x + off_x;
                                (p.vx, p.vy) = player_vel
                            }
                        } else if levelset.is_some()
                            && player_pos.1 > d.1 * TILE_SIZE
                            && player_vel.1 > 0
                        {
                            if level.side_exits.down.is_some() {
                                let old_sliding = level.player_obj().wall_sliding;
                                let old_freeze = level.player_obj().freeze_timer;

                                let r_off_x =
                                    level.side_offsets.down.expect("should have an exit anchor");
                                let off_x = player_pos.0 - TILE_SIZE / 2 - r_off_x;
                                let new_y =
                                    player_pos.1 - TILE_SIZE / 2 - level.dimensions().1 * TILE_SIZE;

                                let level_raw = levelset.as_ref().expect("is some").levels
                                    [level.side_exits.down.expect("is some")]
                                .clone();

                                *current_ind = level.side_exits.down.expect("is some");
                                *level = levels::Level::from_level_raw(
                                    level_raw,
                                    *current_ind,
                                    &levelset.as_ref().unwrap().levels,
                                    &global_state.changed_tiles,
                                );
                                global_state.jumps = 0;
                                global_state.collected_jump_arrows =
                                    std::collections::VecDeque::new();
                                let new_off_x =
                                    level.side_offsets.up.expect("should have an exit anchor");

                                render_off_y += (d.1 * TILE_PIXELS) as f32;
                                render_off_x -= ((new_off_x - r_off_x) / PIXEL_SIZE) as f32;

                                // paused = true;
                                let p = level.player_obj();

                                (p.freeze_timer, p.wall_sliding) = (old_freeze, old_sliding);
                                p.y = new_y;
                                p.x = new_off_x + off_x;
                                (p.vx, p.vy) = player_vel
                            } else {
                                if levelset.is_some() {
                                    transition_ticks = -80;

                                    deaths += 1;
                                } else {
                                    todo!()
                                }
                            }
                        } else if levelset.is_some()
                            && *keys_pressed.entry(KeyCode::Up).or_insert(false)
                        {
                            let p_obj = level.player_obj();
                            let grounded = p_obj.grounded;
                            let aabb = (p_obj as &mut dyn Object).get_aabb();

                            let doors = levels::check_door(aabb, &level.tiles);

                            if let Some(levels::Tile::Door(index)) = doors {
                                if grounded {
                                    // println!("we should be going to {}", index);

                                    transition_ticks = -20;
                                    next_ind = Some(index)
                                }
                            } else if let Some(levels::Tile::SecretDoor(index)) = doors {
                                if grounded {
                                    // println!("we should be going to {}", index);

                                    transition_ticks = -20;
                                    next_ind = Some(index)
                                }
                            }
                        } else {
                            let p_obj = level.player_obj();
                            let aabb = (p_obj as &mut dyn Object).get_aabb();

                            if levels::check_tilemap_death(aabb, &level.tiles)
                                || levels::check_object_death(aabb, &level.objects)
                            {
                                if levelset.is_some() {
                                    transition_ticks = -80;

                                    deaths += 1;
                                } else {
                                    todo!()
                                }
                            }

                            if levels::check_tilemap_win(aabb, &level.tiles) {
                                *won = true;
                                paused_selection = 0;
                            }
                        }

                        for (_, is_pressed) in keys_pressed.iter_mut() {
                            *is_pressed = false
                        }

                        let d = level.dimensions();
                        let t_r_o_x = if d.0 * TILE_PIXELS < SCREEN_WIDTH {
                            (SCREEN_WIDTH / 2 - d.0 * TILE_PIXELS / 2) as f32
                        } else {
                            let p_pos = level.focus_position().0;
                            let p_pos = p_pos + level.player_vel().0 * 8;
                            if p_pos / PIXEL_SIZE < SCREEN_WIDTH / 2 {
                                0.
                            } else if p_pos / PIXEL_SIZE > d.0 * TILE_PIXELS - SCREEN_WIDTH / 2 {
                                -(d.0 * TILE_PIXELS - SCREEN_WIDTH) as f32
                            } else {
                                -(p_pos / PIXEL_SIZE - SCREEN_WIDTH / 2) as f32
                            }
                        };
                        let t_r_o_y = if d.1 * TILE_PIXELS < SCREEN_HEIGHT {
                            (SCREEN_HEIGHT / 2 - d.1 * TILE_PIXELS / 2) as f32
                        } else {
                            let p_pos = level.focus_position().1;
                            let p_pos = p_pos + level.player_vel().1 * 8;
                            if p_pos / PIXEL_SIZE < SCREEN_HEIGHT / 2 {
                                0.
                            } else if p_pos / PIXEL_SIZE > d.1 * TILE_PIXELS - SCREEN_HEIGHT / 2 {
                                -(d.1 * TILE_PIXELS - SCREEN_HEIGHT) as f32
                            } else {
                                -(p_pos / PIXEL_SIZE - SCREEN_HEIGHT / 2) as f32
                            }
                        };
                        render_off_x = (render_off_x * 11. + t_r_o_x) / 12.;
                        render_off_y = (render_off_y * 11. + t_r_o_y) / 12.;
                    }
                    if remaining_timer * 60. >= 1. {
                        remaining_timer -= 1. / 60.;
                    }
                } else {
                    let delta = get_frame_time();
                    remaining_timer += delta;
                    if remaining_timer * 60. >= 1. {
                        paused_frames += 1;
                        remaining_timer -= 1. / 60.;
                    }
                    if !*won {
                        if is_key_pressed(KeyCode::Down) && paused_selection < 2 {
                            paused_selection += 1;
                        }
                        if is_key_pressed(KeyCode::Up) && paused_selection > 0 {
                            paused_selection -= 1;
                        }
                    } else {
                        if is_key_pressed(KeyCode::Down) && paused_selection < 1 {
                            paused_selection += 1;
                        }
                        if is_key_pressed(KeyCode::Up) && paused_selection > 0 {
                            paused_selection -= 1;
                        }
                    }
                }

                clear_background(WHITE);

                for layer in themes[level.theme].bg.iter() {
                    let s_p_b_x = (render_off_x + level.theme_offset.0 as f32) / TILE_PIXELS as f32;
                    let s_p_b_y = (render_off_y + level.theme_offset.1 as f32) / TILE_PIXELS as f32;

                    let x =
                        layer.off_x + (layer.para_factor_x as f32 * s_p_b_x) as i32 / PIXEL_SIZE;
                    let y =
                        layer.off_y + (layer.para_factor_y as f32 * s_p_b_y) as i32 / PIXEL_SIZE;

                    let t = texture_cache!(&mut textures, &layer.image);

                    draw_texture(&t, x as f32, y as f32, WHITE);
                }

                // draw_rectangle(255., 191., 2., 2., BLUE);
                let d = level.dimensions();
                // draw the rest of the level as well!!
                if levelset.is_some() {
                    levelset.as_ref().expect("is some").levels[*current_ind].propagate_draw(
                        render_off_x as i32,
                        render_off_y as i32,
                        &levelset.as_ref().expect("is some").levels,
                        &mut vec![*current_ind],
                        *current_ind,
                        &global_state.changed_tiles,
                        true,
                        &mut textures,
                        &themes,
                        level.theme,
                    );
                }

                // dim area outside screen

                draw_rectangle(
                    (render_off_x as i32 - 1000) as f32,
                    (render_off_y as i32 - 1000) as f32,
                    1000.,
                    2000. + (d.1 * TILE_PIXELS) as f32,
                    color_u8!(0, 0, 0, 51),
                );

                draw_rectangle(
                    (render_off_x as i32 + (d.0 * TILE_PIXELS)) as f32,
                    (render_off_y as i32 - 1000) as f32,
                    1000.,
                    2000. + (d.1 * TILE_PIXELS) as f32,
                    color_u8!(0, 0, 0, 51),
                );

                draw_rectangle(
                    (render_off_x as i32) as f32,
                    (render_off_y as i32 - 1000) as f32,
                    (d.0 * TILE_PIXELS) as f32,
                    1000.,
                    color_u8!(0, 0, 0, 51),
                );

                draw_rectangle(
                    (render_off_x as i32) as f32,
                    (render_off_y as i32 + d.1 * TILE_PIXELS) as f32,
                    (d.0 * TILE_PIXELS) as f32,
                    1000.,
                    color_u8!(0, 0, 0, 51),
                );

                let transition_type = if transition_ticks < 0 {
                    if next_ind.is_some() {
                        TransitionAnimationType::Door(-transition_ticks)
                    } else {
                        TransitionAnimationType::Death(-transition_ticks)
                    }
                } else {
                    TransitionAnimationType::None
                };

                level.draw(
                    render_off_x as i32,
                    render_off_y as i32,
                    &mut textures,
                    &themes[level.theme],
                    &global_state,
                    &transition_type,
                );

                let player_pos = level.focus_position();
                if transition_ticks < 30 {
                    draw_inverted_circle(
                        (player_pos.0 / PIXEL_SIZE) as f32 + (render_off_x as i32) as f32,
                        (player_pos.1 / PIXEL_SIZE) as f32 + (render_off_y as i32) as f32,
                        64. * ((transition_ticks.abs() as f32) / 12.5).powi(4),
                        BLACK,
                    );
                }

                draw_rectangle(
                    0.,
                    SCREEN_HEIGHT as f32 - 16.,
                    SCREEN_WIDTH as f32,
                    16.,
                    color_u8!(0, 0, 0, 191),
                );

                let x = (SCREEN_WIDTH - level.name.len() as i32 * 7) / 2;

                draw_text(&level.name, x as f32, SCREEN_HEIGHT as f32 - 4., 16., WHITE);

                // let vel = level.player_vel();
                // let g = level.player_obj().air_frames;
                // draw_text(
                //     &format!("h {:0>3}", vel.0.abs() / 16,),
                //     2.,
                //     SCREEN_HEIGHT as f32 - 4.,
                //     16.,
                //     if vel.0.abs() >= 4096 { RED } else { WHITE },
                // );
                // draw_text(
                //     &format!("v {:0>3}", vel.1.abs() / 16,),
                //     44.,
                //     SCREEN_HEIGHT as f32 - 4.,
                //     16.,
                //     if vel.1.abs() >= 4096 { RED } else { WHITE },
                // );

                // let t = format!(
                //     "{}/{} | {:0>2}:{:0>2} | {} death{}",
                //     global_state.secrets,
                //     secret_count,
                //     global_state.timer / 3600,
                //     (global_state.timer / 60) % 60,
                //     deaths,
                //     if deaths == 1 { "" } else { "s" }
                // );
                // let x = SCREEN_WIDTH - t.len() as i32 * 7 - 2;

                // draw_text(&t, x as f32, SCREEN_HEIGHT as f32 - 4., 16., WHITE);

                let mut key_pos = 2.;
                for (count, colour) in global_state
                    .keys
                    .iter()
                    .zip(["red", "yellow", "green", "cyan", "blue", "magenta"])
                {
                    if *count > 0 {
                        let t = texture_cache!(textures, &format!("assets/{}key.png", colour));
                        for i in 0..*count {
                            draw_texture(&t, i as f32 * 18. + 2., key_pos, WHITE);
                        }

                        key_pos += 18.;
                    }
                }

                set_default_camera();

                if paused_frames > 0 {
                    let prog = paused_frames as f32 / 80.;
                    let prog = prog.clamp(0., 1.);
                    let prog = 1. - (1. - prog).powi(3);
                    let prog = prog.clamp(0., 1.);
                    let threed_cam = Camera3D {
                        position: Vec3 {
                            x: SCREEN_WIDTH as f32 * (1. + prog * 1.),
                            y: SCREEN_HEIGHT as f32 * (1. - prog * 1.),
                            z: -637.394 * (1. + prog * 2.),
                        },
                        target: Vec3 {
                            x: SCREEN_WIDTH as f32 * (1. + prog * 0.25),
                            y: SCREEN_HEIGHT as f32 * (1. + prog * 0.1),
                            z: 0.,
                        },
                        up: Vec3 {
                            x: 0.,
                            y: -1.,
                            z: prog / 5.,
                        },
                        fovy: std::f32::consts::FRAC_PI_3 * (3. - prog) / 3.,
                        aspect: None,
                        projection: Projection::Perspective,
                        render_target: None,
                        viewport: None,
                    };
                    set_camera(&threed_cam);
                    draw_cube(
                        Vec3 {
                            x: SCREEN_WIDTH as f32,
                            y: SCREEN_HEIGHT as f32,
                            z: 300.,
                        },
                        Vec3 {
                            x: SCREEN_WIDTH as f32 * 2. + 192.,
                            y: SCREEN_HEIGHT as f32 * 2. + 192.,
                            z: 500.,
                        },
                        None,
                        color_u8!(148, 78, 238, 255),
                    );
                    draw_texture_ex(
                        &render_target.texture,
                        0.,
                        0.,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(Vec2 {
                                x: screen_width(),
                                y: screen_height(),
                            }),
                            ..Default::default()
                        },
                    );

                    if !*won {
                        set_camera(&cam_true);
                        draw_rectangle(
                            0.,
                            0.,
                            SCREEN_WIDTH as f32 * 2.,
                            SCREEN_HEIGHT as f32 * 2.,
                            color_u8!(0, 0, 0, (224. * prog) as u8),
                        );

                        let t = texture_cache!(textures, "assets/pausetopbase.png");
                        draw_texture(&t, 0., (-96. * (1. - prog)) as i32 as f32, WHITE);

                        draw_text(
                            "I need to implement this. For now it will remain static.",
                            88.,
                            48. + (-96. * (1. - prog)) as i32 as f32,
                            32.,
                            WHITE,
                        );
                        let t = texture_cache!(textures, "assets/pauseleftbase.png");
                        draw_texture(&t, (-192. * (1. - prog)) as i32 as f32, 0., WHITE);

                        let numbers = texture_cache!(textures, "assets/numbers.png");

                        let t = format!(
                            "{:0>2}:{:0>2}",
                            global_state.timer / 3600,
                            (global_state.timer / 60) % 60,
                        );
                        draw_number_text(
                            &numbers,
                            &t,
                            64. + (-192. * (1. - prog)) as i32 as f32,
                            149.,
                            BLACK,
                            global_state.timer,
                        );

                        let t = format!("{}/{}", global_state.secrets, secret_count,);

                        draw_number_text(
                            &numbers,
                            &t,
                            65. + (-192. * (1. - prog)) as i32 as f32,
                            186.,
                            color_u8!(79, 6, 79, 255),
                            global_state.timer,
                        );
                        let t = format!("{}", deaths);

                        draw_number_text(
                            &numbers,
                            &t,
                            67. + (-192. * (1. - prog)) as i32 as f32,
                            221.,
                            color_u8!(79, 6, 6, 255),
                            global_state.timer,
                        );

                        let t = texture_cache!(textures, "assets/pausebottom.png");
                        draw_texture(&t, 0., (150. * (1. - prog)) as i32 as f32, WHITE);
                        let t = texture_cache!(textures, "assets/pauserightbase.png");
                        draw_texture(&t, (256. * (1. - prog)) as i32 as f32, 0., WHITE);

                        let t = texture_cache!(
                            textures,
                            if paused_selection == 0 {
                                "assets/pauseresume.png"
                            } else {
                                "assets/pauseresume-dull.png"
                            }
                        );
                        draw_texture(&t, (320. * (1. - prog)) as i32 as f32, 0., WHITE);

                        let t = texture_cache!(
                            textures,
                            if paused_selection == 1 {
                                "assets/pausereset.png"
                            } else {
                                "assets/pausereset-dull.png"
                            }
                        );
                        draw_texture(&t, (320. * (1. - prog)) as i32 as f32, 0., WHITE);

                        let t = texture_cache!(
                            textures,
                            if paused_selection == 2 {
                                "assets/pauseexit.png"
                            } else {
                                "assets/pauseexit-dull.png"
                            }
                        );
                        draw_texture(&t, (320. * (1. - prog)) as i32 as f32, 0., WHITE);
                    }
                } else {
                    draw_texture_ex(
                        &render_target.texture,
                        0.,
                        0.,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(Vec2 {
                                x: screen_width(),
                                y: screen_height(),
                            }),
                            ..Default::default()
                        },
                    );
                }
                set_default_camera();
                if *won {
                    let prog = paused_frames as f32 / 80.;
                    let prog = prog.clamp(0., 1.);
                    let prog = 1. - (1. - prog).powi(3);
                    let prog = prog.clamp(0., 1.);
                    set_camera(&cam_true);
                    draw_rectangle(
                        0.,
                        0.,
                        SCREEN_WIDTH as f32 * 2.,
                        SCREEN_HEIGHT as f32 * 2.,
                        color_u8!(0, 0, 0, (224. * prog) as u8),
                    );

                    let t = texture_cache!(textures, "assets/pauseleftbase.png");
                    draw_texture(&t, (-192. * (1. - prog)) as i32 as f32, 0., WHITE);
                    let t = format!(
                        "{:0>2}:{:0>2}",
                        global_state.timer / 3600,
                        (global_state.timer / 60) % 60,
                    );
                    draw_text(
                        &t,
                        71. + (-192. * (1. - prog)) as i32 as f32,
                        176.,
                        32.,
                        BLACK,
                    );

                    let t = format!("{}/{}", global_state.secrets, secret_count,);
                    draw_text(
                        &t,
                        69. + (-192. * (1. - prog)) as i32 as f32,
                        207.,
                        32.,
                        color_u8!(79, 6, 79, 255),
                    );

                    let t = format!("{}", deaths);
                    draw_text(
                        &t,
                        67. + (-192. * (1. - prog)) as i32 as f32,
                        247.,
                        32.,
                        color_u8!(79, 6, 6, 255),
                    );

                    let t = texture_cache!(textures, "assets/pausebottom.png");
                    draw_texture(&t, 0., (150. * (1. - prog)) as i32 as f32, WHITE);
                    let t = texture_cache!(textures, "assets/winrightbase.png");
                    draw_texture(&t, (256. * (1. - prog)) as i32 as f32, 0., WHITE);

                    let t = texture_cache!(
                        textures,
                        if paused_selection == 0 {
                            "assets/pausereset.png"
                        } else {
                            "assets/pausereset-dull.png"
                        }
                    );
                    draw_texture(&t, (320. * (1. - prog)) as i32 as f32, 0., WHITE);

                    let t = texture_cache!(
                        textures,
                        if paused_selection == 1 {
                            "assets/pauseexit.png"
                        } else {
                            "assets/pauseexit-dull.png"
                        }
                    );
                    draw_texture(&t, (320. * (1. - prog)) as i32 as f32, 0., WHITE);
                    if is_key_pressed(KeyCode::Z) {
                        match paused_selection {
                            0 => {
                                let levelset = levels::load_levelset(&format!(
                                    "levels/{}",
                                    levelsets[levelset_ind]
                                ));
                                let current_ind = 0; // we assume the first level is index 0

                                let level_raw = levelset.levels[current_ind].clone();
                                let level = levels::Level::from_level_raw(
                                    level_raw,
                                    0,
                                    &levelset.levels,
                                    &HashMap::new(),
                                );

                                paused = false;
                                render_off_x = 0.;
                                render_off_y = 0.;

                                themes = levelset.themes.clone();
                                deaths = 0;
                                secret_count = levelset.secret_count;
                                transition_ticks = 0;
                                paused_frames = 0;

                                if themes.len() == 0 {
                                    themes.push(Theme {
                                        ..Default::default()
                                    })
                                }

                                for t in themes.iter() {
                                    t.load_textures(&mut textures).await;
                                }

                                state = State::Game {
                                    levelset: Some(levelset),
                                    current_ind,
                                    level,
                                    global_state: levels::GlobalState::new(),
                                    won: false,
                                }
                            }
                            1 => state = State::Menu(MenuState::Main(0)),
                            _ => unreachable!(),
                        }
                    }
                }
                if paused && is_key_pressed(KeyCode::Z) {
                    if is_key_pressed(KeyCode::Z) {
                        match paused_selection {
                            0 => paused = false,
                            1 => {
                                let levelset = levels::load_levelset(&format!(
                                    "levels/{}",
                                    levelsets[levelset_ind]
                                ));
                                let current_ind = 0; // we assume the first level is index 0

                                let level_raw = levelset.levels[current_ind].clone();
                                let level = levels::Level::from_level_raw(
                                    level_raw,
                                    0,
                                    &levelset.levels,
                                    &HashMap::new(),
                                );

                                paused = false;
                                render_off_x = 0.;
                                render_off_y = 0.;

                                themes = levelset.themes.clone();
                                deaths = 0;
                                secret_count = levelset.secret_count;
                                transition_ticks = 0;
                                paused_frames = 0;

                                if themes.len() == 0 {
                                    themes.push(Theme {
                                        ..Default::default()
                                    })
                                }

                                for t in themes.iter() {
                                    t.load_textures(&mut textures).await;
                                }

                                state = State::Game {
                                    levelset: Some(levelset),
                                    current_ind,
                                    level,
                                    global_state: levels::GlobalState::new(),
                                    won: false,
                                }
                            }
                            2 => state = State::Menu(MenuState::Main(0)),
                            _ => unreachable!(),
                        }
                    }
                }
            } // _ => (),
            State::Edit { levelset, level } => {}
        }

        next_frame().await;
    }
}
