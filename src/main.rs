mod save;

use std::time::{SystemTime, UNIX_EPOCH};

use macroquad::prelude::*;
use save::{load as load_save, save as save_to_disk, SaveData};

/// Logical game area (portrait), scaled to fit the window.
const GAME_W: f32 = 480.0;
const GAME_H: f32 = 800.0;

const BIRD_X: f32 = 120.0;
const BIRD_RADIUS: f32 = 22.0;
const GRAVITY: f32 = 1850.0;
const FLAP_IMPULSE: f32 = -520.0;
const MAX_FALL_SPEED: f32 = 900.0;
const PIPE_WIDTH: f32 = 70.0;
const PIPE_GAP: f32 = 155.0;
const PIPE_SPAWN_DISTANCE: f32 = 260.0;
const BASE_SCROLL: f32 = 210.0;
const GROUND_H: f32 = 100.0;
const CEILING_PAD: f32 = 4.0;

/// PNG backgrounds and `frame-1.png`…`frame-8.png` bird cycle (project root: `assets/`).
struct SpriteArt {
    bg: Texture2D,
    bird_frames: Vec<Texture2D>,
}

async fn try_load_art() -> Option<SpriteArt> {
    let bg = load_texture("assets/bg.png").await.ok()?;
    let mut bird_frames = Vec::with_capacity(8);
    for i in 1..=8 {
        let path = format!("assets/frame-{i}.png");
        bird_frames.push(load_texture(&path).await.ok()?);
    }
    Some(SpriteArt { bg, bird_frames })
}

#[derive(Clone)]
struct PipePair {
    x: f32,
    gap_center_y: f32,
    scored: bool,
}

impl PipePair {
    fn top_height(&self) -> f32 {
        (self.gap_center_y - PIPE_GAP * 0.5).max(PIPE_WIDTH)
    }

    fn bottom_top(&self) -> f32 {
        self.gap_center_y + PIPE_GAP * 0.5
    }

    fn bottom_height(&self) -> f32 {
        GAME_H - GROUND_H - self.bottom_top()
    }
}

#[derive(PartialEq, Eq)]
enum Phase {
    Ready,
    Playing,
    GameOver,
}

struct Game {
    phase: Phase,
    bird_y: f32,
    bird_vel: f32,
    /// Drives wing flap cycle (radians advance per frame).
    wing_phase: f32,
    pipes: Vec<PipePair>,
    scroll: f32,
    score: u32,
    high_score: u32,
    save_data: SaveData,
    flap_cooldown: f32,
}

impl Game {
    fn new() -> Self {
        let save_data = load_save();
        let high_score = save_data.high_score;
        Self {
            phase: Phase::Ready,
            bird_y: GAME_H * 0.45,
            bird_vel: 0.0,
            wing_phase: 0.0,
            pipes: Vec::new(),
            scroll: BASE_SCROLL,
            score: 0,
            high_score,
            save_data,
            flap_cooldown: 0.0,
        }
    }

    fn reset_round(&mut self) {
        self.phase = Phase::Ready;
        self.bird_y = GAME_H * 0.45;
        self.bird_vel = 0.0;
        self.pipes.clear();
        self.scroll = BASE_SCROLL;
        self.score = 0;
        self.flap_cooldown = 0.0;
        self.wing_phase = 0.0;
    }

    fn start_playing(&mut self) {
        self.phase = Phase::Playing;
        self.bird_vel = FLAP_IMPULSE * 0.65;
        self.spawn_initial_pipes();
    }

    fn spawn_initial_pipes(&mut self) {
        let mut x = GAME_W + 80.0;
        while x < GAME_W + PIPE_SPAWN_DISTANCE * 4.0 {
            self.pipes.push(PipePair {
                x,
                gap_center_y: random_gap_y(),
                scored: false,
            });
            x += PIPE_SPAWN_DISTANCE;
        }
    }

    fn maybe_spawn_pipe(&mut self) {
        let last_x = self
            .pipes
            .last()
            .map(|p| p.x)
            .unwrap_or(GAME_W + PIPE_SPAWN_DISTANCE);
        if last_x < GAME_W + PIPE_SPAWN_DISTANCE {
            self.pipes.push(PipePair {
                x: last_x + PIPE_SPAWN_DISTANCE,
                gap_center_y: random_gap_y(),
                scored: false,
            });
        }
    }

    fn update_difficulty(&mut self) {
        let t = (self.score as f32 * 0.06).min(1.0);
        self.scroll = BASE_SCROLL + 85.0 * t;
    }

    fn circle_hits_rect(cx: f32, cy: f32, r: f32, rx: f32, ry: f32, rw: f32, rh: f32) -> bool {
        let nx = cx.clamp(rx, rx + rw);
        let ny = cy.clamp(ry, ry + rh);
        let dx = cx - nx;
        let dy = cy - ny;
        dx * dx + dy * dy < r * r
    }

    fn check_collision(&self, bird_y: f32) -> bool {
        if bird_y - BIRD_RADIUS < CEILING_PAD || bird_y + BIRD_RADIUS > GAME_H - GROUND_H {
            return true;
        }
        for p in &self.pipes {
            let top_h = p.top_height();
            if Self::circle_hits_rect(BIRD_X, bird_y, BIRD_RADIUS, p.x, 0.0, PIPE_WIDTH, top_h) {
                return true;
            }
            let bt = p.bottom_top();
            let bh = p.bottom_height();
            if Self::circle_hits_rect(BIRD_X, bird_y, BIRD_RADIUS, p.x, bt, PIPE_WIDTH, bh) {
                return true;
            }
        }
        false
    }

    fn update_playing(&mut self, dt: f32) {
        self.update_difficulty();
        self.bird_vel = (self.bird_vel + GRAVITY * dt).min(MAX_FALL_SPEED);
        self.bird_y += self.bird_vel * dt;

        for p in &mut self.pipes {
            p.x -= self.scroll * dt;
        }
        self.pipes.retain(|p| p.x + PIPE_WIDTH > -40.0);
        self.maybe_spawn_pipe();

        for p in &mut self.pipes {
            if !p.scored && p.x + PIPE_WIDTH < BIRD_X {
                p.scored = true;
                self.score += 1;
            }
        }

        if self.check_collision(self.bird_y) {
            self.phase = Phase::GameOver;
            if self.score > self.high_score {
                self.high_score = self.score;
                self.save_data.high_score = self.high_score;
                let _ = save_to_disk(&self.save_data);
            }
        }
    }

    fn flap(&mut self) {
        if self.flap_cooldown > 0.0 {
            return;
        }
        self.flap_cooldown = 0.08;
        match self.phase {
            Phase::Ready => self.start_playing(),
            Phase::Playing => self.bird_vel = FLAP_IMPULSE,
            Phase::GameOver => {
                self.reset_round();
                self.start_playing();
            }
        }
    }
}

fn random_gap_y() -> f32 {
    let min_c = 160.0 + PIPE_GAP * 0.5;
    let max_c = GAME_H - GROUND_H - 160.0 - PIPE_GAP * 0.5;
    rand::gen_range(min_c, max_c)
}

fn game_to_screen_scale() -> Vec2 {
    let sw = screen_width();
    let sh = screen_height();
    let sx = sw / GAME_W;
    let sy = sh / GAME_H;
    let s = sx.min(sy);
    vec2(s, s)
}

fn game_origin_screen() -> Vec2 {
    let scale = game_to_screen_scale();
    let sw = screen_width();
    let sh = screen_height();
    let gw = GAME_W * scale.x;
    let gh = GAME_H * scale.y;
    vec2((sw - gw) * 0.5, (sh - gh) * 0.5)
}

/// Flappy Bird–style sprite using primitives; collision stays a circle at `BIRD_X` / `bird_y`.
fn draw_flappy_bird(cx: f32, cy: f32, s: f32, game: &Game) {
    let body = Color::from_rgba(251, 216, 74, 255);
    let wing_dark = Color::from_rgba(235, 168, 40, 255);
    let wing_mid = Color::from_rgba(244, 188, 52, 255);
    let belly = Color::from_rgba(255, 236, 156, 255);
    let beak = Color::from_rgba(236, 165, 78, 255);
    let beak_tip = Color::from_rgba(225, 120, 55, 255);

    // Tilt nose-up when rising, nose-down when falling (like the original).
    let vel_tilt = (game.bird_vel / MAX_FALL_SPEED).clamp(-1.0, 1.0);
    let tilt = vel_tilt * 0.55;

    let flap_base = match game.phase {
        Phase::Ready => (get_time() as f32 * 8.0).sin(),
        Phase::Playing => game.wing_phase.sin(),
        Phase::GameOver => 0.0,
    };
    let flap = if game.phase == Phase::GameOver {
        0.0
    } else {
        flap_base * 0.85
    };

    // Back wing (behind body)
    draw_ellipse(
        cx - 10.0 * s,
        cy + 3.0 * s,
        24.0 * s,
        15.0 * s,
        tilt + flap * 0.9 + 0.35,
        wing_dark,
    );

    // Main body
    draw_ellipse(cx + 2.0 * s, cy - 1.0 * s, 38.0 * s, 28.0 * s, tilt + 0.08, body);

    // Lighter belly patch
    draw_ellipse(
        cx + 6.0 * s,
        cy + 7.0 * s,
        20.0 * s,
        12.0 * s,
        tilt + 0.1,
        belly,
    );

    // Front wing
    draw_ellipse(
        cx - 4.0 * s,
        cy + 5.0 * s,
        26.0 * s,
        16.0 * s,
        tilt - flap * 1.05 - 0.2,
        wing_mid,
    );

    // Outline on body (subtle)
    draw_ellipse_lines(
        cx + 2.0 * s,
        cy - 1.0 * s,
        38.0 * s,
        28.0 * s,
        tilt + 0.08,
        2.0,
        Color::from_rgba(200, 140, 30, 180),
    );

    // Big cartoon eye (white + pupil + shine)
    let eye_x = cx + 14.0 * s;
    let eye_y = cy - 8.0 * s;
    draw_circle(eye_x, eye_y, 12.5 * s, WHITE);
    draw_circle_lines(eye_x, eye_y, 12.5 * s, 1.5, Color::from_rgba(40, 40, 40, 120));
    draw_circle(eye_x + 5.0 * s, eye_y + 1.0 * s, 5.0 * s, BLACK);
    draw_circle(eye_x + 7.0 * s, eye_y - 2.0 * s, 2.2 * s, WHITE);

    // Beak: two triangles for depth
    let snout = 22.0 * s;
    let p1 = vec2(cx + 22.0 * s, cy + 1.0 * s);
    let p2 = vec2(cx + 22.0 * s + snout, cy - 2.0 * s);
    let p3 = vec2(cx + 22.0 * s + snout * 0.92, cy + 9.0 * s);
    draw_triangle(p1, p2, p3, beak);
    draw_triangle(
        p1 + vec2(1.0 * s, 2.0 * s),
        p3 + vec2(-2.0 * s, 0.0 * s),
        vec2(p3.x - 4.0 * s, p3.y + 2.0 * s),
        beak_tip,
    );
}

fn bird_sprite_frame_index(game: &Game) -> usize {
    let t = match game.phase {
        Phase::Ready => get_time() as f32 * 14.0,
        Phase::Playing => game.wing_phase * 2.8,
        Phase::GameOver => 0.0,
    };
    (t as usize).rem_euclid(8)
}

/// Raster bird; `flip_x` if your sprites face the wrong pipe direction.
fn draw_bird_sprite(
    art: &SpriteArt,
    cx: f32,
    cy: f32,
    scale: f32,
    game: &Game,
    flip_x: bool,
) {
    let idx = bird_sprite_frame_index(game).min(art.bird_frames.len().saturating_sub(1));
    let tex = &art.bird_frames[idx];
    let vel_tilt = (game.bird_vel / MAX_FALL_SPEED).clamp(-1.0, 1.0);
    let tilt = vel_tilt * 0.45;
    let dest_w = BIRD_RADIUS * 2.8 * scale;
    let dest_h = dest_w * tex.height() / tex.width();
    let x = cx - dest_w / 2.0;
    let y = cy - dest_h / 2.0;
    draw_texture_ex(
        tex,
        x,
        y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(dest_w, dest_h)),
            rotation: tilt,
            flip_x,
            ..Default::default()
        },
    );
}

fn draw_game_scaled(game: &Game, art: Option<&SpriteArt>) {
    let scale = game_to_screen_scale();
    let origin = game_origin_screen();

    let gx = origin.x;
    let gy = origin.y;
    let s = scale.x;

    let play_w = GAME_W * s;
    let play_h = GAME_H * s;

    // Clip to the letterboxed play area so pipes/sprites cannot draw into side margins.
    let clip_x = gx.floor() as i32;
    let clip_y = gy.floor() as i32;
    let clip_w = ((gx + play_w).ceil() as i32 - clip_x).max(1);
    let clip_h = ((gy + play_h).ceil() as i32 - clip_y).max(1);
    unsafe {
        let mut gl = get_internal_gl();
        gl.flush();
        gl.quad_gl.scissor(Some((clip_x, clip_y, clip_w, clip_h)));
    }

    // Background: image or solid sky
    if let Some(a) = art {
        draw_texture_ex(
            &a.bg,
            gx,
            gy,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(play_w, play_h)),
                ..Default::default()
            },
        );
    } else {
        draw_rectangle(gx, gy, play_w, play_h, SKYBLUE);
    }

    // Pipes
    for p in &game.pipes {
        let px = gx + p.x * s;
        let top_h = p.top_height() * s;
        draw_rectangle(px, gy, PIPE_WIDTH * s, top_h, DARKGREEN);
        let bt = p.bottom_top();
        let bh = p.bottom_height() * s;
        draw_rectangle(px, gy + bt * s, PIPE_WIDTH * s, bh, DARKGREEN);
    }

    // Ground
    let ground_y = gy + (GAME_H - GROUND_H) * s;
    draw_rectangle(gx, ground_y, GAME_W * s, GROUND_H * s, Color::from_rgba(139, 90, 43, 255));
    draw_rectangle(gx, ground_y, GAME_W * s, 8.0 * s, Color::from_rgba(101, 67, 33, 255));

    // Bird (visual center matches physics circle at BIRD_X, bird_y)
    let bx = gx + BIRD_X * s;
    let by = gy + game.bird_y * s;
    if let Some(a) = art {
        // Set true if the PNG faces away from the pipes.
        draw_bird_sprite(a, bx, by, s, game, false);
    } else {
        draw_flappy_bird(bx, by, s, game);
    }

    // HUD — text in screen space above playfield
    let title_size = 26.0;
    let hint_size = 22.0;
    match game.phase {
        Phase::Ready => {
            draw_text(
                "FLAPPY RUST",
                gx + 40.0 * s,
                gy + 120.0 * s,
                title_size * s,
                BLACK,
            );
            draw_text(
                "SPACE / CLICK — FLAP & START",
                gx + 25.0 * s,
                gy + 200.0 * s,
                hint_size * s,
                DARKGRAY,
            );
        }
        Phase::Playing => {
            let score_txt = format!("{}", game.score);
            let sz = 48.0 * s;
            let tw = measure_text(&score_txt, None, sz as u16, 1.0).width;
            draw_text(&score_txt, gx + (GAME_W * s - tw) * 0.5, gy + 80.0 * s, sz, BLACK);
        }
        Phase::GameOver => {
            draw_text(
                "GAME OVER",
                gx + 80.0 * s,
                gy + 280.0 * s,
                44.0 * s,
                RED,
            );
            let line1 = format!("SCORE: {}", game.score);
            draw_text(&line1, gx + 100.0 * s, gy + 350.0 * s, 28.0 * s, BLACK);
            let line2 = format!("BEST: {}", game.high_score);
            draw_text(&line2, gx + 100.0 * s, gy + 395.0 * s, 26.0 * s, DARKGRAY);
            draw_text(
                "SPACE / CLICK — RETRY",
                gx + 45.0 * s,
                gy + 470.0 * s,
                hint_size * s,
                DARKGRAY,
            );
        }
    }

    unsafe {
        let mut gl = get_internal_gl();
        gl.flush();
        gl.quad_gl.scissor(None);
    }
}

fn flap_pressed() -> bool {
    is_key_pressed(KeyCode::Space) || is_mouse_button_pressed(MouseButton::Left)
}

#[macroquad::main(window_conf)]
async fn main() {
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(1);
    rand::srand(seed);

    let art = try_load_art().await;
    if art.is_none() {
        eprintln!("Note: missing assets/*.png — falling back to vector bird + solid sky. Run from the project root.");
    }

    let mut game = Game::new();

    loop {
        let dt = get_frame_time();
        game.flap_cooldown = (game.flap_cooldown - dt).max(0.0);

        if flap_pressed() {
            game.flap();
        }
        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        if game.phase == Phase::Playing {
            let flap_spd = 11.0 + game.score as f32 * 0.15;
            game.wing_phase += dt * flap_spd * std::f32::consts::TAU / 2.8;
            game.update_playing(dt);
        } else if game.phase == Phase::Ready {
            // Gentle idle bob + wing cycle
            game.wing_phase += dt * 7.0 * std::f32::consts::TAU / 3.5;
            game.bird_y = GAME_H * 0.45 + (get_time() as f32 * 3.0).sin() * 12.0;
        }

        clear_background(BLACK);
        draw_game_scaled(&game, art.as_ref());

        next_frame().await;
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Flappy Rust".to_string(),
        window_width: 480,
        window_height: 800,
        window_resizable: true,
        ..Default::default()
    }
}
