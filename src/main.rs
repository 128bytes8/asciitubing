mod color;
mod ramp;

use color::{color_at, ColorCtx, ColorMode, LUM_THRESHOLD};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    style::{Color as TermColor, Print, ResetColor, SetForegroundColor},
    terminal::{self, Clear, ClearType},
    QueueableCommand,
};
use opencv::core::{Rect, Size};
use opencv::imgproc;
use opencv::objdetect::{CascadeClassifier, CascadeClassifierTrait};
use opencv::prelude::*;
use opencv::videoio::{VideoCapture, VideoCaptureTrait, CAP_V4L2};
use ramp::{compute_edges, edge_strength, char_for, RampMode};
use std::fmt::Write as FmtWrite;
use std::io::{self, Write};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

struct FrameCache {
    bgr: Mat,
    gray: Mat,
    edges: Option<Mat>,
}

struct FaceDetector {
    cascade: Option<CascadeClassifier>,
    rect: Option<Rect>,
}

impl FaceDetector {
    fn new() -> Self {
        let path = "/usr/share/opencv4/haarcascades/haarcascade_frontalface_default.xml";
        let cascade = CascadeClassifier::new(path)
            .or_else(|_| CascadeClassifier::new("/usr/share/opencv/haarcascades/haarcascade_frontalface_default.xml"))
            .ok();
        Self {
            cascade,
            rect: None,
        }
    }

    fn update(&mut self, gray: &Mat) {
        let Some(ref mut cascade) = self.cascade else {
            self.rect = None;
            return;
        };
        let mut faces = opencv::core::Vector::<Rect>::new();
        if cascade
            .detect_multi_scale(gray, &mut faces, 1.2, 3, 0, Size::new(60, 60), Size::new(0, 0))
            .is_ok()
            && !faces.is_empty()
        {
            self.rect = faces.get(0).ok();
        } else {
            self.rect = None;
        }
    }

    fn contains(&self, x: i32, y: i32) -> bool {
        let Some(r) = self.rect else {
            return false;
        };
        x >= r.x && x < r.x + r.width && y >= r.y && y < r.y + r.height
    }
}

struct GlitchState {
    columns: Vec<u16>,
    frames_left: u8,
}

impl GlitchState {
    fn tick(&mut self, cols: u16) {
        if self.frames_left > 0 {
            self.frames_left -= 1;
            return;
        }
        if fastrand::u8(..) < 20 {
            self.columns.clear();
            let n = fastrand::usize(1..=8);
            for _ in 0..n {
                self.columns.push(fastrand::u16(..cols));
            }
            self.frames_left = 2;
        }
    }

    fn hit(&self, x: u16) -> bool {
        self.frames_left > 0 && self.columns.contains(&x)
    }
}

struct FrameStats {
    dark: u32,
    light: u32,
    lum_sum: f64,
    total: u32,
}

impl FrameStats {
    fn new() -> Self {
        Self {
            dark: 0,
            light: 0,
            lum_sum: 0.0,
            total: 0,
        }
    }

    fn record(&mut self, lum: f32) {
        self.total += 1;
        self.lum_sum += lum as f64;
        if lum < LUM_THRESHOLD {
            self.dark += 1;
        } else {
            self.light += 1;
        }
    }

    fn avg_lum(&self) -> f32 {
        if self.total == 0 {
            0.0
        } else {
            (self.lum_sum / self.total as f64) as f32
        }
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
    let mut ramp_mode = RampMode::Normal;
    let mut mirror = false;
    let mut invert_lum = false;
    let mut paused = false;
    let mut tile_size: u16 = 100;
    let mut anim_t = 0.0f32;
    let mut glitch = GlitchState {
        columns: Vec::new(),
        frames_left: 0,
    };
    let mut face_det = FaceDetector::new();
    let mut cache: Option<FrameCache> = None;
    let mut running = true;

    while running {
        let frame_start = Instant::now();

        if event::poll(Duration::from_millis(1))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => running = false,
                    KeyCode::Char('c') | KeyCode::Char('C') => color_mode = color_mode.next(),
                    KeyCode::Char('b') | KeyCode::Char('B') => ramp_mode = ramp_mode.next(),
                    KeyCode::Char('m') | KeyCode::Char('M') => mirror = !mirror,
                    KeyCode::Char('p') | KeyCode::Char('P') => paused = !paused,
                    KeyCode::Char('i') | KeyCode::Char('I') => invert_lum = !invert_lum,
                    KeyCode::Char('s') | KeyCode::Char('S') => {
                        if let Some(ref c) = cache {
                            let _ = save_screenshot(c, color_mode, ramp_mode, tile_size, mirror, invert_lum);
                        }
                    }
                    KeyCode::Char('+') | KeyCode::Char('=') => {
                        tile_size = (tile_size.saturating_add(10)).min(300);
                    }
                    KeyCode::Char('-') | KeyCode::Char('_') => {
                        tile_size = tile_size.saturating_sub(10).max(4);
                    }
                    KeyCode::Char(d) if d.is_ascii_digit() => {
                        if let Some(m) = ColorMode::from_digit(d as u8 - b'0') {
                            color_mode = m;
                        }
                    }
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

        if !paused {
            let mut frame = Mat::default();
            if cap.read(&mut frame).unwrap_or(false) && !frame.empty() {
                if mirror {
                    let mut flipped = Mat::default();
                    let _ = opencv::core::flip(&frame, &mut flipped, 1);
                    frame = flipped;
                }

                let mut bgr = Mat::default();
                let _ = imgproc::resize(
                    &frame,
                    &mut bgr,
                    Size::new(ascii_w, ascii_h),
                    0.0,
                    0.0,
                    imgproc::INTER_AREA,
                );

                let mut gray = Mat::default();
                let _ = imgproc::cvt_color(
                    &bgr,
                    &mut gray,
                    imgproc::COLOR_BGR2GRAY,
                    0,
                    opencv::core::AlgorithmHint::ALGO_HINT_DEFAULT,
                );

                let edges = if ramp_mode == RampMode::Edge {
                    compute_edges(&gray)
                } else {
                    None
                };

                face_det.update(&gray);

                cache = Some(FrameCache { bgr, gray, edges });
            }
        }

        if let Some(ref cached) = cache {
            anim_t += 0.015;
            glitch.tick(term_cols);

            let mut stats = FrameStats::new();
            let mut screen = String::with_capacity((ascii_w * ascii_h) as usize * 2);

            for y in 0..ascii_h {
                for x in 0..ascii_w {
                    let raw_lum = *cached.gray.at_2d::<u8>(y, x).unwrap_or(&0) as f32 / 255.0;
                    let lum = if invert_lum { 1.0 - raw_lum } else { raw_lum };
                    stats.record(lum);

                    let edge_mag = cached
                        .edges
                        .as_ref()
                        .map(|m| edge_strength(m, x, y))
                        .unwrap_or(0.0);

                    let ch = char_for(lum, ramp_mode, x as u16, y as u16, edge_mag);

                    let bgr_pixel = cached.bgr.at_2d::<opencv::core::Vec3b>(y, x).ok().map(|v| {
                        (v[0], v[1], v[2])
                    });

                    let ctx = ColorCtx {
                        x: x as u16,
                        y: y as u16,
                        lum,
                        t: anim_t,
                        cols: term_cols,
                        rows: term_rows.saturating_sub(1),
                        tile: tile_size,
                        glitch_hit: glitch.hit(x as u16),
                        in_face: face_det.contains(x, y),
                        bgr: bgr_pixel,
                    };

                    let pair = color_at(color_mode, &ctx);

                    if let Some(bg) = pair.bg {
                        let (br, bgc, bb) = term_color_rgb(bg);
                        let _ = write!(screen, "\x1b[48;2;{};{};{}m", br, bgc, bb);
                    }
                    let (fr, fg, fb) = term_color_rgb(pair.fg);
                    let _ = write!(screen, "\x1b[38;2;{};{};{}m{}", fr, fg, fb, ch);
                }
            }

            stdout.queue(cursor::MoveTo(0, 0))?;
            stdout.queue(Print(&screen))?;
            stdout.queue(ResetColor)?;

            let status = format!(
                " color:{} | dark:{} light:{} avg:{:.2} | ramp:{} tile:{} | {}{}{} | c:color b:ramp m:mirror p:pause i:invert s:shot +/-:tile 1-9:jump q:quit ",
                color_mode.name(),
                stats.dark,
                stats.light,
                stats.avg_lum(),
                ramp_mode.name(),
                tile_size,
                if mirror { "MIRROR " } else { "" },
                if paused { "PAUSED " } else { "" },
                if invert_lum { "INV " } else { "" },
            );

            stdout.queue(cursor::MoveTo(0, term_rows - 1))?;
            stdout.queue(SetForegroundColor(crossterm::style::Color::DarkGrey))?;
            let status_show = if status.len() > term_cols as usize {
                &status[..term_cols as usize]
            } else {
                &status
            };
            stdout.queue(Print(status_show))?;
            stdout.queue(ResetColor)?;
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

fn term_color_rgb(c: TermColor) -> (u8, u8, u8) {
    match c {
        TermColor::Rgb { r, g, b } => (r, g, b),
        TermColor::Black => (0, 0, 0),
        TermColor::White => (255, 255, 255),
        TermColor::DarkGrey => (80, 80, 80),
        TermColor::Grey => (128, 128, 128),
        TermColor::Red => (255, 0, 0),
        TermColor::Green => (0, 255, 0),
        TermColor::Blue => (0, 0, 255),
        TermColor::Yellow => (255, 255, 0),
        TermColor::Cyan => (0, 255, 255),
        TermColor::Magenta => (255, 0, 255),
        _ => (200, 200, 200),
    }
}

fn save_screenshot(
    cache: &FrameCache,
    color_mode: ColorMode,
    ramp_mode: RampMode,
    tile_size: u16,
    mirror: bool,
    invert_lum: bool,
) -> io::Result<()> {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let path = format!("asciitubing_{}.txt", ts);
    let h = cache.gray.rows();
    let w = cache.gray.cols();
    let mut out = String::new();
    for y in 0..h {
        for x in 0..w {
            let raw = *cache.gray.at_2d::<u8>(y, x).unwrap_or(&0) as f32 / 255.0;
            let lum = if invert_lum { 1.0 - raw } else { raw };
            let edge_mag = cache
                .edges
                .as_ref()
                .map(|m| edge_strength(m, x, y))
                .unwrap_or(0.0);
            let ch = char_for(lum, ramp_mode, x as u16, y as u16, edge_mag);
            out.push(ch);
        }
        out.push('\n');
    }
    let header = format!(
        "# asciitubing capture\n# color:{} ramp:{} tile:{} mirror:{} invert:{}\n",
        color_mode.name(),
        ramp_mode.name(),
        tile_size,
        mirror,
        invert_lum
    );
    std::fs::write(&path, format!("{header}{out}"))?;
    Ok(())
}
