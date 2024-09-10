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
    Settings(usize),
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

    oneway: Option<String>,
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

                        "oneway" => theme.oneway = Some(b.trim().into()),

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
            texture!(textures, self.wall_2.as_ref().expect("is some"));
        }
        if self.wall_3.is_some() {
            texture!(textures, self.wall_3.as_ref().expect("is some"));
        }
        if self.wall_4.is_some() {
            texture!(textures, self.wall_4.as_ref().expect("is some"));
        }

        if self.back_wall_1.is_some() {
            texture!(textures, self.back_wall_1.as_ref().expect("is some"));
        }
        if self.back_wall_2.is_some() {
            texture!(textures, self.back_wall_2.as_ref().expect("is some"));
        }
        if self.back_wall_3.is_some() {
            texture!(textures, self.back_wall_3.as_ref().expect("is some"));
        }
        if self.back_wall_4.is_some() {
            texture!(textures, self.back_wall_4.as_ref().expect("is some"));
        }

        if self.oneway.is_some() {
            texture!(textures, self.oneway.as_ref().expect("is some"));
        }
    }
}

struct Adjacencies {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}

fn draw_inverted_circle(x: f32, y: f32, r: f32, c: Color, wt: Option<&Texture2D>) {
    let k = 0.5;
    let j = 0.75_f32.sqrt();

    if wt.is_none() {
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
    } else {
        draw_rectangle(
            x - r - SCREEN_WIDTH as f32 * 2.,
            y - SCREEN_HEIGHT as f32 * 2.,
            SCREEN_WIDTH as f32 * 2.,
            SCREEN_HEIGHT as f32 * 4.,
            c,
        );
        draw_rectangle(
            x + r,
            y - SCREEN_HEIGHT as f32 * 2.,
            SCREEN_WIDTH as f32 * 2.,
            SCREEN_HEIGHT as f32 * 4.,
            c,
        );
        draw_rectangle(
            x - r,
            y - r - SCREEN_HEIGHT as f32 * 2.,
            r * 2.,
            SCREEN_HEIGHT as f32 * 2.,
            c,
        );
        draw_rectangle(x - r, y + r, r * 2., SCREEN_HEIGHT as f32 * 2., c);
        draw_texture_ex(
            wt.expect("is some"),
            x - r,
            y - r,
            c,
            DrawTextureParams {
                dest_size: Some(Vec2 {
                    x: r * 2.,
                    y: r * 2.,
                }),
                ..Default::default()
            },
        );
    }
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

const FONT_KERN: [i32; 96] = [
    4, 3, 2, 0, 0, 0, 0, 3, 2, 2, 1, 1, 3, 1, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 3, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 2, 0, 1,
    2, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0,
];
const FONT_Y_OFF: [i32; 96] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 3, 0, 0, 0, 0, 0, 3, 3, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0,
];

fn draw_text_cool(tx: &Texture2D, t: &str, x: i32, y: i32, c: Color) {
    let mut back_off = 0;
    for (i, ch) in t.chars().enumerate() {
        let ind = ch as u32;
        let ind = if ind >= 32 && ind <= 127 {
            ind - 32
        } else {
            95
        };
        let (sx, sy) = (ind % 16, ind / 16);

        let kern = FONT_KERN[ind as usize];
        let yoff = FONT_Y_OFF[ind as usize];

        draw_texture_ex(
            &tx,
            (x + i as i32 * 12 - kern - back_off) as f32,
            (y + yoff) as f32,
            c,
            DrawTextureParams {
                source: Some(Rect {
                    x: sx as f32 * 12.,
                    y: sy as f32 * 16.,
                    w: 12.,
                    h: 16.,
                }),
                ..Default::default()
            },
        );

        back_off += kern * 2;
    }
}

fn draw_text_cool_c(tx: &Texture2D, t: &str, x: i32, y: i32, c: Color) {
    let mut total_width = 0;
    for ch in t.chars() {
        let ind = ch as u32;
        let ind = if ind >= 32 && ind <= 127 {
            ind - 32
        } else {
            95
        };
        let kern = FONT_KERN[ind as usize];
        total_width += 12 - kern * 2;
    }

    draw_text_cool(tx, t, x - total_width / 2, y, c);
}

fn draw_text_cool_l(tx: &Texture2D, t: &str, x: i32, y: i32, c: Color) {
    let mut total_width = 0;
    for ch in t.chars() {
        let ind = ch as u32;
        let ind = if ind >= 32 && ind <= 127 {
            ind - 32
        } else {
            95
        };
        let kern = FONT_KERN[ind as usize];
        total_width += 12 - kern * 2;
    }

    draw_text_cool(tx, t, x - total_width, y, c);
}

fn draw_tip_text(tx: &Texture2D, t: &str, x: i32, y: i32, w: i32, slant_every: i32, c: Color) {
    let mut c_line_width = 0;
    let mut lines: Vec<usize> = vec![0];
    for word in t.split(" ") {
        let mut my_width = 0;
        for ch in word.chars() {
            let ind = ch as u32;
            let ind = if ind >= 32 && ind <= 127 {
                ind - 32
            } else {
                95
            };
            let (sx, sy) = (ind % 16, ind / 16);

            let kern = FONT_KERN[ind as usize];
            my_width += 12 - kern * 2;
        }
        if c_line_width + my_width < w {
            c_line_width += my_width + 4;
            *lines.last_mut().expect("should have last") += word.len() + 1
        } else {
            c_line_width = my_width + 4;
            lines.push(word.len() + 1);
        }
    }
    let mut back_off = 0;
    let mut current_line = 0;
    let mut ind_off = 0;
    for (i, ch) in t.chars().enumerate() {
        if i - ind_off >= lines[current_line] {
            back_off = 0;
            ind_off += lines[current_line];
            current_line += 1;
        }
        let ind = ch as u32;
        let ind = if ind >= 32 && ind <= 127 {
            ind - 32
        } else {
            95
        };
        let (sx, sy) = (ind % 16, ind / 16);

        let kern = FONT_KERN[ind as usize];
        let yoff = FONT_Y_OFF[ind as usize];

        let slant = (i - ind_off) as i32 / slant_every;

        draw_texture_ex(
            &tx,
            (x + (i - ind_off) as i32 * 12 - kern - back_off) as f32,
            (y + yoff + current_line as i32 * 18 - lines.len() as i32 * 9 - slant) as f32,
            c,
            DrawTextureParams {
                source: Some(Rect {
                    x: sx as f32 * 12.,
                    y: sy as f32 * 16.,
                    w: 12.,
                    h: 16.,
                }),
                ..Default::default()
            },
        );

        back_off += kern * 2;
    }
}

enum TransitionAnimationType {
    None,
    Death(i32),
    Door(bool),
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

struct Settings {
    fullscreen: bool,
    show_fps: bool,
    show_input: bool,
    show_stats: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Settings {
            fullscreen: false,
            show_fps: false,
            show_input: false,
            show_stats: false,
        }
    }
}
impl Settings {
    fn load(path: &str) -> Self {
        let s = std::fs::read_to_string(path);
        if s.is_err() {
            // fuck
            let settings = Settings::default();
            Settings::save(path, &settings);
            return settings;
        }
        let s = s.expect("is not err");
        let mut new_settings = Settings::default();

        for l in s.lines() {
            if l.split(": ").count() < 2 {
                continue;
            }
            let mut parts = l.split(": ");
            let key = parts.next().expect("length >= 2");
            let val = parts.next().expect("length >= 2");
            match key {
                "fullscreen" => new_settings.fullscreen = val.trim() == "true",
                "show_fps" => new_settings.show_fps = val.trim() == "true",
                "show_input" => new_settings.show_input = val.trim() == "true",
                "show_stats" => new_settings.show_stats = val.trim() == "true",
                _ => (),
            }
        }

        new_settings
    }
    fn save(path: &str, s: &Settings) {
        let mut output_str = "".to_string();

        output_str.push_str(&format!("fullscreen: {}\n", s.fullscreen));
        output_str.push_str(&format!("show_fps: {}\n", s.show_fps));
        output_str.push_str(&format!("show_input: {}\n", s.show_input));
        output_str.push_str(&format!("show_stats: {}\n", s.show_stats));

        let _ = std::fs::write(path, &output_str);
    }
    fn apply(&self) {
        set_fullscreen(self.fullscreen);
        if !self.fullscreen {
            request_new_screen_size((SCREEN_WIDTH * 2) as f32, (SCREEN_HEIGHT * 2) as f32)
        }
    }
}

const PAUSE_BG_FRAGMENT_SHADER: &'static str = include_str!("pause_bg.frag");
const DEFAULT_VERTEX_SHADER: &'static str = "#version 100
precision lowp float;

attribute vec3 position;
attribute vec2 texcoord;

varying vec2 uv;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    uv = texcoord;
}
";

#[macroquad::main(window_conf)]
async fn main() {
    let mut settings = Settings::load("settings");
    settings.apply();
    let mut textures: HashMap<String, Texture2D> = HashMap::new();

    let fs = PAUSE_BG_FRAGMENT_SHADER.to_string();
    let vs = DEFAULT_VERTEX_SHADER.to_string();

    let bg_material = load_material(
        ShaderSource::Glsl {
            vertex: &vs,
            fragment: &fs,
        },
        MaterialParams {
            uniforms: vec![
                UniformDesc {
                    name: "iResolution".into(),
                    uniform_type: UniformType::Float2,
                    array_count: 1,
                },
                UniformDesc {
                    name: "iTime".into(),
                    uniform_type: UniformType::Float1,
                    array_count: 1,
                },
            ],
            ..Default::default()
        },
    )
    .unwrap();

    bg_material.set_uniform("iResolution", [SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32]);

    let tips = std::fs::read_to_string("tips.txt").expect("Tips are an essential feature.");
    let tips: Vec<&str> = tips.lines().collect();

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
        "assets/secretwindow.png",
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
        "assets/letters.png",
        "assets/binocular.png",
        "assets/pausebg.png",
        "assets/buttondisplay.png",
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
    let mut secret_transition = false;
    let mut next_ind: Option<usize> = None;
    let mut levelset_ind = 0;

    let mut global_timer = 0.;

    let font = texture_cache!(textures, "assets/letters.png");

    loop {
        clear_background(WHITE);

        set_default_camera();

        match &mut state {
            State::Menu(menu_state) => {
                clear_background(BLACK);

                set_camera(&cam_true);
                match menu_state {
                    MenuState::Main(ind) => {
                        draw_text_cool(&font, "main menu (temporary)", 4, 2, WHITE);

                        for (i, o) in ["play", "settings", "quit"].iter().enumerate() {
                            draw_text_cool(
                                &font,
                                &format!("{}{}", if *ind == i { "> " } else { "    " }, o),
                                4,
                                22 + 20 * i as i32,
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
                                1 => *menu_state = MenuState::Settings(0),
                                2 => panic!("user closed game"),
                                _ => (),
                            }
                        }
                    }
                    MenuState::LevelsetSelect(ind) => {
                        draw_text_cool(&font, "select levelset", 4, 2, WHITE);

                        for (i, l) in levelsets.iter().chain(["back".into()].iter()).enumerate() {
                            draw_text_cool(
                                &font,
                                &format!("{}{}", if *ind == i { "> " } else { "    " }, l),
                                4,
                                22 + 16 * i as i32,
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
                                secret_transition = false;
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
                    MenuState::Settings(ind) => {
                        draw_text_cool(&font, "settings", 4, 2, WHITE);
                        let m = |b| if b { "yes" } else { "no" };
                        let things = [
                            ("fullscreen", m(settings.fullscreen)),
                            ("show fps", m(settings.show_fps)),
                            ("show input", m(settings.show_input)),
                            ("show stats", m(settings.show_stats)),
                            ("back", ""),
                        ];

                        for (i, (t, v)) in things.iter().enumerate() {
                            draw_text_cool(
                                &font,
                                &format!(
                                    "{}{}{}{}",
                                    if *ind == i { "> " } else { "    " },
                                    t,
                                    if *v != "" { ": " } else { "" },
                                    v
                                ),
                                4,
                                22 + 20 * i as i32,
                                WHITE,
                            );
                        }
                        if is_key_pressed(KeyCode::Down) && *ind < things.len() - 1 {
                            *ind += 1
                        }
                        if is_key_pressed(KeyCode::Up) && *ind > 0 {
                            *ind -= 1
                        }
                        if is_key_pressed(KeyCode::Z) {
                            match things[*ind].0 {
                                "fullscreen" => {
                                    settings.fullscreen = !settings.fullscreen;
                                    Settings::save("settings", &settings);
                                    settings.apply();
                                }
                                "show fps" => {
                                    settings.show_fps = !settings.show_fps;
                                    Settings::save("settings", &settings);
                                }
                                "show input" => {
                                    settings.show_input = !settings.show_input;
                                    Settings::save("settings", &settings);
                                }
                                "show stats" => {
                                    settings.show_stats = !settings.show_stats;
                                    Settings::save("settings", &settings);
                                }
                                "back" => *menu_state = MenuState::Main(0),
                                _ => (),
                            }
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
                    if global_state.binocularing {
                        global_state.binocularing = false;
                    } else {
                        paused = !paused;
                        paused_selection = 0;
                    }
                }
                if !paused && !*won {
                    let delta = get_frame_time();
                    global_timer += delta;
                    remaining_timer += delta;

                    for (keycode, is_pressed) in keys_pressed.iter_mut() {
                        if is_key_pressed(*keycode) {
                            *is_pressed = true
                        }
                    }
                    paused_frames = (paused_frames - 2).clamp(0, 80);
                    if remaining_timer * 60. >= 1. {
                        if global_state.binocularing {
                            global_state.binocular_t = (global_state.binocular_t + 1).min(30);
                        } else {
                            global_state.binocular_t = (global_state.binocular_t - 1).max(0);
                        }
                        transition_ticks += 1;
                        if levelset.is_some() {
                            let l = levelset.as_ref().expect("is some").levels.len() - 1;
                            if *keys_pressed.entry(KeyCode::LeftBracket).or_insert(false) {
                                transition_ticks = -20;
                                secret_transition = false;
                                if *current_ind == 0 {
                                    next_ind = Some(l);
                                } else {
                                    next_ind = Some(*current_ind - 1);
                                }
                                keys_pressed.insert(KeyCode::LeftBracket, false);
                            }
                            if *keys_pressed.entry(KeyCode::RightBracket).or_insert(false) {
                                transition_ticks = -20;
                                secret_transition = false;
                                if *current_ind == l {
                                    next_ind = Some(0);
                                } else {
                                    next_ind = Some(*current_ind + 1);
                                }
                                keys_pressed.insert(KeyCode::RightBracket, false);
                            }
                        }
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
                                    secret_transition = false;

                                    deaths += 1;
                                } else {
                                    todo!()
                                }
                            }
                        } else if levelset.is_some()
                            && *keys_pressed.entry(KeyCode::Up).or_insert(false)
                            && !global_state.binocularing
                        {
                            let p_obj = level.player_obj();
                            let grounded = p_obj.grounded;
                            let aabb = (p_obj as &mut dyn Object).get_aabb();

                            let doors = levels::check_door(aabb, &level.tiles);

                            if let Some(levels::Tile::Door(index)) = doors {
                                if grounded {
                                    // println!("we should be going to {}", index);

                                    transition_ticks = -20;
                                    secret_transition = false;
                                    next_ind = Some(index)
                                }
                            } else if let Some(levels::Tile::SecretDoor(index)) = doors {
                                if grounded {
                                    // println!("we should be going to {}", index);

                                    transition_ticks = -20;
                                    secret_transition = true;
                                    next_ind = Some(index)
                                }
                            } else if let Some(levels::Tile::Binocular) = doors {
                                if grounded {
                                    global_state.binocularing = true;
                                    global_state.binocular_rx = if d.0 * TILE_PIXELS < SCREEN_WIDTH
                                    {
                                        SCREEN_WIDTH / 2 - d.0 * TILE_PIXELS / 2
                                    } else {
                                        let p_pos = level.focus_position().0;
                                        let p_pos = p_pos + level.player_vel().0 * 8;
                                        if p_pos / PIXEL_SIZE < SCREEN_WIDTH / 2 {
                                            0
                                        } else if p_pos / PIXEL_SIZE
                                            > d.0 * TILE_PIXELS - SCREEN_WIDTH / 2
                                        {
                                            -(d.0 * TILE_PIXELS - SCREEN_WIDTH)
                                        } else {
                                            -(p_pos / PIXEL_SIZE - SCREEN_WIDTH / 2)
                                        }
                                    };
                                    global_state.binocular_ry = if d.1 * TILE_PIXELS < SCREEN_HEIGHT
                                    {
                                        SCREEN_HEIGHT / 2 - d.1 * TILE_PIXELS / 2
                                    } else {
                                        let p_pos = level.focus_position().1;
                                        let p_pos = p_pos + level.player_vel().1 * 8;
                                        if p_pos / PIXEL_SIZE < SCREEN_HEIGHT / 2 {
                                            0
                                        } else if p_pos / PIXEL_SIZE
                                            > d.1 * TILE_PIXELS - SCREEN_HEIGHT / 2
                                        {
                                            -(d.1 * TILE_PIXELS - SCREEN_HEIGHT)
                                        } else {
                                            -(p_pos / PIXEL_SIZE - SCREEN_HEIGHT / 2)
                                        }
                                    };
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

                                    secret_transition = false;

                                    deaths += 1;
                                } else {
                                    todo!()
                                }
                            }

                            if levels::check_tilemap_win(aabb, &level.tiles) {
                                *won = true;
                                clear_input_queue();
                                paused_selection = 0;
                            }
                        }

                        for (_, is_pressed) in keys_pressed.iter_mut() {
                            *is_pressed = false
                        }

                        let d = level.dimensions();
                        if !global_state.binocularing {
                            let t_r_o_x = if d.0 * TILE_PIXELS < SCREEN_WIDTH {
                                (SCREEN_WIDTH / 2 - d.0 * TILE_PIXELS / 2) as f32
                            } else {
                                let p_pos = level.focus_position().0;
                                let p_pos = p_pos + level.player_vel().0 * 8;
                                if p_pos / PIXEL_SIZE < SCREEN_WIDTH / 2 {
                                    0.
                                } else if p_pos / PIXEL_SIZE > d.0 * TILE_PIXELS - SCREEN_WIDTH / 2
                                {
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
                                } else if p_pos / PIXEL_SIZE > d.1 * TILE_PIXELS - SCREEN_HEIGHT / 2
                                {
                                    -(d.1 * TILE_PIXELS - SCREEN_HEIGHT) as f32
                                } else {
                                    -(p_pos / PIXEL_SIZE - SCREEN_HEIGHT / 2) as f32
                                }
                            };
                            render_off_x = (render_off_x * 11. + t_r_o_x) / 12.;
                            render_off_y = (render_off_y * 11. + t_r_o_y) / 12.;
                        } else {
                            if is_key_down(KeyCode::Up) {
                                global_state.binocular_ry += 5;
                            }
                            if is_key_down(KeyCode::Down) {
                                global_state.binocular_ry -= 5;
                            }
                            if is_key_down(KeyCode::Left) {
                                global_state.binocular_rx += 5;
                            }
                            if is_key_down(KeyCode::Right) {
                                global_state.binocular_rx -= 5;
                            }

                            global_state.binocular_rx = global_state
                                .binocular_rx
                                .clamp((d.0 - 40) * -TILE_PIXELS, 0);
                            global_state.binocular_ry = global_state
                                .binocular_ry
                                .clamp((d.1 - 23) * -TILE_PIXELS, 0);

                            render_off_x =
                                (render_off_x * 7. + global_state.binocular_rx as f32) / 8.;
                            render_off_y =
                                (render_off_y * 7. + global_state.binocular_ry as f32) / 8.;
                        }
                    }
                    if remaining_timer * 60. >= 1. {
                        remaining_timer -= 1. / 60.;
                    }
                } else {
                    let delta = get_frame_time();
                    global_timer += delta;
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

                    let x = layer.off_x
                        + (layer.para_factor_x as f32 * s_p_b_x
                            + (global_state.timer * layer.scroll_x) as f32)
                            as i32
                            % if layer.mod_x == 1 {
                                std::i32::MAX
                            } else {
                                layer.mod_x
                            }
                            / PIXEL_SIZE;
                    let y = layer.off_y
                        + (layer.para_factor_y as f32 * s_p_b_y
                            + (global_state.timer * layer.scroll_y) as f32)
                            as i32
                            % if layer.mod_y == 1 {
                                std::i32::MAX
                            } else {
                                layer.mod_y
                            }
                            / PIXEL_SIZE;

                    let t = texture_cache!(&mut textures, &layer.image);

                    draw_texture(&t, x as f32, y as f32, WHITE);

                    if layer.mod_x != 1 {
                        draw_texture(&t, (x - layer.mod_x / PIXEL_SIZE) as f32, y as f32, WHITE);
                        draw_texture(&t, (x + layer.mod_x / PIXEL_SIZE) as f32, y as f32, WHITE);
                    }

                    if layer.mod_y != 1 {
                        draw_texture(&t, x as f32, (y - layer.mod_y / PIXEL_SIZE) as f32, WHITE);
                        draw_texture(&t, x as f32, (y + layer.mod_y / PIXEL_SIZE) as f32, WHITE);
                    }

                    if layer.mod_x != 1 && layer.mod_y != -1 {
                        draw_texture(
                            &t,
                            (x - layer.mod_x / PIXEL_SIZE) as f32,
                            (y - layer.mod_y / PIXEL_SIZE) as f32,
                            WHITE,
                        );
                        draw_texture(
                            &t,
                            (x + layer.mod_x / PIXEL_SIZE) as f32,
                            (y - layer.mod_y / PIXEL_SIZE) as f32,
                            WHITE,
                        );
                        draw_texture(
                            &t,
                            (x - layer.mod_x / PIXEL_SIZE) as f32,
                            (y + layer.mod_y / PIXEL_SIZE) as f32,
                            WHITE,
                        );
                        draw_texture(
                            &t,
                            (x + layer.mod_x / PIXEL_SIZE) as f32,
                            (y + layer.mod_y / PIXEL_SIZE) as f32,
                            WHITE,
                        );
                    }
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
                        TransitionAnimationType::Door(secret_transition)
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
                    let t = if secret_transition {
                        Some(texture_cache!(textures, "assets/secretwindow.png"))
                    } else {
                        None
                    };
                    draw_inverted_circle(
                        (player_pos.0 / PIXEL_SIZE) as f32 + (render_off_x as i32) as f32,
                        (player_pos.1 / PIXEL_SIZE) as f32 + (render_off_y as i32) as f32,
                        64. * ((transition_ticks.abs() as f32) / 12.5).powi(4),
                        BLACK,
                        t.as_ref(),
                    );
                }
                if global_state.binocular_t > 0 {
                    let p = ((1.
                        - (1. - (global_state.binocular_t as f32 / 30.).clamp(0., 1.)).powi(2))
                        * 18.) as i32;
                    draw_rectangle(0., 0., p as f32, SCREEN_HEIGHT as f32, BLACK);
                    draw_rectangle(
                        (SCREEN_WIDTH - p) as f32,
                        0.,
                        p as f32,
                        SCREEN_HEIGHT as f32,
                        BLACK,
                    );
                    draw_rectangle(0., 0., SCREEN_WIDTH as f32, p as f32, BLACK);
                    draw_rectangle(
                        0.,
                        (SCREEN_HEIGHT - p) as f32,
                        SCREEN_WIDTH as f32,
                        p as f32,
                        BLACK,
                    );
                }

                draw_rectangle(
                    0.,
                    SCREEN_HEIGHT as f32 - 18.,
                    SCREEN_WIDTH as f32,
                    18.,
                    color_u8!(0, 0, 0, 191),
                );

                draw_text_cool_c(
                    &font,
                    &level.name,
                    SCREEN_WIDTH / 2,
                    SCREEN_HEIGHT - 17,
                    WHITE,
                );

                if settings.show_stats {
                    let vel = level.player_vel();
                    // let g = level.player_obj().air_frames;
                    draw_text_cool(
                        &font,
                        &format!("h{:0>3}", vel.0.abs() / 16,),
                        2,
                        SCREEN_HEIGHT - 17,
                        if vel.0.abs() >= 4096 { RED } else { WHITE },
                    );
                    draw_text_cool(
                        &font,
                        &format!("v{:0>3}", vel.1.abs() / 16,),
                        50,
                        SCREEN_HEIGHT - 17,
                        if vel.1.abs() >= 4096 { RED } else { WHITE },
                    );

                    // this will break when stuff happens
                    let t = format!(
                        "{}/{}|{:0>2}:{:0>2}|Æ{}",
                        global_state.secrets,
                        secret_count,
                        global_state.timer / 3600,
                        (global_state.timer / 60) % 60,
                        deaths,
                    );
                    draw_text_cool_l(&font, &t, SCREEN_WIDTH - 2, SCREEN_HEIGHT - 17, WHITE);
                }

                if settings.show_input {
                    let t = texture!(&mut textures, "assets/buttondisplay.png");
                    let buttons = [
                        ((0, 0), is_key_down(KeyCode::Escape)),
                        ((16, 0), is_key_down(KeyCode::Up)),
                        ((0, 16), is_key_down(KeyCode::Left)),
                        ((16, 16), is_key_down(KeyCode::Down)),
                        ((32, 16), is_key_down(KeyCode::Right)),
                        ((48, 16), is_key_down(KeyCode::Z)),
                        ((48, 0), is_key_down(KeyCode::X)),
                    ];
                    for ((ox, oy), down) in buttons.iter() {
                        let n_oy = if *down { *oy + 32 } else { *oy };
                        draw_texture_ex(
                            &t,
                            (2 + ox) as f32,
                            (SCREEN_HEIGHT - 51 + oy) as f32,
                            WHITE,
                            DrawTextureParams {
                                source: Some(Rect {
                                    x: *ox as f32,
                                    y: n_oy as f32,
                                    w: 16.,
                                    h: 16.,
                                }),
                                ..Default::default()
                            },
                        );
                    }
                }

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
                            x: SCREEN_WIDTH as f32
                                * if !paused && !*won {
                                    1. + (prog * 5. - 4.).max(0.)
                                } else {
                                    1. + prog * 1.
                                },
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
                        fovy: std::f32::consts::FRAC_PI_3 * (7. + prog) / 7.,
                        aspect: None,
                        projection: Projection::Perspective,
                        render_target: None,
                        viewport: None,
                    };

                    set_camera(&cam_true);
                    let t = texture_cache!(textures, "assets/pausebg.png");
                    draw_texture(&t, 0., 0., WHITE);
                    gl_use_default_material();

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
                        color_u8!(148, 78, 237, 255),
                    );

                    draw_texture_ex(
                        &render_target.texture,
                        0.,
                        0.,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(Vec2 {
                                x: (SCREEN_WIDTH * 2) as f32,
                                y: (SCREEN_HEIGHT * 2) as f32,
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
                            color_u8!(0, 0, 0, (128. * prog) as u8),
                        );

                        // if levelset.is_some() {
                        //     let sy = levelset.as_ref().expect("is some").levels[*current_ind].tiles
                        //         [0]
                        //     .len();
                        //     let sx = levelset.as_ref().expect("is some").levels[*current_ind].tiles
                        //         [0][0]
                        //         .len();
                        //     levelset.as_ref().expect("is some").levels[*current_ind].minimap_draw(
                        //         SCREEN_WIDTH / 2 - sx as i32,
                        //         ((SCREEN_HEIGHT / 2) as f32 * (5. - 4. * prog)) as i32 - sy as i32,
                        //         &levelset.as_ref().expect("is some").levels,
                        //         &mut vec![*current_ind],
                        //         *current_ind,
                        //         &global_state.changed_tiles,
                        //     );
                        // }

                        let t = texture_cache!(textures, "assets/pausetopbase.png");
                        draw_texture(&t, 0., (-96. * (1. - prog)) as i32 as f32, WHITE);

                        draw_tip_text(
                            &font,
                            &tips[global_state.timer as usize % tips.len()],
                            78,
                            42 + (-100. * (1. - prog)) as i32,
                            330,
                            5,
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
                            64. + (-200. * (1. - prog)) as i32 as f32,
                            149.,
                            BLACK,
                            global_state.timer,
                        );

                        let t = format!("{}/{}", global_state.secrets, secret_count,);

                        draw_number_text(
                            &numbers,
                            &t,
                            65. + (-200. * (1. - prog)) as i32 as f32,
                            186.,
                            color_u8!(79, 6, 79, 255),
                            global_state.timer,
                        );
                        let t = format!("{}", deaths);

                        draw_number_text(
                            &numbers,
                            &t,
                            67. + (-200. * (1. - prog)) as i32 as f32,
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
                                secret_transition = false;
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
                                secret_transition = false;
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

        if settings.show_fps {
            draw_text_cool(
                &font,
                format!("{} fps", get_fps()).as_str(),
                0,
                0,
                color_u8!(0, 255, 0, 255),
            )
        }

        next_frame().await;
    }
}
