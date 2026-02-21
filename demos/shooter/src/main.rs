mod animation;
mod tilemap;

use rand::Rng;

use egor::{
    app::{App, FrameContext, WindowEvent, egui::Window},
    input::{KeyCode, MouseButton},
    math::{Rect, Vec2, vec2},
    render::{Align, Color, OffscreenTarget},
};
use egor::app::egui::epaint::Vertex;
use egor::app::egui::Pos2;
use egor::render::PathStep;
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

            gfx.camera().center(state.player.rect.position, screen_size);
            gfx.clear(Color::WHITE);



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





            gfx.path()
                .thickness(4.0)
                .stroke_color(Color::BLACK)
                // .fill_color(Color::BLUE)
                .steps(&[
                    PathStep::LineTo(vec2(100.0, 0.0)),
                    PathStep::QuadBezierTo(vec2(200.0, 0.0), vec2(200.0, 100.0)),
                    PathStep::CubicBezierTo(vec2(100.0, 100.0), vec2(0.0, 100.0), vec2(0.0, 0.0)),
                ]);





                // -------------------------
                // 1️⃣ Simple stroked shape
                // -------------------------
                gfx.path()
                    .at(vec2(0.0, 0.0)) // start here
                    .thickness(4.0)
                    .stroke_color(Color::BLACK)
                    .steps(&[
                        PathStep::LineTo(vec2(100.0, 0.0)),
                        PathStep::QuadBezierTo(vec2(200.0, 0.0), vec2(200.0, 100.0)),
                        PathStep::CubicBezierTo(vec2(100.0, 100.0), vec2(0.0, 100.0), vec2(0.0, 0.0)),
                    ]);

                // -------------------------
                // 2️⃣ Closed filled shape
                // -------------------------
                gfx.path()
                    .at(vec2(0.0, 0.0))
                    .thickness(3.0)
                    .stroke_color(Color::BLACK)
                    .fill_color(Color::BLUE)
                    .steps(&[
                        PathStep::LineTo(vec2(50.0, 0.0)),
                        PathStep::LineTo(vec2(50.0, 50.0)),
                        PathStep::LineTo(vec2(0.0, 50.0)),
                        PathStep::LineTo(vec2(0.0, 0.0)), // closes the square
                    ]);

                // -------------------------
                // 3️⃣ Looped cubic/quad curve
                // -------------------------
            let radius = 100.0;
            let center = vec2(200.0, 200.0); // arbitrary position

            // magic constant for approximating a circle with cubic Beziers
            let kappa = 0.552284749831; // ~4*(√2-1)/3

            gfx.path()
                .at(center + vec2(0.0, -radius)) // top of circle
                .thickness(4.0)
                .stroke_color(Color::BLACK)
                .fill_color(Color::BLUE)
                .steps(&[
                    // top-right
                    PathStep::CubicBezierTo(
                        center + vec2(radius * kappa, -radius),
                        center + vec2(radius, -radius * kappa),
                        center + vec2(radius, 0.0),
                    ),
                    // bottom-right
                    PathStep::CubicBezierTo(
                        center + vec2(radius, radius * kappa),
                        center + vec2(radius * kappa, radius),
                        center + vec2(0.0, radius),
                    ),
                    // bottom-left
                    PathStep::CubicBezierTo(
                        center + vec2(-radius * kappa, radius),
                        center + vec2(-radius, radius * kappa),
                        center + vec2(-radius, 0.0),
                    ),
                    // top-left
                    PathStep::CubicBezierTo(
                        center + vec2(-radius, -radius * kappa),
                        center + vec2(-radius * kappa, -radius),
                        center + vec2(0.0, -radius),
                    ),
                ]);


            let center = vec2(400.0, 100.0); // arbitrary positi

            gfx.path()
                .at(center + vec2(0.0, -radius)) // top of circle
                .thickness(4.0)
                .stroke_color(Color::BLACK)
                .fill_color(Color::GREEN)
                .steps(&[
                    // top-right
                    PathStep::CubicBezierTo(
                        center + vec2(radius * kappa, -radius),
                        center + vec2(radius, -radius * kappa),
                        center + vec2(radius, 0.0),
                    ),
                    // bottom-right
                    PathStep::CubicBezierTo(
                        center + vec2(radius, radius * kappa),
                        center + vec2(radius * kappa, radius),
                        center + vec2(0.0, radius),
                    ),
                    // bottom-left
                    PathStep::CubicBezierTo(
                        center + vec2(-radius * kappa, radius),
                        center + vec2(-radius, radius * kappa),
                        center + vec2(-radius, 0.0),
                    ),
                    // top-left
                    PathStep::CubicBezierTo(
                        center + vec2(-radius, -radius * kappa),
                        center + vec2(-radius * kappa, -radius),
                        center + vec2(0.0, -radius),
                    ),
                ]);



                // -------------------------
                // 4️⃣ Thick stroked arc-ish shape
                // -------------------------
                gfx.path()
                    .at(vec2(300.0, 50.0))
                    .thickness(8.0)
                    .stroke_color(Color::GREEN)
                    .steps(&[
                        PathStep::QuadBezierTo(vec2(350.0, 0.0), vec2(400.0, 50.0)),
                        PathStep::QuadBezierTo(vec2(450.0, 100.0), vec2(400.0, 150.0)),
                        PathStep::QuadBezierTo(vec2(350.0, 200.0), vec2(300.0, 150.0)),
                    ]);

              //  -------------------------
               //  Flower petals
             //   -------------------------
                let petals = [
                    (vec2(500.0, 100.0), vec2(520.0, 50.0), vec2(580.0, 50.0), vec2(600.0, 100.0)),
                    (vec2(600.0, 100.0), vec2(580.0, 150.0), vec2(520.0, 150.0), vec2(500.0, 100.0)),
                ];

                for petal in petals {
                    gfx.path()
                        .at(petal.0)
                        .thickness(2.0)
                        .stroke_color(Color::GREEN)
                        .fill_color(Color::BLUE)
                        .steps(&[
                            PathStep::CubicBezierTo(petal.1, petal.2, petal.3),
                        ]);
                }

                // -------------------------
                // 6️⃣ Grid of small Beziers
                // -------------------------
                for i in 0..5 {
                    for j in 0..5 {
                        let x = 50.0 + i as f32 * 40.0;
                        let y = 300.0 + j as f32 * 40.0;
                        gfx.path()
                            .at(vec2(x, y))
                            .thickness(1.0)
                            .stroke_color(Color::GREEN)
                            .steps(&[
                                PathStep::CubicBezierTo(vec2(x + 10.0, y - 10.0), vec2(x + 30.0, y + 10.0), vec2(x + 40.0, y)),
                            ]);
                    }
                }




             gfx.polyline().points(&points).thickness(4.0).color(Color::RED);




        },
    );
}
