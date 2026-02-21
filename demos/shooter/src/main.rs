mod animation;
mod tilemap;

use rand::Rng;

use crate::{animation::SpriteAnim, tilemap::EgorMap};
use egor::render::{PathStep, Shape};
use egor::{
    app::{App, FrameContext, WindowEvent, egui::Window},
    input::{KeyCode, MouseButton},
    math::{Rect, Vec2, vec2},
    render::{Align, Color, OffscreenTarget},
};

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
    rotation: f32,
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
        rotation: 0.0,
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

            let speed = 5.8;
            state.rotation += speed * timer.delta;

            let position = vec2(550.0, 350.0);

            let blade_length = 120.0;
            let blade_width = 40.0;

            // BASE
            gfx.shape()
                .at(position + vec2(0.0, 200.0))
                .scale(vec2(1.5, 1.0))
                .thickness(3.0)
                .stroke_color(Color::BLACK)
                .fill_color(Color::new([0.2, 0.2, 0.2, 1.0]))
                .shape(Shape::Path {
                    steps: vec![
                        PathStep::Begin(vec2(-60.0, 0.0)),
                        PathStep::LineTo(vec2(60.0, 0.0)),
                        PathStep::LineTo(vec2(80.0, 40.0)),
                        PathStep::LineTo(vec2(-80.0, 40.0)),
                        PathStep::LineTo(vec2(-60.0, 0.0)),
                    ],
                });

            // STAND
            gfx.shape()
                .at(position)
                .thickness(12.0)
                .stroke_color(Color::new([0.3, 0.3, 0.3, 1.0]))
                .shape(Shape::Path {
                    steps: vec![
                        PathStep::Begin(vec2(0.0, 30.0)),
                        PathStep::LineTo(vec2(0.0, 200.0)),
                    ],
                });

            // BLADES
            for i in 0..4 {
                let base_angle = i as f32 * std::f32::consts::FRAC_PI_2;

                // Main blade body
                let k = 0.5522847498;
                let r = blade_width * 0.5;
                let tip_x = blade_length;

                gfx.shape()
                    .at(position)
                    .rotate(state.rotation + base_angle)
                    .scale(vec2(1.2, 1.0))
                    .thickness(2.0)
                    .stroke_color(Color::BLACK)
                    .fill_color(Color::new([0.5, 0.3, 0.9, 1.0]))
                    .shape(Shape::Path {
                        steps: vec![
                            // Start bottom-left
                            PathStep::Begin(vec2(0.0, -r)),
                            // Straight bottom edge
                            PathStep::LineTo(vec2(tip_x - r, -r)),
                            // Rounded tip (bottom → top)
                            PathStep::CubicBezierTo(
                                vec2(tip_x - r + r * k, -r),
                                vec2(tip_x, -r + r * k),
                                vec2(tip_x, 0.0),
                            ),
                            PathStep::CubicBezierTo(
                                vec2(tip_x, r - r * k),
                                vec2(tip_x - r + r * k, r),
                                vec2(tip_x - r, r),
                            ),
                            // Straight top edge back to base
                            PathStep::LineTo(vec2(0.0, r)),
                            // Close manually
                            PathStep::LineTo(vec2(0.0, -r)),
                        ],
                    });
            }

            // CENTER HUB
            gfx.shape()
                .at(position)
                .scale(vec2(1.1, 1.1)) // small scale test
                .thickness(3.0)
                .stroke_color(Color::BLACK)
                .fill_color(Color::new([0.7, 0.7, 0.7, 1.0]))
                .shape(Shape::Circle {
                    center: vec2(0.0, 0.0),
                    radius: 30.0,
                });

            //    gfx.polyline().points(&points).thickness(4.0).color(Color::RED);
        },
    );
}
