use std::collections::HashMap;

use macroquad::prelude::*;

const PIXEL_SIZE: i32 = 256;
const TILE_PIXELS: i32 = 16;
const TILE_SIZE: i32 = TILE_PIXELS * PIXEL_SIZE;

const MAX_PLAYER_SPEED: i32 = TILE_SIZE * 3 / 16;
const PLAYER_ACCEL: i32 = TILE_SIZE / 16;

const SCREEN_WIDTH: i32 = 640;
const SCREEN_HEIGHT: i32 = 360;

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
    },
    Edit {
        levelset: String,
        level: Option<String>,
    },
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
                println!("{}", t_path);
                return None;
            }
            println!("found levelset {:?}", a);
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
    ];

    for p in preload_textures {
        texture!(&mut textures, p);
    }

    let cam = Camera2D::from_display_rect(Rect::new(
        0.,
        SCREEN_HEIGHT as f32,
        SCREEN_WIDTH as f32,
        -SCREEN_HEIGHT as f32,
    ));

    set_camera(&cam);
    let mut render_off_x = 0.;
    let mut render_off_y = 0.;

    let mut remaining_timer = 0.;
    let mut paused = false;

    let mut keys_pressed: HashMap<KeyCode, bool> = HashMap::new();

    let mut state = State::Menu(MenuState::Main(0));

    loop {
        clear_background(WHITE);

        match &mut state {
            State::Menu(menu_state) => {
                clear_background(BLACK);
                match menu_state {
                    MenuState::Main(ind) => {
                        draw_text("main menu (temporary)", 4., 12., 16., WHITE);

                        for (i, o) in ["play", "edit", "quit"].iter().enumerate() {
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
                                1 => *menu_state = MenuState::EditSelect(0),
                                2 => panic!("user closed game"),
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
                                let current_ind = 0; // we assume the first level is index 0

                                let level_raw = levelset.levels[current_ind].clone();
                                let level =
                                    levels::Level::from_level_raw(level_raw, 0, &HashMap::new());

                                paused = false;
                                render_off_x = 0.;
                                render_off_y = 0.;

                                state = State::Game {
                                    levelset: Some(levelset),
                                    current_ind,
                                    level,
                                    global_state: levels::GlobalState::new(),
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
            } => {
                if is_key_pressed(KeyCode::Escape) {
                    paused = !paused
                }
                if !paused {
                    let delta = get_frame_time();
                    remaining_timer += delta;

                    for (keycode, is_pressed) in keys_pressed.iter_mut() {
                        if is_key_pressed(*keycode) {
                            *is_pressed = true
                        }
                    }

                    if remaining_timer * 60. >= 1. {
                        remaining_timer -= 1. / 60.;
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
                                    &global_state.changed_tiles,
                                );
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
                                    &global_state.changed_tiles,
                                );
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
                                    &global_state.changed_tiles,
                                );
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
                                    &global_state.changed_tiles,
                                );
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
                                    let level_raw = levelset.as_ref().expect("is some").levels
                                        [*current_ind]
                                        .clone();
                                    *level = levels::Level::from_level_raw(
                                        level_raw,
                                        *current_ind,
                                        &global_state.changed_tiles,
                                    );
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
                                    println!("we should be going to {}", index);

                                    let level_raw =
                                        levelset.as_ref().expect("is some").levels[index].clone();

                                    let old_ind = *current_ind;
                                    *current_ind = index;
                                    *level = levels::Level::from_level_raw(
                                        level_raw,
                                        *current_ind,
                                        &global_state.changed_tiles,
                                    );

                                    let p_pos = levels::find_door(old_ind, &level.tiles);
                                    if let Some((x, y)) = p_pos {
                                        let p_obj = level.player_obj();

                                        (p_obj.x, p_obj.y) = (x * TILE_SIZE, y * TILE_SIZE);
                                    }

                                    let new_d = level.dimensions();

                                    render_off_x =
                                        (SCREEN_WIDTH / 2 - new_d.0 * TILE_PIXELS / 2) as f32;
                                    render_off_y =
                                        (SCREEN_HEIGHT / 2 - new_d.1 * TILE_PIXELS / 2) as f32;
                                }
                            }
                        } else {
                            let p_obj = level.player_obj();
                            let aabb = (p_obj as &mut dyn Object).get_aabb();

                            if levels::check_tilemap_death(aabb, &level.tiles) {
                                if levelset.is_some() {
                                    let level_raw = levelset.as_ref().expect("is some").levels
                                        [*current_ind]
                                        .clone();
                                    *level = levels::Level::from_level_raw(
                                        level_raw,
                                        *current_ind,
                                        &global_state.changed_tiles,
                                    );
                                } else {
                                    todo!()
                                }
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

                level.draw(render_off_x as i32, render_off_y as i32, &mut textures);

                draw_rectangle(
                    0.,
                    SCREEN_HEIGHT as f32 - 16.,
                    SCREEN_WIDTH as f32,
                    16.,
                    color_u8!(0, 0, 0, 191),
                );

                let x = (SCREEN_WIDTH - level.name.len() as i32 * 8) / 2;

                draw_text(&level.name, x as f32, SCREEN_HEIGHT as f32 - 4., 16., WHITE);

                let vel = level.player_vel();
                let g = level.player_obj().air_frames;
                draw_text(
                    &format!(
                        "h {:0>3} v {:0>3} g {}",
                        vel.0.abs() / 16,
                        vel.1.abs() / 16,
                        (15 - g).max(0)
                    ),
                    2.,
                    SCREEN_HEIGHT as f32 - 4.,
                    16.,
                    WHITE,
                );

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

                if paused {
                    draw_rectangle(
                        0.,
                        0.,
                        SCREEN_WIDTH as f32,
                        SCREEN_HEIGHT as f32,
                        color_u8!(0, 0, 0, 192),
                    );

                    draw_text("paused !!", 4., 12., 16., WHITE);
                    draw_text("q to quit", 4., 28., 16., WHITE);

                    if is_key_pressed(KeyCode::Q) {
                        state = State::Menu(MenuState::Main(0))
                    }
                }
            } // _ => (),
            State::Edit { levelset, level } => {}
        }

        next_frame().await;
    }
}
