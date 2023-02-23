use crossterm::{
    cursor, queue,
    style::{self, Stylize},
    terminal,
};
use std::{collections::HashSet, io::Write, vec};

trait CollisionDetector {
    fn has_collision(&self, point: &(u16, u16)) -> bool;
}

struct SnakeState {
    vec: Vec<(u16, u16)>,
    hash: HashSet<(u16, u16)>,
}

impl CollisionDetector for SnakeState {
    fn has_collision(&self, point: &(u16, u16)) -> bool {
        self.hash.contains(point)
    }
}

impl SnakeState {
    fn head(&self) -> Option<&(u16, u16)> {
        self.vec.first()
    }

    fn push(&mut self, point: &(u16, u16)) {
        self.vec.insert(0, point.clone());
        self.hash.insert(point.clone());
    }

    fn pop(&mut self) {
        self.vec.pop().and_then(|p| Some(self.hash.remove(&p)));
    }

    fn vectorized(&self) -> Vec<(u16, u16, style::StyledContent<char>)> {
        self.vec
            .clone()
            .into_iter()
            .map(|point| {
                (
                    point.0,
                    point.1,
                    'ó'.with(style::Color::Cyan)
                        .attribute(style::Attribute::Bold),
                )
            })
            .collect()
    }
}

struct GameArea {
    from: (u16, u16),
    to: (u16, u16),
}

impl CollisionDetector for GameArea {
    fn has_collision(&self, point: &(u16, u16)) -> bool {
        point.0 < self.from.0 || point.0 > self.to.0 || point.1 < self.from.1 || point.1 > self.to.1
    }
}

impl GameArea {
    fn vectorized(&self) -> Vec<(u16, u16, style::StyledContent<char>)> {
        let (x0, y0) = self.from;
        let (xn, yn) = self.to;
        let block_char = '▒'.with(style::Color::Red);

        let v = (x0..(xn + 1))
            .map(|x| [(x, y0, block_char), (x, yn, block_char)])
            .chain((y0..(yn + 1)).map(|y| [(x0, y, block_char), (xn, y, block_char)]))
            .flatten()
            .collect();
        v
    }
}

struct FoodState {
    vec: Vec<(u16, u16)>,
    hash: HashSet<(u16, u16)>,
}

impl CollisionDetector for FoodState {
    fn has_collision(&self, point: &(u16, u16)) -> bool {
        self.hash.contains(point)
    }
}

impl FoodState {
    fn pop(&mut self) -> (u16, u16) {
        self.vec
            .pop()
            .and_then(|p| {
                if self.hash.contains(&p) {
                    self.hash.remove(&p);
                }
                Some(p)
            })
            .unwrap_or((0, 0))
    }

    fn push(&mut self, point: (u16, u16)) {
        self.vec.push(point);
        self.hash.insert(point);
    }

    fn vectorized(&self) -> Vec<(u16, u16, style::StyledContent<char>)> {
        self.vec
            .clone()
            .into_iter()
            .map(|point| {
                (
                    point.0,
                    point.1,
                    '$'.with(style::Color::Green)
                        .attribute(style::Attribute::Bold),
                )
            })
            .collect()
    }
}

struct ScreenPrinter {
    output: std::io::Stdout,
}

impl ScreenPrinter {
    fn new() -> Self {
        ScreenPrinter {
            output: std::io::stdout(),
        }
    }

    fn setup(&mut self) -> Result<(), std::io::Error> {
        terminal::enable_raw_mode()?;
        queue!(self.output, cursor::Hide)?;
        Ok(())
    }

    fn clear(&mut self) -> Result<(), std::io::Error> {
        terminal::disable_raw_mode()?;
        queue!(
            self.output,
            terminal::Clear(terminal::ClearType::All),
            cursor::Show,
            cursor::MoveTo(0, 0)
        )?;
        Ok(())
    }
}

fn get_next_point(point: &(u16, u16), direction: &Direction) -> (u16, u16) {
    match direction {
        Direction::RIGHT => (point.0.saturating_add(1), point.1),
        Direction::LEFT => (point.0.saturating_sub(1), point.1),
        Direction::DOWN => (point.0, point.1.saturating_add(1)),
        Direction::UP => (point.0 + 1, point.1.saturating_sub(1)),
    }
}

enum Direction {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

fn main() -> Result<(), std::io::Error> {
    let (cols, rows) = terminal::size()?;
    let mut screen = ScreenPrinter::new();
    let mut snake = SnakeState {
        vec: vec![(10, 10)],
        hash: HashSet::from([(10, 10)]),
    };
    let area = GameArea {
        from: (8, 8),
        to: (cols / 2, rows / 2),
    };
    let mut food = FoodState {
        vec: vec![(10, 15)],
        hash: HashSet::from([(10, 15)]),
    };
    let direction: Direction = Direction::DOWN;

    screen.setup()?;
    loop {
        queue!(screen.output, terminal::Clear(terminal::ClearType::All))?;

        let mut cursor_points = snake.vectorized();
        cursor_points.append(&mut area.vectorized());
        cursor_points.append(&mut food.vectorized());

        for ent in cursor_points {
            queue!(
                screen.output,
                cursor::MoveTo(ent.0, ent.1),
                style::PrintStyledContent(ent.2)
            )?;
        }

        let next_point = get_next_point(snake.head().unwrap_or(&area.from), &direction);
        if snake.has_collision(&next_point) {
            break;
        }
        if area.has_collision(&next_point) {
            break;
        }
        snake.push(&next_point);
        if food.has_collision(&next_point) {
            let p = food.pop();
            let mut point = (10, 20);
            if p.0 == 10 && p.1 == 20 {
                point = (20, 10);
            }
            food.push(point)
        } else {
            snake.pop();
        }

        queue!(screen.output, cursor::MoveTo(cols, rows))?;
        screen.output.flush()?;
        std::thread::sleep(std::time::Duration::from_millis(800));
    }
    screen.clear()?;
    Ok(())
}
