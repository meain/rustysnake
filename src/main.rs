#![feature(drain_filter)]

extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate rand;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::*;
use piston::input::*;
use piston::window::WindowSettings;

use rand::prelude::*;

use std::collections::LinkedList;

#[derive(Clone, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

struct Game {
    width: u32,
    height: u32,
    gl: GlGraphics,
    snake: Snake,
    food: Food,
}

impl Game {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 0.0];

        self.gl.draw(args.viewport(), |_c, gl| {
            clear(BLACK, gl);
        });

        self.width = args.width;
        self.height = args.height;

        self.snake.render(args, &mut self.gl);
        self.food.render(args, &mut self.gl);
    }

    fn update(&mut self, _args: &UpdateArgs) {
        let had_food = self.food.update(&self.snake.nodes);
        self.snake.update(had_food, self.width, self.height);
    }

    fn pressed(&mut self, btn: &Button) {
        let last_dir = self.snake.dir.clone();
        self.snake.dir = match btn {
            &Button::Keyboard(Key::Up) if last_dir != Direction::Down => Direction::Up,
            &Button::Keyboard(Key::Down) if last_dir != Direction::Up => Direction::Down,
            &Button::Keyboard(Key::Left) if last_dir != Direction::Right => Direction::Left,
            &Button::Keyboard(Key::Right) if last_dir != Direction::Left => Direction::Right,
            _ => last_dir,
        }
    }
}

struct Food {
    nodes: LinkedList<Node>,
}

impl Food {
    fn render(&mut self, args: &RenderArgs, gl: &mut GlGraphics) {
        use graphics::*;

        const FOOD_CLR: [f32; 4] = [1.0, 0.6, 1.0, 0.8];
        gl.draw(args.viewport(), |c, gl| {
            let transform = c.transform;
            for node in &self.nodes {
                let square = rectangle::square((node.x * 20) as f64, (node.y * 20) as f64, 20.0);
                rectangle(FOOD_CLR, square, transform, gl);
            }
        })
    }

    fn update(&mut self, snake_nodes: &LinkedList<Node>) -> bool {
        let mut had_food = false;
        for snake_node in snake_nodes {
            // self.nodes.drain_filter(|node| node == snake_node); // nightly only (drain_filter)
            self.nodes.drain_filter(|node| {
                if node == snake_node {
                    had_food = true;
                }
                node == snake_node
            });
        }
        let mut rng = thread_rng();
        if rng.gen_range(0, 30) == 3 {
            self.nodes.push_front(Node {
                x: rng.gen_range(0, 30),
                y: rng.gen_range(0, 20),
            })
        }
        had_food
    }
}

#[derive(Clone, PartialEq)]
struct Node {
    x: i32,
    y: i32,
}

struct Snake {
    dir: Direction,
    nodes: LinkedList<Node>,
}

impl Snake {
    fn render(&mut self, args: &RenderArgs, gl: &mut GlGraphics) {
        use graphics::*;

        const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
        gl.draw(args.viewport(), |c, gl| {
            let transform = c.transform;
            for node in &self.nodes {
                let square = rectangle::square((node.x * 20) as f64, (node.y * 20) as f64, 20.0);
                rectangle(WHITE, square, transform, gl);
            }
        })
    }

    fn update(&mut self, had_food: bool, width: u32, height: u32) {
        let mut new_node = Node { x: 0, y: 0 };
        if let Some(node) = &self.nodes.front() {
            new_node.x = node.x;
            new_node.y = node.y;
        }

        match self.dir {
            Direction::Right => new_node.x += 1,
            Direction::Left => new_node.x -= 1,
            Direction::Down => new_node.y += 1,
            Direction::Up => new_node.y -= 1,
        }

        self.nodes.push_front(new_node);
        if !had_food {
            self.nodes.pop_back();
        }

        self.check_border_collition(width, height);
    }

    fn check_border_collition(&mut self, width: u32, height: u32) {
        let mut should_reset = false;
        for node in &self.nodes {
            if node.x * 20 > width as i32
                || node.y * 20 > height as i32
                || node.x * 20 < 0
                || node.y * 20 < 0
            {
                should_reset = true;
                break;
            }
        }
        if should_reset {
            self.reset();
        }
    }

    fn reset(&mut self) {
        let mut nodes: LinkedList<Node> = LinkedList::new();
        nodes.push_front(Node { x: 0, y: 0 });
        self.nodes = nodes;
        self.dir = Direction::Right;
    }
}

fn main() {
    let opengl = OpenGL::V3_2;

    let mut window: Window = WindowSettings::new("rustysnake", [600, 400])
        .opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut nodes: LinkedList<Node> = LinkedList::new();
    nodes.push_front(Node { x: 0, y: 0 });
    let mut game = Game {
        gl: GlGraphics::new(opengl),
        width: 600,
        height: 400,
        snake: Snake {
            dir: Direction::Right,
            nodes: nodes,
        },
        food: Food {
            nodes: LinkedList::new(),
        },
    };

    let mut events = Events::new(EventSettings::new()).ups(10);
    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            game.render(&r);
        }

        if let Some(u) = e.update_args() {
            game.update(&u);
        }

        if let Some(b) = e.button_args() {
            if b.state == ButtonState::Press {
                game.pressed(&b.button)
            }
        }
    }
}
