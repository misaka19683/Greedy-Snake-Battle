use piston_window::*;
use rand::Rng;

const GRID_SIZE: i32 = 20;
const WINDOW_SIZE: i32 = 30;

#[derive(Clone, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, PartialEq)]
enum FoodType {
    Normal,
    Special,
}

struct Snake {
    body: Vec<(i32, i32)>,
    direction: Direction,
    next_direction: Direction,
}

struct Game {
    snake: Snake,
    food: (i32, i32),
    food_type: FoodType,
    game_over: bool,
    base_speed:u64,
    is_accelerating: bool,
}

impl Game {
    fn new() -> Self {
        let mut rng = rand::rng();
        let origin_position: Vec<(i32, i32)> = vec![(5, 5), (4, 5), (3, 5)];
        let (food, food_type) = generate_food(&mut rng, &origin_position); // 调用辅助函数生成食物
        Game {
            snake: Snake {
                body: origin_position,
                direction: Direction::Right,
                next_direction: Direction::Right,
            },
            food_type,
            food,
            game_over: false,
            base_speed: 150,
            is_accelerating: false,
        }
    }

    fn update(&mut self) {
        if self.game_over {
            return;
        }

        self.snake.direction = self.snake.next_direction.clone();

        let head = self.snake.body[0];
        let mut new_head = match self.snake.direction {
            Direction::Up => (head.0, head.1 - 1),
            Direction::Down => (head.0, head.1 + 1),
            Direction::Left => (head.0 - 1, head.1),
            Direction::Right => (head.0 + 1, head.1),
        };

        // 实现穿墙逻辑
        new_head.0 = (new_head.0 + WINDOW_SIZE) % WINDOW_SIZE;
        new_head.1 = (new_head.1 + WINDOW_SIZE) % WINDOW_SIZE;

        // 碰撞检测（仅保留身体碰撞检测）
        if self.snake.body.contains(&new_head) {
            self.game_over = true;
            return;
        }

        // 移动身体
        self.snake.body.insert(0, new_head);

        // 吃食物判断
        if new_head == self.food {
            let mut rng = rand::rng();
            let (new_food, new_food_type) = generate_food(&mut rng, &self.snake.body); // 生成新的食物
            self.food = new_food;


            // 根据食物类型调整蛇的长度
            match self.food_type {
                FoodType::Normal => {} // 普通食物不改变长度
                FoodType::Special => {
                    // 特殊食物让蛇变长四格
                    for _ in 0..4 {
                        let tail = *self.snake.body.last().unwrap();
                        self.snake.body.push(tail);
                    }
                }
            }
            self.food_type = new_food_type;
        } else {
            self.snake.body.pop();
        }
    }

    fn handle_input(&mut self, key: Key, pressed:bool) {
        if self.game_over {
            // 如果游戏结束，仅允许通过 R 键重启游戏
            if key == Key::R {
                *self = Game::new() // 重置游戏状态
            }
            return;
        }
        
        // 处理空格键加速
        if key == Key::Space {
            self.is_accelerating = pressed;
        }
        
        self.snake.next_direction = match key {
            Key::Up if self.snake.direction != Direction::Down => Direction::Up,
            Key::Down if self.snake.direction != Direction::Up => Direction::Down,
            Key::Left if self.snake.direction != Direction::Right => Direction::Left,
            Key::Right if self.snake.direction != Direction::Left => Direction::Right,
            _ => return,
        };
    }
}

// 辅助函数：生成食物及其类型
fn generate_food(rng: &mut impl Rng, snake_body: &[(i32, i32)]) -> ((i32, i32), FoodType) {
    loop {
        let food = (
            rng.random_range(1..WINDOW_SIZE - 1),
            rng.random_range(1..WINDOW_SIZE - 1),
        );
        if !snake_body.contains(&food) {
            let food_type = if rng.random_range(0..10) < 8 {
                FoodType::Normal // 80% 概率生成普通食物
            } else {
                FoodType::Special // 20% 概率生成特殊食物
            };
            return (food, food_type);
        }
    }
}

fn main() {
    let mut window: PistonWindow = WindowSettings::new(
        "Snake-RS",
        [WINDOW_SIZE as u32 * GRID_SIZE as u32, WINDOW_SIZE as u32 * GRID_SIZE as u32],
    )
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut game = Game::new();
    let mut last_update = std::time::Instant::now();

    while let Some(e) = window.next() {
        // 处理按键事件（包含按住状态）
        if let Some(ButtonArgs { button: Button::Keyboard(key), state, .. }) = e.button_args() {
            game.handle_input(key, state == ButtonState::Press);
        }

        let current_speed = if game.is_accelerating {
            game.base_speed / 2  // 加速时速度加倍
        } else { game.base_speed };
        // println!("{}",current_speed);
        //更新游戏状态
        if last_update.elapsed() >= std::time::Duration::from_millis(current_speed) {
            game.update();
            last_update = std::time::Instant::now();
        }

        //渲染画面
        window.draw_2d(&e, |c, g, _| {
            clear([0.0; 4], g);

            for (x, y) in &game.snake.body {
                let color = if game.game_over {
                    [1.0, 0.0, 0.0, 1.0]
                } else {
                    [0.0, 1.0, 0.0, 1.0]
                };
                rectangle(
                    color,
                    [
                        (*x as f64) * GRID_SIZE as f64,
                        (*y as f64) * GRID_SIZE as f64,
                        GRID_SIZE as f64 - 1.0,
                        GRID_SIZE as f64 - 1.0,
                    ],
                    c.transform,
                    g,
                );
            }
            let food_color = match game.food_type {
                FoodType::Normal => [1.0, 0.0, 0.0, 1.0],
                FoodType::Special => [0.0, 0.0, 1.0, 1.0],
            };
            rectangle(
                food_color,
                [
                    game.food.0 as f64 * GRID_SIZE as f64,
                    game.food.1 as f64 * GRID_SIZE as f64,
                    GRID_SIZE as f64 - 1.0,
                    GRID_SIZE as f64 - 1.0,
                ],
                c.transform,
                g,
            );
        });
    }
}