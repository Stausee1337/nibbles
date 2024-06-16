use rand::{Rng, seq::SliceRandom};

use crate::{Board, SnakeCell, Direction};

pub(crate) fn render_l1(_: &mut Board) {}

pub(crate) fn spawn_l1(board: &Board) -> (Vec<SnakeCell>, Direction) {
    let Board { width, height, .. } = *board;
    let center_x = width / 2;
    let center_y = height / 2;

    let mut thread_random = rand::thread_rng();
    let head = SnakeCell(
        thread_random.gen_range(center_x - 3 .. center_x + 3) as i32,
        thread_random.gen_range(center_y - 3 .. center_y + 3) as i32
    );
    let mut vec = vec![head];
    let direction = thread_random.gen();
    let SnakeCell(mut nx, mut ny) = head;
    match direction {
        Direction::Up | Direction::Down => ny -= direction.as_integer(),
        Direction::Right | Direction::Left => nx -= direction.as_integer()
    }
    vec.push(SnakeCell(nx, ny));

    (vec, direction)
}

pub(crate) fn render_l2(board: &mut Board) {
    let Board { width, height, .. } = *board;

    let bar_width = width - (width / 2);
    let bar_y = height / 2;
    let bar_x = (width / 2) - (bar_width / 2);

    draw_line(board, bar_x, bar_y, bar_x + bar_width, bar_y);
}

pub(crate) fn spawn_l2(board: &Board) -> (Vec<SnakeCell>, Direction) {
    let Board { width, height, .. } = *board;

    let mut thread_random = rand::thread_rng();
    let mut head = SnakeCell(
        thread_random.gen_range(1..width) as i32,
        thread_random.gen_range(1..height) as i32
    );

    if head.1 == height as i32 / 2 {
        head.1 -= 1;
    }
    let mut vec = vec![head];
    let direction = *[Direction::Left, Direction::Right].choose(&mut thread_random).unwrap();
    let mut nx = head.0;
    nx -= direction.as_integer();
    vec.push(SnakeCell(nx, head.1));

    (vec, direction)
}

pub(crate) fn render_l3(board: &mut Board) {
    let Board { width, height, .. } = *board;

    let bar_height = height - (height / 3);
    let bar_y = height / 2 - bar_height / 2;
    let mut bar_x = width / 3;

    draw_line(board, bar_x, bar_y, bar_x, bar_y + bar_height);

    bar_x += width / 3;
    draw_line(board, bar_x, bar_y, bar_x, bar_y + bar_height);
}

pub(crate) fn spawn_l3(board: &Board) -> (Vec<SnakeCell>, Direction) {
    let Board { width, height, .. } = *board;

    let mut thread_random = rand::thread_rng();
    let mut head = SnakeCell(
        thread_random.gen_range(1..width) as i32,
        thread_random.gen_range(1..height) as i32
    );

    if head.0 == width as i32 / 3 || head.0 == (width as i32 / 3) * 2 {
        head.0 -= 1;
    }
    let mut vec = vec![head];
    let direction = *[Direction::Up, Direction::Down].choose(&mut thread_random).unwrap();
    let mut ny = head.1;
    ny -= direction.as_integer();
    vec.push(SnakeCell(head.0, ny));

    (vec, direction)
}


pub(crate) fn render_l4(board: &mut Board) {
    let Board { width, height, .. } = *board;

    let bar_space = (height as f32 * 0.4).round() as usize;
    let bar_height = height - bar_space;
    let mut bar_y = 1;
    let mut bar_x = width / 4;

    // vertical
    draw_line(board, bar_x, bar_y, bar_x, bar_y + bar_height);

    bar_x += width / 2;
    bar_y = height - bar_height;
    // vertical
    draw_line(board, bar_x, bar_y, bar_x, bar_y + bar_height);

    let bar_width = width - (width / 2);
    let bar_x = width - 1 - bar_width;
    let mut bar_y = bar_space / 2;

    // horizontal
    draw_line(board, bar_x, bar_y, bar_x + bar_width, bar_y);

    bar_y += bar_height;
    // horizontal
    draw_line(board, 1, bar_y, 1 + bar_width, bar_y);
}

pub(crate) fn spawn_centred(board: &Board) -> (Vec<SnakeCell>, Direction) {
    let Board { width, height, .. } = *board;

    let mut thread_random = rand::thread_rng();
    let head = SnakeCell(
        width as i32 / 2,
        height as i32 / 2
    );

    let mut vec = vec![head];
    let direction: Direction = thread_random.gen();
    let SnakeCell(mut nx, mut ny) = head;
    match direction {
        Direction::Up | Direction::Down => ny -= direction.as_integer(),
        Direction::Right | Direction::Left => nx -= direction.as_integer()
    }
    vec.push(SnakeCell(nx, ny));

    (vec, direction)
}

pub(crate) fn render_l5(board: &mut Board) {
    let Board { width, height, .. } = *board;

    let bar_height = (height / 2) - 4;
    let bar_y = height / 2 - bar_height / 2;
    let mut bar_x = width / 4;

    // vertical
    draw_line(board, bar_x, bar_y, bar_x, bar_y + bar_height);

    bar_x += width / 2;
    // vertical
    draw_line(board, bar_x, bar_y, bar_x, bar_y + bar_height);

    let bar_width = (width / 2) - 4;
    let bar_x = width / 2 - bar_width / 2 - 1;
    let mut bar_y = height / 4;

    // horizontal
    draw_line(board, bar_x, bar_y, bar_x + bar_width, bar_y);

    bar_y += height / 2;
    // horizontal
    draw_line(board, bar_x, bar_y, bar_x + bar_width, bar_y);
}

pub(crate) fn render_l6(board: &mut Board) {
    let Board { width, height, .. } = *board;

    let stepwidth = width / 8;
    let mut bar_x = stepwidth;
    let bar_height = height / 2 - height / 12;
    for _ in 0..8 {
        // vertiacl
        draw_line(board, bar_x, 1, bar_x, 1 + bar_height);

        draw_line(board, bar_x, height - bar_height, bar_x, height);
        bar_x += stepwidth;
    }
}

pub(crate) fn render_l7(board: &mut Board) {
    let Board { width, height, .. } = *board;
    let bar_x = width / 2;

    for y in 0..height {
        if y % 2 != 0 {
            board.set_pixel(bar_x, y, 9);
        }
    }
}

pub(crate) fn render_l8(board: &mut Board) {
    let Board { width, height, .. } = *board;

    let stepwidth = width / 4;
    let mut bar_x = 2 + stepwidth / 2;
    let bar_height = height - height / 6;
    for _ in 0..4 {
        draw_line(board, bar_x, 1, bar_x, 1 + bar_height);
        bar_x += stepwidth;
    }

    let stepwidth = width / 4;
    let mut bar_x = stepwidth + 2;
    let bar_height = height - height / 6;
    for _ in 0..3 {
        draw_line(board, bar_x, height - bar_height, bar_x, height);
        bar_x += stepwidth;
    }
}

fn draw_line_low(board: &mut Board, x0: isize, y0: isize, x1: isize, y1: isize) {
    let dx = x1 - x0;
    let mut dy = y1 - y0;
    let mut yi = 1;
    if dy < 0 {
        yi = -1;
        dy = -dy;
    }
    let mut distance = (2 * dy) - dx;
    let mut y = y0;

    for x in x0..=x1 {
        board.set_pixel(x as usize, y as usize, 9);
        if distance > 0 {
            y += yi;
            distance += 2 * (dy - dx);
        } else {
            distance += 2 * dy;
        }
    }
}

fn draw_line_high(board: &mut Board, x0: isize, y0: isize, x1: isize, y1: isize) {
    let mut dx = x1 - x0;
    let dy = y1 - y0;
    let mut xi = 1;
    if dx < 0 {
        xi = -1;
        dx = -dx;
    }
    let mut distance = (2 * dx) - dy;
    let mut x = x0;

    for y in y0..=y1 {
        board.set_pixel(x as usize, y as usize, 9);
        if distance > 0 {
            x += xi;
            distance += 2 * (dx - dy);
        } else {
            distance += 2 * dx;
        }
    }
}

fn draw_line(board: &mut Board, x0: usize, y0: usize, x1: usize, y1: usize) {
    let (x0, y0) = (x0 as isize, y0 as isize);
    let (x1, y1) = (x1 as isize, y1 as isize);

    if (y1 - y0).abs() < (x1 - x0).abs() {
        if x0 > x1 {
            draw_line_low(board, x1, y1, x0, y0);
        } else {
            draw_line_low(board, x0, y0, x1, y1);
        }
    } else {
        if y0 > y1 {
            draw_line_high(board, x1, y1, x0, y0);
        } else {
            draw_line_high(board, x0, y0, x1, y1);
        }
    }
}
