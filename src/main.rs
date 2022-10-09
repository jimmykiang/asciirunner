#![warn(clippy::pedantic)]

use bracket_lib::prelude::*;

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const FRAME_DURATION: f32 = 75.0;

const RUNNING_FRAMES: [&str; 4] = ["┤", "│", "├", "┴"];

struct Player {
    x: i32,
    y: i32,
    velocity: f32,
    frame: usize,
}

impl Player {
    fn new(x: i32, y: i32) -> Self {
        Player {
            x,
            y,
            velocity: 0.0,
            frame: 0,
        }
    }

    fn gravity_and_move(&mut self) {
        // Increment gravity
        if self.velocity < 2.0 {
            self.velocity += 0.2;
        }

        // Apply gravity
        self.y += &(self.velocity as i32);
        if self.y >= SCREEN_HEIGHT - 1 {
            self.y = SCREEN_HEIGHT - 1;
        }

        // Move the player
        self.x += 1;

        if self.y < SCREEN_HEIGHT - 1 {
            self.frame = 3;
        } else {
            self.frame += 1;
            self.frame %= 4;
        }
    }

    fn jump(&mut self) {
        if self.y == SCREEN_HEIGHT - 1 {
            self.velocity = -2.5;
        }
    }

    fn render(&mut self, ctx: &mut BTerm) {
        ctx.set(
            20,
            self.y,
            GREEN,
            BLACK,
            to_cp437(RUNNING_FRAMES[self.frame].parse().unwrap()),
        );
    }
}

struct Obstacle {
    x: i32,
    gap_y: i32,
    size: i32,
}

impl Obstacle {
    // fn new(x: i32, score: i32) -> Self {
    fn new(x: i32, _: i32) -> Self {
        let mut random = RandomNumberGenerator::new();
        Obstacle {
            x,
            gap_y: random.range(39, 41),
            size: random.range(2, 10),
        }
    }

    fn render(&mut self, ctx: &mut BTerm, player_x: i32) {
        let screen_x = self.x - player_x;
        // let half_size = self.size / 2;
        let half_size = self.size;

        // Draw the bottom half of the obstacle
        for y in self.gap_y + half_size..SCREEN_HEIGHT {
            ctx.set(screen_x, y, GREEN, BLACK, to_cp437('|'));
        }
    }

    fn hit_obstacle(&self, player: &Player) -> bool {
        // let half_size = self.size / 2;
        let half_size = self.size;
        let does_x_match = player.x + 20 == self.x; // (1)
        let player_above_gap = player.y < self.gap_y - half_size; // (2)
        let player_below_gap = player.y > self.gap_y + half_size;
        does_x_match && (player_above_gap || player_below_gap) // (3)
    }
}

enum GameMode {
    Menu,
    Playing,
    End,
}

struct State {
    player: Player,
    frame_time: f32,
    obstacle: Obstacle,
    mode: GameMode,
    score: i32,
}

impl State {
    fn new() -> Self {
        State {
            player: Player::new(5, 25),
            frame_time: 0.0,
            obstacle: Obstacle::new(SCREEN_WIDTH, 0),
            mode: GameMode::Menu,
            score: 0,
        }
    }

    fn restart(&mut self) {
        self.player = Player::new(5, SCREEN_HEIGHT - 10);
        self.frame_time = 0.0;
        self.obstacle = Obstacle::new(SCREEN_WIDTH, 0);
        self.mode = GameMode::Playing;
        self.score = 0;
    }

    fn main_menu(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_color_centered(5, GREEN,BLACK,"Welcome to ASCII Runner!",);
        ctx.print_color_centered(7, GREEN,BLACK, "Made with ASCII graphics only - And inspired by The Dinosaur Game!");
        ctx.print_color_centered(14, GREEN,BLACK, "(P) Play Game");
        // No quitting in the wasm version.
        // ctx.print_centered(15, "(Q) Quit Game");

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),
                // No quitting in the wasm version.
                // VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }

    fn dead(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_color_centered(5,GREEN,BLACK, "Game Over!");
        ctx.print_color_centered(8, GREEN,BLACK,&format!("Score: {}", self.score));
        ctx.print_color_centered(15,GREEN,BLACK, "(P) Play Again");

        // No quitting in the wasm version.
        // ctx.print_centered(16, "(Q) Quit Game");

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),

                // No quitting in the wasm version.
                // VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }

    fn play(&mut self, ctx: &mut BTerm) {
        ctx.cls_bg(BLACK);
        self.frame_time += ctx.frame_time_ms;
        if self.frame_time > FRAME_DURATION {
            self.frame_time = 0.0;

            self.player.gravity_and_move();
        }
        if let Some(VirtualKeyCode::Space) = ctx.key {
            self.player.jump();
        }
        self.player.render(ctx);
        ctx.print_color(10, 15, GREEN,BLACK,"Press SPACE to jump!.");
        ctx.print_color(10, 16, GREEN,BLACK,&format!("Score: {}", self.score)); // (4)

        self.obstacle.render(ctx, self.player.x); // (5)
        if self.player.x > self.obstacle.x {
            // (6)
            self.score += 1;
            self.obstacle = Obstacle::new(self.player.x + SCREEN_WIDTH, self.score);
        }

        if self.obstacle.hit_obstacle(&self.player) {
            self.mode = GameMode::End;
        }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        match self.mode {
            GameMode::Menu => self.main_menu(ctx),
            GameMode::End => self.dead(ctx),
            GameMode::Playing => self.play(ctx),
        }
    }
}

fn main() -> BError {
    let context = BTermBuilder::simple80x50()
        .with_title("ASCII Runner")
        .build()?;

    main_loop(context, State::new())
}
