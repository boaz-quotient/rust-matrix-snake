use crossterm::{
    cursor,
    event::{poll, read, Event, KeyCode},
    queue,
    style::{self, style, StyledContent, Stylize},
    terminal,
};
use rand::Rng;
use std::{collections::HashSet, io::Write, time::Duration, vec};

trait CollisionDetector {
    fn has_collision(&self, point: &(u16, u16)) -> bool;
}

#[derive(Clone, Debug)]
struct LookupPointQueue {
    vec: Vec<(u16, u16)>,
    hash: HashSet<(u16, u16)>,
}

impl LookupPointQueue {
    fn new(points: &Vec<(u16, u16)>) -> Self {
        let vec = points.clone();
        let mut hash = HashSet::new();
        for point in points {
            hash.insert(point.clone());
        }
        Self { vec, hash }
    }

    fn lookup(&self, point: &(u16, u16)) -> bool {
        self.hash.contains(point)
    }

    fn head(&self) -> Option<&(u16, u16)> {
        self.vec.first()
    }

    fn push(&mut self, point: &(u16, u16)) {
        self.vec.insert(0, point.clone());
        self.hash.insert(point.clone());
    }

    fn pop(&mut self) -> Option<(u16, u16)> {
        self.vec.pop().and_then(|p| {
            self.hash.remove(&p);
            Some(p)
        })
    }
}

impl Iterator for LookupPointQueue {
    type Item = (u16, u16);
    fn next(&mut self) -> Option<Self::Item> {
        self.vec.pop()
    }
}

struct SnakeState {
    lq: LookupPointQueue,
}

impl CollisionDetector for SnakeState {
    fn has_collision(&self, point: &(u16, u16)) -> bool {
        self.lq.lookup(point)
    }
}

#[derive(Debug, Clone)]
struct GameArea {
    from: (u16, u16),
    to: (u16, u16),
}

impl CollisionDetector for GameArea {
    fn has_collision(&self, point: &(u16, u16)) -> bool {
        point.0 <= self.from.0
            || point.0 >= self.to.0
            || point.1 <= self.from.1
            || point.1 >= self.to.1
    }
}

impl Into<Vec<(u16, u16)>> for GameArea {
    fn into(self) -> Vec<(u16, u16)> {
        let (x0, y0) = self.from;
        let (xn, yn) = self.to;

        (x0..(xn + 1))
            .map(|x| [(x, y0), (x, yn)])
            .chain((y0..(yn + 1)).map(|y| [(x0, y), (xn, y)]))
            .flatten()
            .collect::<Vec<(u16, u16)>>()
    }
}

struct FoodState {
    lq: LookupPointQueue,
}

impl CollisionDetector for FoodState {
    fn has_collision(&self, point: &(u16, u16)) -> bool {
        self.lq.lookup(point)
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
        Direction::UP => (point.0, point.1.saturating_sub(1)),
    }
}

enum Direction {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

fn main() -> Result<(), std::io::Error> {
    let mut rng = rand::thread_rng();
    let japanese_vec: Vec<char> = (65382..=65437)
        .collect::<Vec<u32>>()
        .iter()
        .map(|n| std::char::from_u32(*n).unwrap_or(' '))
        .collect();
    let (cols, rows) = terminal::size()?;
    let mut screen = ScreenPrinter::new();
    let area = GameArea {
        from: (cols / 4, rows / 4),
        to: (3 * cols / 4, 3 * rows / 4),
    };
    let area_vec: Vec<(u16, u16)> = area.clone().into();
    let mut snake = SnakeState {
        lq: LookupPointQueue::new(&vec![(area.from.0 + 1, area.from.1 + 1)]),
    };
    let initial_food_point = (
        rng.gen_range((area.from.0 + 1)..(area.to.0 - 1)),
        rng.gen_range((area.from.1 + 1)..(area.to.1 - 1)),
    );
    let mut food = FoodState {
        lq: LookupPointQueue::new(&vec![initial_food_point]),
    };
    let mut direction: Direction = Direction::DOWN;

    screen.setup()?;
    loop {
        queue!(screen.output, terminal::Clear(terminal::ClearType::All))?;

        let s = snake
            .lq
            .clone()
            .into_iter()
            .enumerate()
            .map(|(ix, point)| {
                let mut chr = japanese_vec[rng.gen_range(0..japanese_vec.len())]
                    .with(style::Color::DarkGreen)
                    .attribute(style::Attribute::Bold);
                if ix == snake.lq.vec.len() - 1 {
                    chr = chr.with(style::Color::White);
                }
                (point.0, point.1, chr)
            })
            .chain(
                area_vec
                    .clone()
                    .into_iter()
                    .map(|point| (point.0, point.1, ' '.on_magenta())),
            )
            .chain(food.lq.clone().into_iter().map(|point| {
                (
                    point.0,
                    point.1,
                    '$'.with(style::Color::White)
                        .attribute(style::Attribute::Bold),
                )
            }));

        for ent in s.collect::<Vec<(u16, u16, StyledContent<char>)>>() {
            queue!(
                screen.output,
                cursor::MoveTo(ent.0, ent.1),
                style::PrintStyledContent(ent.2)
            )?;
        }

        let next_point = get_next_point(snake.lq.head().unwrap_or(&area.from), &direction);
        if snake.has_collision(&next_point) {
            break;
        }
        if area.has_collision(&next_point) {
            break;
        }
        snake.lq.push(&next_point);
        if food.has_collision(&next_point) {
            food.lq.pop();
            food.lq.push(&(
                rng.gen_range((area.from.0 + 1)..(area.to.0 - 1)),
                rng.gen_range((area.from.1 + 1)..(area.to.1 - 1)),
            ))
        } else {
            snake.lq.pop();
        }

        queue!(screen.output, cursor::MoveTo(cols, rows))?;
        screen.output.flush()?;

        if poll(Duration::from_millis(300))? {
            if let Event::Key(key) = read()? {
                match key.code {
                    KeyCode::Up => direction = Direction::UP,
                    KeyCode::Down => direction = Direction::DOWN,
                    KeyCode::Right => direction = Direction::RIGHT,
                    KeyCode::Left => direction = Direction::LEFT,
                    _ => {}
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    screen.clear()?;
    Ok(())
}
