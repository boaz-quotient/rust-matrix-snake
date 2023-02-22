use std::{collections::HashSet, io::Write};
use crossterm::{cursor, queue, style::{self, Stylize}, terminal};

trait CollisionDetector {
    fn has_collision(&self, point: &(u16, u16)) -> bool;
    
}

struct SnakeState {
    vec: Vec<(u16, u16)>,
    hash: HashSet<(u16, u16)>
}

impl CollisionDetector for SnakeState {
    fn has_collision(&self, point: &(u16, u16)) -> bool {
        self.hash.contains(point)
    }
}

impl SnakeState {
    fn update(&mut self, direction: &Direction) {
        let next_point = self.vec.first().map(|&(x, y)| {
            match direction {
                Direction::DOWN => (x, y+1),
                Direction::UP => (x, y-1),
                Direction::RIGHT => (x+1, y),
                Direction::LEFT => (x-1, y),
            }
        });
        self.vec.pop().and_then(|tail_point| Some(self.hash.remove(&tail_point)));
        next_point.and_then(|head_point| {
            self.vec.insert(0, head_point);
            Some(self.hash.insert(head_point))
        });
    }
}

struct GameArea {
    from: (u16, u16),
    to: (u16, u16)
}

struct FoodState {
    position: (u16, u16),
}

struct ScreenPrinter {
    output: std::io::Stdout,
}

impl ScreenPrinter {
    fn new() -> Self {
        ScreenPrinter { output: std::io::stdout() }
    }

    fn setup(&mut self) -> Result<(), std::io::Error>{
        terminal::enable_raw_mode()?;
        queue!(self.output, cursor::Hide)?;
        Ok(())
    }

    fn clear(&mut self) -> Result<(), std::io::Error> {
        terminal::disable_raw_mode()?;
        queue!(self.output, terminal::Clear(terminal::ClearType::All), cursor::Show, cursor::MoveTo(0, 0))?;
        Ok(())
    }

    fn print(&mut self, snake: &Vec<(u16, u16)>, food: &(u16, u16)) -> Result<(), std::io::Error> {
        queue!(self.output, terminal::Clear(terminal::ClearType::All))?;
        for &(x, y) in snake {
            queue!(self.output, cursor::MoveTo(x, y), style::PrintStyledContent('@'.with(style::Color::Green)))?
        }
        let &(x, y) = food;
        queue!(self.output, cursor::MoveTo(x, y), style::PrintStyledContent('$'.with(style::Color::White)))?;
        self.output.flush()?;
        Ok(())
    }
}

enum Direction {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

fn main() -> Result<(), std::io::Error>{
    let mut screen = ScreenPrinter::new();
    let mut snake = SnakeState { vec: vec![(10, 10)], hash: HashSet::from([(10, 10)]) };
    //let mut area = GameArea { from: (1, 1), to: (100, 100) };
    let food = FoodState { position: (50, 20) };
    let direction: Direction = Direction::DOWN;
    
    let mut count = 11;

    screen.setup()?;
    loop {
        screen.print(&snake.vec, &food.position)?;
        std::thread::sleep(std::time::Duration::from_millis(800));
        snake.update(&direction);
        count -= 1;
        if count == 0 {
            break;
        }

    }
    screen.clear()?;
    Ok(())
}
