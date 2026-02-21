mod animation;
mod tilemap;

use rand::Rng;

use egor::{
    app::{App, FrameContext, WindowEvent, egui::Window},
    input::{KeyCode, MouseButton},
    math::{Rect, Vec2, vec2},
    render::{Align, Color, OffscreenTarget},
};
use egor::render::{PathStep, Shape};
use crate::{animation::SpriteAnim, tilemap::EgorMap};

const PLAYER_SIZE: f32 = 64.0;
const BULLET_SIZE: Vec2 = vec2(5.0, 10.0);

struct Bullet {
    rect: Rect,
    vel: Vec2,
}

struct Zombie {
    rect: Rect,
    speed: f32,
    hp: f32,
    flash: f32,
}

struct Soldier {
    rect: Rect,
    hp: f32,
    flash: f32,
}

struct GameState {
    map: EgorMap,
    minimap: Option<OffscreenTarget>,
    minimap_tex: usize,
    player: Soldier,
    player_anim: SpriteAnim,
    player_tex: usize,
    enemies: Vec<Zombie>,
    enemy_anim: SpriteAnim,
    enemy_tex: usize,
    bullets: Vec<Bullet>,
    wave: usize,
    kills: usize,
    hp: f32,
    fire_cd: f32,
    fire_rate: f32,
    spread: usize,
    game_over: bool,
}

fn spawn_wave(position: Vec2, count: usize, speed: (f32, f32), hp: f32) -> Vec<Zombie> {
    let mut rng = rand::thread_rng();
    (0..count)
        .map(|_| {
            let a = rng.gen_range(0.0..std::f32::consts::TAU);
            let d = rng.gen_range(300.0..800.0);
            let pos = position + vec2(a.cos(), a.sin()) * d;
            Zombie {
                rect: Rect::new(pos, Vec2::splat(PLAYER_SIZE)),
                speed: rng.gen_range(speed.0..speed.1),
                hp,
                flash: 0.0,
            }
        })
        .collect()
}

fn spawn_bullets(position: Vec2, target: Vec2, count: usize) -> Vec<Bullet> {
    let angle = (target - position).y.atan2((target - position).x);
    let spread = 0.3;
    let half = (count as f32 - 1.0) / 2.0;

    (0..count)
        .map(|i| {
            let offset = (i as f32 - half) * spread / half.max(1.0);
            let a = angle + offset;
            Bullet {
                rect: Rect::new(position - BULLET_SIZE / 2.0, BULLET_SIZE),
                vel: vec2(a.cos(), a.sin()) * 500.0,
            }
        })
        .collect()
}

fn handle_bullet_hits(bullets: &mut Vec<Bullet>, enemies: &mut Vec<Zombie>, player: Vec2) -> usize {
    let mut kills = 0;
    bullets.retain(|b| {
        for e in enemies.iter_mut() {
            if e.rect.contains(b.rect.position) {
                e.hp -= 1.0;
                e.flash = 0.1;
                return false;
            }
        }
        let offscreen = (b.rect.position - player).length() > 2000.0;
        !offscreen
    });

    enemies.retain(|e| {
        if e.hp <= 0.0 {
            kills += 1;
            false
        } else {
            true
        }
    });

    kills
}

fn main() {
    let mut state = GameState {
        map: EgorMap::new(include_str!("../assets/map.json")),
        minimap: None,
        minimap_tex: 0,
        player: Soldier {
            rect: Rect::new(Vec2::ZERO, Vec2::splat(PLAYER_SIZE)),
            hp: 100.0,
            flash: 0.0,
        },
        player_anim: SpriteAnim::new(3, 6, 16, 0.2),
        player_tex: 0,
        enemies: spawn_wave(Vec2::ZERO, 5, (50.0, 125.0), 1.0),
        enemy_anim: SpriteAnim::new(2, 6, 11, 0.2),
        enemy_tex: 0,
        bullets: vec![],
        wave: 1,
        kills: 0,
        hp: 1.0,
        fire_cd: 0.0,
        fire_rate: 2.0,
        spread: 1,
        game_over: false,
    };

    App::new().title("Egor Shooter Demo").run(
        move |FrameContext {
                  gfx,
                  input,
                  timer,
                  egui_ctx,
                  events,
                  ..
              }| {

            if timer.frame == 0 {
                state.map.load_tileset(
                    gfx,
                    include_bytes!("../assets/otsp_tiles_01.png"),
                    "otsp_tiles_01.png",
                );
                state.map.load_tileset(
                    gfx,
                    include_bytes!("../assets/otsp_walls_01.png"),
                    "otsp_walls_01.png",
                );
                return;
            }

            let screen_size = gfx.screen_size();

          //  gfx.camera().center(state.player.rect.position, screen_size);
            gfx.clear(Color::BLACK);



            for e in &mut state.enemies {
                let dir = (state.player.rect.position - e.rect.position).normalize_or_zero();
                e.rect.translate(dir * e.speed * timer.delta);
            }





            let mut points = Vec::new();

            let radius = 100.0;
            let segment_count = 12;

            for i in 0..=segment_count {
                let theta = (i as f32 / segment_count as f32) * std::f32::consts::TAU;

                let x = radius * theta.cos();
                let y = radius * theta.sin();

                points.push(Vec2::new(x, y));
            }



            // let mut values = Vec::new();
            //
            // let radius = 100.0;
            // let segment_count = 6;
            // for i in 0..=segment_count {
            //     values.push(std::f32::consts::TAU * (i as f32 / segment_count as f32));
            // }
            //
            //     let mut points = Vec::new();
            //
            //
            //
            //
            //     for i in 0..segment_count {
            //
            //         points.push(Vec2::new(radius * values[i].sin(), radius * values[i].cos()));
            //     }





            // gfx.shape()
            //     .at(vec2(300.0, 0.0))
            //     .thickness(4.0)
            //     .stroke_color(Color::new([1.0, 0.25, 0.0, 1.0]))
            //     .fill_color(Color::BLUE)
            //     .shape(Shape::Path(
            //         vec![
            //             PathStep::Begin(vec2(0.0, 0.0)),
            //             PathStep::LineTo(vec2(100.0, 0.0)),
            //             PathStep::QuadBezierTo(vec2(200.0, 0.0), vec2(200.0, 100.0)),
            //             PathStep::CubicBezierTo(vec2(100.0, 100.0), vec2(0.0, 100.0), vec2(0.0, 0.0)),
            //          ]
            //     ));
            //
            //
            //
            // gfx.shape()
            //     .thickness(4.0)
            //     .stroke_color(Color::WHITE)
            //     .fill_color(Color::RED)
            //     .shape(Shape::Rect(vec2(200.0, 300.0)));
            //







                let center = vec2(400.0, 300.0); // position of sunflower on screen
                let center_radius = 40.0;
                let petal_length = 80.0;
                let petal_width = 30.0;
                let petal_count = 16;

                // -------------------------
                // 1️⃣ Flower center
                // -------------------------
                // gfx.shape()
                //     .at(center)
                //     .thickness(2.0)
                //     .stroke_color(Color::BLACK)
                //     .fill_color(Color::new([1.0, 0.6, 0.0, 1.0])) // orange
                //     .shape(Shape::Circle(center_radius));

                // -------------------------
                // 2️⃣ Petals
                // -------------------------
                for i in 0..petal_count {
                    let angle = (i as f32) / (petal_count as f32) * std::f32::consts::TAU;
                    let sin = angle.sin();
                    let cos = angle.cos();

                    // Define the petal shape relative to the center
                    let tip = vec2(cos * petal_length, sin * petal_length);
                    let control1 = vec2(cos * (petal_length * 0.3) - sin * (petal_width * 0.5),
                                        sin * (petal_length * 0.3) + cos * (petal_width * 0.5));
                    let control2 = vec2(cos * (petal_length * 0.7) - sin * (petal_width * 0.5),
                                        sin * (petal_length * 0.7) + cos * (petal_width * 0.5));

                    gfx.shape()
                        .at(center)
                        .thickness(2.0)
                        .stroke_color(Color::BLACK)
                        .fill_color(Color::new([0.8, 0.8, 0.1, 1.0]))
                        .shape(Shape::Path(vec![
                            PathStep::Begin(vec2(0.0, 0.0)),        // start at flower center
                            PathStep::CubicBezierTo(control1, control2, tip),
                            PathStep::CubicBezierTo(control2 * -1.0, control1 * -1.0, vec2(0.0, 0.0)), // back to center

                        ]));
                }

                // -------------------------
                // 3️⃣ Stem
                // -------------------------
                gfx.shape()
                    .at(center)
                    .thickness(10.0)
                    .stroke_color(Color::new([0.0, 0.5, 0.0, 1.0]))
                    .fill_color(Color::new([0.0, 0.5, 0.0, 1.0]))
                    .shape(Shape::Path(vec![
                        PathStep::Begin(vec2(0.0, center_radius)),
                        PathStep::LineTo(vec2(0.0, center_radius + 150.0)),
                    ]));

                // -------------------------
                // 4️⃣ Optional leaves
                // -------------------------
                let leaf_offsets = [vec2(-20.0, 80.0), vec2(20.0, 120.0)];
                for leaf in leaf_offsets {
                    gfx.shape()
                        .at(center)
                        .thickness(2.0)
                        .stroke_color(Color::new([0.0, 0.4, 0.0, 1.0]))
                        .fill_color(Color::new([0.0, 0.8, 0.0, 1.0]))
                        .shape(Shape::Path(vec![
                            PathStep::Begin(leaf),
                            PathStep::QuadBezierTo(leaf + vec2(40.0, 20.0), leaf + vec2(0.0, 40.0)),
                            PathStep::QuadBezierTo(leaf + vec2(-40.0, 20.0), leaf),

                        ]));
                }
















         //    gfx.polyline().points(&points).thickness(4.0).color(Color::RED);




        },
    );
}
