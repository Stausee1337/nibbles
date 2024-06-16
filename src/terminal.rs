use std::fmt::Write;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum TermColor {
    Default,
    Color(u8)
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) struct Cell {
    pub foreground: TermColor,
    pub background: TermColor,
    pub data: char
}

impl Default for Cell {
    fn default() -> Self {
        Cell {
            foreground: TermColor::Default,
            background: TermColor::Default,
            data: ' '
        }
    }
}

pub(crate) struct Terminal {
    foreground: TermColor,
    background: TermColor,
    pub size: (usize, usize),
    cursor: (usize, usize),
    is_related: bool,
    pub rows: Vec<Vec<Cell>>
}

impl Terminal {
    pub(crate) fn new(width: usize, height: usize) -> Terminal {
        Terminal {
            foreground: TermColor::Default,
            background: TermColor::Default,
            size: (width, height),
            cursor: (0, 0),
            is_related: true,
            rows: vec![vec![Cell::default(); width]; height]
        }
    }

    pub fn clear(&mut self) {
        self.is_related = false;
    }

    pub fn set_foreground(&mut self, color: TermColor) {
        self.foreground = color;
    }

    pub fn set_background(&mut self, color: TermColor) {
        self.background = color;
    }

    pub fn goto(&mut self, mut x: usize, mut y: usize) {
        x = clamp(x, 0, self.size.0 - 1);
        y = clamp(y, 0, self.size.1 - 1);
        self.cursor = (x, y);
    }

    pub fn up(&mut self, amount: usize) {
        self.goto(self.cursor.0, self.cursor.1.checked_sub(amount).unwrap_or(0));
    }

    pub fn right(&mut self, amount: usize) {
        self.goto(self.cursor.0 + amount, self.cursor.1);
    }

    pub fn down(&mut self, amount: usize) {
        self.goto(self.cursor.0, self.cursor.1 + amount);
    }

    pub fn left(&mut self, amount: usize) {
        self.goto(self.cursor.0.checked_sub(amount).unwrap_or(0), self.cursor.1);
    }

    pub fn ret(&mut self) {
        self.cursor.0 = 0;
    }
}

impl Write for Terminal {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        let lines = s.split_inclusive("\n");
        for line in lines {
            let insert_newline = line.ends_with("\n");
            let chars: Vec<_> = line.chars().collect();

            let left_space = (self.size.0 - self.cursor.0) + 1;
            let slice_len = std::cmp::min(chars.len() - insert_newline as usize, left_space);
            

            for char in chars[..slice_len].iter() {
                let cell = &mut self.rows[self.cursor.1][self.cursor.0];
                cell.data = *char;
                cell.foreground = self.foreground;
                cell.background = self.background;
                self.right(1);
            }

            if insert_newline {
                self.goto(0, self.cursor.1 + 1);
            }
        }
        Ok(())
    }
}

pub fn clamp<T: PartialOrd>(input: T, min: T, max: T) -> T {
    debug_assert!(min <= max, "min must be less than or equal to max");
    if input < min {
        min
    } else if input > max {
        max
    } else {
        input
    }
}
