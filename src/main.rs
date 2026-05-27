use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, Clear, ClearType},
    QueueableCommand,
};
use opencv::imgproc;
use opencv::prelude::*;
use opencv::videoio::{VideoCapture, VideoCaptureTrait, CAP_V4L2};
use std::io::{self, Write};
use std::time::{Duration, Instant};

const ASCII_RAMP: &[u8] = b" .'`^\":;Il!i><~+_-?][}{1)(|\\/tfjrxnuvczXYUJCLQ0OZmwqpdbkhao*#MW&8%B@$";

#[derive(Clone, Copy)]
enum ColorMode {
    Green,
    Red,
    Blue,
    Yellow,
    Orange,
    Purple,
    Cyan,
    White,
    Rainbow,
}

impl ColorMode {
    fn next(self) -> Self {
        match self {
            ColorMode::Green => ColorMode::Red,
            ColorMode::Red => ColorMode::Blue,
            ColorMode::Blue => ColorMode::Yellow,
            ColorMode::Yellow => ColorMode::Orange,
            ColorMode::Orange => ColorMode::Purple,
            ColorMode::Purple => ColorMode::Cyan,
            ColorMode::Cyan => ColorMode::White,
            ColorMode::White => ColorMode::Rainbow,
            ColorMode::Rainbow => ColorMode::Green,
        }
    }

    fn name(self) -> &'static str {
        match self {
            ColorMode::Green => "green",
            ColorMode::Red => "red",
            ColorMode::Blue => "blue",
            ColorMode::Yellow => "yellow",
            ColorMode::Orange => "orange",
            ColorMode::Purple => "purple",
            ColorMode::Cyan => "cyan",
            ColorMode::White => "white",
            ColorMode::Rainbow => "rainbow",
        }
    }

    fn to_crossterm(self, x: u16, y: u16, t: f32) -> Color {
        match self {
            ColorMode::Green => Color::Rgb { r: 50, g: 255, b: 50 },
            ColorMode::Red => Color::Rgb { r: 255, g: 50, b: 50 },
            ColorMode::Blue => Color::Rgb { r: 50, g: 100, b: 255 },
            ColorMode::Yellow => Color::Rgb { r: 255, g: 235, b: 50 },
            ColorMode::Orange => Color::Rgb { r: 255, g: 140, b: 20 },
            ColorMode::Purple => Color::Rgb { r: 180, g: 50, b: 255 },
            ColorMode::Cyan => Color::Rgb { r: 50, g: 240, b: 255 },
            ColorMode::White => Color::Rgb { r: 230, g: 230, b: 230 },
            ColorMode::Rainbow => {
                let hue = ((x as f32 * 4.0 + y as f32 * 2.0) / 60.0 + t).rem_euclid(1.0);
                hsv_to_rgb(hue, 0.9, 1.0)
            }
        }
    }
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Color {
    let i = (h * 6.0).floor() as i32;
    let f = h * 6.0 - i as f32;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);
    let (r, g, b) = match i % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };
    Color::Rgb {
        r: (r * 255.0) as u8,
        g: (g * 255.0) as u8,
        b: (b * 255.0) as u8,
    }
}

fn main() -> io::Result<()> {
    let mut cap = VideoCapture::new(0, CAP_V4L2)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("opencv: {}", e)))?;

    if !cap.is_opened().unwrap_or(false) {
        return Err(io::Error::new(io::ErrorKind::Other, "Failed to open webcam"));
    }

    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    stdout.queue(Clear(ClearType::All))?;
    stdout.queue(cursor::Hide)?;
    stdout.flush()?;

    let mut color_mode = ColorMode::Green;
    let mut rainbow_t = 0.0f32;
    let mut running = true;

    while running {
        let frame_start = Instant::now();

        if event::poll(Duration::from_millis(1))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => running = false,
                    KeyCode::Char('c') | KeyCode::Char('C') => color_mode = color_mode.next(),
                    _ => {}
                }
            }
        }

        let (term_cols, term_rows) = terminal::size()?;
        let ascii_w = term_cols as i32;
        let ascii_h = term_rows.saturating_sub(1) as i32;

        if ascii_w <= 0 || ascii_h <= 0 {
            std::thread::sleep(Duration::from_millis(16));
            continue;
        }

        let mut frame = Mat::default();
        if cap.read(&mut frame).unwrap_or(false) && !frame.empty() {
            let mut small = Mat::default();
            let _ = imgproc::resize(
                &frame,
                &mut small,
                opencv::core::Size::new(ascii_w, ascii_h),
                0.0,
                0.0,
                imgproc::INTER_AREA,
            );

            let mut gray = Mat::default();
            let _ = imgproc::cvt_color(
                &small,
                &mut gray,
                imgproc::COLOR_BGR2GRAY,
                0,
                opencv::core::AlgorithmHint::ALGO_HINT_DEFAULT,
            );

            rainbow_t += 0.015;

            stdout.queue(cursor::MoveTo(0, 0))?;
            let ramp_len = ASCII_RAMP.len() as f32;

            for y in 0..ascii_h {
                for x in 0..ascii_w {
                    let val = gray.at_2d::<u8>(y, x).unwrap_or(&0);
                    let lum = *val as f32 / 255.0;
                    let idx = ((lum * (ramp_len - 1.0)).round() as usize).min(ASCII_RAMP.len() - 1);
                    let ch = ASCII_RAMP[idx] as char;

                    let color = color_mode.to_crossterm(x as u16, y as u16, rainbow_t);
                    stdout.queue(SetForegroundColor(color))?;
                    stdout.queue(Print(ch))?;
                }
            }

            stdout.queue(ResetColor)?;
            stdout.queue(cursor::MoveTo(0, term_rows - 1))?;
            stdout.queue(Print(format!(
                " color: {} | c:cycle q:quit ",
                color_mode.name()
            )))?;
            stdout.flush()?;
        }

        let elapsed = frame_start.elapsed();
        if elapsed < Duration::from_millis(33) {
            std::thread::sleep(Duration::from_millis(33) - elapsed);
        }
    }

    stdout.queue(ResetColor)?;
    stdout.queue(cursor::Show)?;
    stdout.flush()?;
    terminal::disable_raw_mode()?;
    Ok(())
}
