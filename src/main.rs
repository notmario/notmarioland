use std::collections::HashMap;

use macroquad::prelude::*;

const PIXEL_SIZE: i32 = 256;
const TILE_PIXELS: i32 = 16;
const TILE_SIZE: i32 = TILE_PIXELS * PIXEL_SIZE;

const MAX_PLAYER_SPEED: i32 = TILE_SIZE * 3 / 16;
const PLAYER_ACCEL: i32 = TILE_SIZE / 16;

const SCREEN_WIDTH: i32 = 512;
const SCREEN_HEIGHT: i32 = 384;

mod levels;
use levels::Object;

mod macros;

enum MenuState {
    Main(usize),
    LevelsetSelect(usize),
}

enum State {
    Menu(MenuState),
    Game,
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

    let preload_textures = ["assets/player.png"];

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

    let levelset = levels::load_levelset("levels/testset");
    let mut current_ind = 0;

    let level_raw = levelset.levels[current_ind].clone();
    let mut level = levels::Level::from_level_raw(level_raw);

    let mut render_off_x = 0.;
    let mut render_off_y = 0.;

    let mut remaining_timer = 0.;
    let mut paused = false;

    let mut keys_pressed: HashMap<KeyCode, bool> = HashMap::new();

    loop {
        clear_background(WHITE);

        if is_key_pressed(KeyCode::P) {
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
                level.update(&mut keys_pressed);

                // check if we should exit!!
                let player_pos = level.focus_position();
                let player_vel = level.player_vel();
                let d = level.dimensions();
                if player_pos.0 < 0 && player_vel.0 < 0 {
                    if level.side_exits.left.is_some() {
                        let old_sliding = level.player_obj().wall_sliding;
                        let old_freeze = level.player_obj().freeze_timer;

                        let r_off_y = level.side_offsets.left.expect("should have an exit anchor");
                        let off_y = player_pos.1 - TILE_SIZE / 2 - r_off_y;
                        let new_x = player_pos.0 - TILE_SIZE / 2;

                        let level_raw =
                            levelset.levels[level.side_exits.left.expect("is some")].clone();

                        current_ind = level.side_exits.left.expect("is some");
                        level = levels::Level::from_level_raw(level_raw);
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
                } else if player_pos.0 > d.0 * TILE_SIZE && player_vel.0 > 0 {
                    if level.side_exits.right.is_some() {
                        let old_sliding = level.player_obj().wall_sliding;
                        let old_freeze = level.player_obj().freeze_timer;

                        let r_off_y = level
                            .side_offsets
                            .right
                            .expect("should have an exit anchor");
                        let off_y = player_pos.1 - TILE_SIZE / 2 - r_off_y;
                        let new_x = player_pos.0 - d.0 * TILE_SIZE - TILE_SIZE / 2;

                        let level_raw =
                            levelset.levels[level.side_exits.right.expect("is some")].clone();

                        current_ind = level.side_exits.right.expect("is some");
                        level = levels::Level::from_level_raw(level_raw);
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
                } else if player_pos.1 < 0 && player_vel.1 < 0 {
                    if level.side_exits.up.is_some() {
                        let old_sliding = level.player_obj().wall_sliding;
                        let old_freeze = level.player_obj().freeze_timer;

                        let r_off_x = level.side_offsets.up.expect("should have an exit anchor");
                        let off_x = player_pos.0 - TILE_SIZE / 2 - r_off_x;
                        let new_y = player_pos.1 - TILE_SIZE / 2;

                        let level_raw =
                            levelset.levels[level.side_exits.up.expect("is some")].clone();

                        current_ind = level.side_exits.up.expect("is some");
                        level = levels::Level::from_level_raw(level_raw);
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
                } else if player_pos.1 > d.1 * TILE_SIZE && player_vel.1 > 0 {
                    if level.side_exits.down.is_some() {
                        let old_sliding = level.player_obj().wall_sliding;
                        let old_freeze = level.player_obj().freeze_timer;

                        let r_off_x = level.side_offsets.down.expect("should have an exit anchor");
                        let off_x = player_pos.0 - TILE_SIZE / 2 - r_off_x;
                        let new_y = player_pos.1 - TILE_SIZE / 2 - level.dimensions().1 * TILE_SIZE;

                        let level_raw =
                            levelset.levels[level.side_exits.down.expect("is some")].clone();

                        current_ind = level.side_exits.down.expect("is some");
                        level = levels::Level::from_level_raw(level_raw);
                        let new_off_x = level.side_offsets.up.expect("should have an exit anchor");

                        render_off_y += (d.1 * TILE_PIXELS) as f32;
                        render_off_x -= ((new_off_x - r_off_x) / PIXEL_SIZE) as f32;

                        // paused = true;
                        let p = level.player_obj();

                        (p.freeze_timer, p.wall_sliding) = (old_freeze, old_sliding);
                        p.y = new_y;
                        p.x = new_off_x + off_x;
                        (p.vx, p.vy) = player_vel
                    } else {
                        let level_raw = levelset.levels[current_ind].clone();
                        level = levels::Level::from_level_raw(level_raw);
                    }
                } else if *keys_pressed.entry(KeyCode::Up).or_insert(false) {
                    let p_obj = level.player_obj();
                    let grounded = p_obj.grounded;
                    let aabb = (p_obj as &mut dyn Object).get_aabb();

                    let doors = levels::check_door(aabb, &level.tiles);

                    if let Some(levels::Tile::Door(index)) = doors {
                        if grounded {
                            println!("we should be going to {}", index);

                            let level_raw = levelset.levels[index].clone();

                            let old_ind = current_ind;
                            current_ind = index;
                            level = levels::Level::from_level_raw(level_raw);

                            let p_pos = levels::find_door(old_ind, &level.tiles);
                            if let Some((x, y)) = p_pos {
                                let p_obj = level.player_obj();

                                (p_obj.x, p_obj.y) = (x * TILE_SIZE, y * TILE_SIZE);
                            }

                            let new_d = level.dimensions();

                            render_off_x = (SCREEN_WIDTH / 2 - new_d.0 * TILE_PIXELS / 2) as f32;
                            render_off_y = (SCREEN_HEIGHT / 2 - new_d.1 * TILE_PIXELS / 2) as f32;
                        }
                    }
                } else {
                    let p_obj = level.player_obj();
                    let aabb = (p_obj as &mut dyn Object).get_aabb();

                    if levels::check_tilemap_death(aabb, &level.tiles) {
                        let level_raw = levelset.levels[current_ind].clone();
                        level = levels::Level::from_level_raw(level_raw);
                    }
                }

                for (_, is_pressed) in keys_pressed.iter_mut() {
                    *is_pressed = false
                }
            }
            let d = level.dimensions();
            let t_r_o_x = if d.0 * TILE_PIXELS < SCREEN_WIDTH {
                (SCREEN_WIDTH / 2 - d.0 * TILE_PIXELS / 2) as f32
            } else {
                let p_pos = level.focus_position().0;
                let p_pos = p_pos + level.player_vel().0 * 5;
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
                let p_pos = p_pos + level.player_vel().1 * 5;
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

        // draw_rectangle(255., 191., 2., 2., BLUE);
        let d = level.dimensions();
        // draw the rest of the level as well!!
        levelset.levels[current_ind].propagate_draw(
            render_off_x as i32,
            render_off_y as i32,
            &levelset.levels,
            &mut vec![current_ind],
            true,
            &mut textures,
        );

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
        next_frame().await;
    }
}
