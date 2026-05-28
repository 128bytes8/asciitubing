use crossterm::style::Color;

pub const LUM_THRESHOLD: f32 = 0.5;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    Green,
    Red,
    Blue,
    Yellow,
    Orange,
    Purple,
    Cyan,
    White,
    Rainbow,
    Checkerboard,
    Matrix,
    Heatmap,
    Grayscale,
    Neon,
    Sepia,
    Complement,
    Scanlines,
    Vignette,
    Fire,
    Ocean,
    Synthwave,
    Duotone,
    Pulse,
    Wave,
    Glitch,
    Strobe,
    Aurora,
    Stripes,
    Dots,
    Radial,
    Noise,
    TrueColor,
    Face,
}

impl ColorMode {
    pub const ALL: &[ColorMode] = &[
        ColorMode::Green,
        ColorMode::Red,
        ColorMode::Blue,
        ColorMode::Yellow,
        ColorMode::Orange,
        ColorMode::Purple,
        ColorMode::Cyan,
        ColorMode::White,
        ColorMode::Rainbow,
        ColorMode::Checkerboard,
        ColorMode::Matrix,
        ColorMode::Heatmap,
        ColorMode::Grayscale,
        ColorMode::Neon,
        ColorMode::Sepia,
        ColorMode::Complement,
        ColorMode::Scanlines,
        ColorMode::Vignette,
        ColorMode::Fire,
        ColorMode::Ocean,
        ColorMode::Synthwave,
        ColorMode::Duotone,
        ColorMode::Pulse,
        ColorMode::Wave,
        ColorMode::Glitch,
        ColorMode::Strobe,
        ColorMode::Aurora,
        ColorMode::Stripes,
        ColorMode::Dots,
        ColorMode::Radial,
        ColorMode::Noise,
        ColorMode::TrueColor,
        ColorMode::Face,
    ];

    pub fn next(self) -> Self {
        let i = Self::ALL.iter().position(|&m| m == self).unwrap_or(0);
        Self::ALL[(i + 1) % Self::ALL.len()]
    }

    pub fn from_digit(d: u8) -> Option<Self> {
        match d {
            1 => Some(ColorMode::Green),
            2 => Some(ColorMode::Red),
            3 => Some(ColorMode::Blue),
            4 => Some(ColorMode::Yellow),
            5 => Some(ColorMode::Orange),
            6 => Some(ColorMode::Purple),
            7 => Some(ColorMode::Cyan),
            8 => Some(ColorMode::White),
            9 => Some(ColorMode::Rainbow),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
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
            ColorMode::Checkerboard => "checkerboard",
            ColorMode::Matrix => "matrix",
            ColorMode::Heatmap => "heatmap",
            ColorMode::Grayscale => "grayscale",
            ColorMode::Neon => "neon",
            ColorMode::Sepia => "sepia",
            ColorMode::Complement => "complement",
            ColorMode::Scanlines => "scanlines",
            ColorMode::Vignette => "vignette",
            ColorMode::Fire => "fire",
            ColorMode::Ocean => "ocean",
            ColorMode::Synthwave => "synthwave",
            ColorMode::Duotone => "duotone",
            ColorMode::Pulse => "pulse",
            ColorMode::Wave => "wave",
            ColorMode::Glitch => "glitch",
            ColorMode::Strobe => "strobe",
            ColorMode::Aurora => "aurora",
            ColorMode::Stripes => "stripes",
            ColorMode::Dots => "dots",
            ColorMode::Radial => "radial",
            ColorMode::Noise => "noise",
            ColorMode::TrueColor => "truecolor",
            ColorMode::Face => "face",
        }
    }

}

pub struct ColorCtx {
    pub x: u16,
    pub y: u16,
    pub lum: f32,
    pub t: f32,
    pub cols: u16,
    pub rows: u16,
    pub tile: u16,
    pub glitch_hit: bool,
    pub in_face: bool,
    pub bgr: Option<(u8, u8, u8)>,
}

pub struct ColorPair {
    pub fg: Color,
    pub bg: Option<Color>,
}

pub fn color_at(mode: ColorMode, ctx: &ColorCtx) -> ColorPair {
    if mode == ColorMode::Glitch && ctx.glitch_hit {
        return ColorPair {
            fg: Color::Rgb {
                r: fastrand::u8(..),
                g: fastrand::u8(..),
                b: fastrand::u8(..),
            },
            bg: None,
        };
    }

    if mode == ColorMode::Strobe && (ctx.t * 4.0) as i32 % 2 == 1 {
        let mut p = color_at_inner(mode, ctx);
        p.fg = invert_color(p.fg);
        if let Some(bg) = p.bg {
            p.bg = Some(invert_color(bg));
        }
        return p;
    }

    if mode == ColorMode::TrueColor {
        if let Some((b, g, r)) = ctx.bgr {
            return ColorPair {
                fg: Color::Rgb { r, g, b },
                bg: None,
            };
        }
    }

    color_at_inner(mode, ctx)
}

fn color_at_inner(mode: ColorMode, ctx: &ColorCtx) -> ColorPair {
    let x = ctx.x;
    let y = ctx.y;
    let lum = ctx.lum;
    let t = ctx.t;
    let cols = ctx.cols.max(1);
    let rows = ctx.rows.max(1);
    let tile = ctx.tile.max(1);

    match mode {
        ColorMode::Green => solid(Color::Rgb { r: 50, g: 255, b: 50 }),
        ColorMode::Red => solid(Color::Rgb { r: 255, g: 50, b: 50 }),
        ColorMode::Blue => solid(Color::Rgb { r: 50, g: 100, b: 255 }),
        ColorMode::Yellow => solid(Color::Rgb { r: 255, g: 235, b: 50 }),
        ColorMode::Orange => solid(Color::Rgb { r: 255, g: 140, b: 20 }),
        ColorMode::Purple => solid(Color::Rgb { r: 180, g: 50, b: 255 }),
        ColorMode::Cyan => solid(Color::Rgb { r: 50, g: 240, b: 255 }),
        ColorMode::White => solid(Color::Rgb { r: 230, g: 230, b: 230 }),

        ColorMode::Rainbow => {
            let hue = ((x as f32 * 4.0 + y as f32 * 2.0) / 60.0 + t).rem_euclid(1.0);
            solid(hsv_to_rgb(hue, 0.9, 1.0))
        }

        ColorMode::Checkerboard => {
            let light = checker_light(x, y, tile);
            ColorPair {
                fg: if light { Color::Black } else { Color::White },
                bg: Some(if light { Color::White } else { Color::Black }),
            }
        }

        ColorMode::Matrix => {
            if lum < 0.15 {
                ColorPair {
                    fg: Color::Black,
                    bg: None,
                }
            } else {
                let g = (lum * 200.0 + 55.0) as u8;
                solid(Color::Rgb { r: 0, g, b: 0 })
            }
        }

        ColorMode::Heatmap => solid(heatmap(lum)),

        ColorMode::Grayscale => {
            let v = (lum * 255.0) as u8;
            solid(Color::Rgb { r: v, g: v, b: v })
        }

        ColorMode::Neon => {
            if lum > 0.7 {
                solid(Color::Rgb { r: 0, g: 255, b: 255 })
            } else if lum > 0.35 {
                solid(Color::Rgb { r: 255, g: 0, b: 180 })
            } else {
                solid(Color::Rgb { r: 40, g: 0, b: 60 })
            }
        }

        ColorMode::Sepia => {
            let r = (lum * 255.0).min(255.0) as u8;
            let g = (lum * 200.0).min(255.0) as u8;
            let b = (lum * 140.0).min(255.0) as u8;
            solid(Color::Rgb { r, g, b })
        }

        ColorMode::Complement => {
            if lum > LUM_THRESHOLD {
                solid(Color::Black)
            } else {
                solid(Color::White)
            }
        }

        ColorMode::Scanlines => {
            if y % 2 == 0 {
                solid(Color::Rgb {
                    r: (lum * 220.0) as u8,
                    g: (lum * 220.0) as u8,
                    b: (lum * 220.0) as u8,
                })
            } else {
                ColorPair {
                    fg: Color::Rgb {
                        r: (lum * 80.0) as u8,
                        g: (lum * 80.0) as u8,
                        b: (lum * 90.0) as u8,
                    },
                    bg: Some(Color::Rgb { r: 20, g: 20, b: 30 }),
                }
            }
        }

        ColorMode::Vignette => {
            let cx = cols as f32 / 2.0;
            let cy = rows as f32 / 2.0;
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let dist = (dx * dx + dy * dy).sqrt();
            let max_dist = (cx * cx + cy * cy).sqrt();
            let vig = 1.0 - (dist / max_dist).min(1.0).powf(1.5);
            let v = (lum * vig * 255.0) as u8;
            solid(Color::Rgb { r: v, g: v, b: (v as f32 * 1.1) as u8 })
        }

        ColorMode::Fire => {
            let row = y as f32 / rows as f32;
            let heat = (1.0 - row + t * 0.15).rem_euclid(1.0);
            let r = ((heat + lum) * 0.5 * 255.0).min(255.0) as u8;
            let g = (heat * lum * 180.0).min(255.0) as u8;
            let b = (lum * heat * 40.0) as u8;
            solid(Color::Rgb { r, g, b })
        }

        ColorMode::Ocean => {
            let r = (lum * 30.0) as u8;
            let g = (lum * 120.0 + 40.0).min(255.0) as u8;
            let b = (lum * 200.0 + 55.0).min(255.0) as u8;
            solid(Color::Rgb { r, g, b })
        }

        ColorMode::Synthwave => {
            let row = y as f32 / rows as f32;
            let base = hsv_to_rgb(0.78 - row * 0.2, 0.7, 0.35 + row * 0.25);
            if lum > 0.65 {
                solid(Color::Rgb { r: 0, g: 255, b: 255 })
            } else {
                solid(scale_color(base, 0.4 + lum * 0.8))
            }
        }

        ColorMode::Duotone => {
            let navy = Color::Rgb { r: 20, g: 30, b: 90 };
            let gold = Color::Rgb { r: 255, g: 200, b: 60 };
            solid(lerp_color(navy, gold, lum))
        }

        ColorMode::Pulse => {
            let pulse = 0.55 + 0.45 * (t * 3.0).sin();
            let g = (lum * pulse * 255.0) as u8;
            solid(Color::Rgb { r: (g / 3) as u8, g, b: (g / 2) as u8 })
        }

        ColorMode::Wave => {
            let hue = (x as f32 / cols as f32 + t * 0.2).rem_euclid(1.0);
            solid(hsv_to_rgb(hue, 0.85, 0.5 + lum * 0.5))
        }

        ColorMode::Glitch => solid(Color::Rgb { r: 0, g: 255, b: 100 }),

        ColorMode::Strobe => solid(Color::Rgb {
            r: (lum * 255.0) as u8,
            g: (lum * 255.0) as u8,
            b: (lum * 255.0) as u8,
        }),

        ColorMode::Aurora => {
            let hue = (y as f32 / rows as f32 * 0.4 + t * 0.08).rem_euclid(1.0);
            solid(hsv_to_rgb(hue, 0.75, 0.35 + lum * 0.65))
        }

        ColorMode::Stripes => {
            let stripe = (x / tile) % 2 == 0;
            if stripe {
                ColorPair {
                    fg: Color::Rgb { r: 30, g: 30, b: 40 },
                    bg: Some(Color::Rgb { r: 200, g: 220, b: 255 }),
                }
            } else {
                ColorPair {
                    fg: Color::White,
                    bg: Some(Color::Rgb { r: 20, g: 20, b: 30 }),
                }
            }
        }

        ColorMode::Dots => {
            let tx = x / tile;
            let ty = y / tile;
            let dot = (tx + ty) % 2 == 0 && (x % tile < 2 || y % tile < 2);
            if dot {
                solid(Color::White)
            } else {
                ColorPair {
                    fg: Color::Rgb { r: 30, g: 30, b: 40 },
                    bg: Some(Color::Black),
                }
            }
        }

        ColorMode::Radial => {
            let cx = cols as f32 / 2.0;
            let cy = rows as f32 / 2.0;
            let dist = ((x as f32 - cx).powi(2) + (y as f32 - cy).powi(2)).sqrt();
            let ring = (dist / tile as f32) as u32 % 2 == 0;
            if ring {
                solid(hsv_to_rgb(lum * 0.3 + t * 0.05, 0.8, 0.9))
            } else {
                solid(Color::Rgb { r: 15, g: 15, b: 25 })
            }
        }

        ColorMode::Noise => {
            let h = hash01(x, y, (t * 1000.0) as u32);
            solid(hsv_to_rgb(h, 0.7, 0.3 + lum * 0.7))
        }

        ColorMode::TrueColor => solid(Color::White),

        ColorMode::Face => {
            if ctx.in_face {
                solid(Color::Rgb { r: 255, g: 120, b: 180 })
            } else {
                solid(Color::Rgb { r: 80, g: 180, b: 255 })
            }
        }
    }
}

fn solid(c: Color) -> ColorPair {
    ColorPair { fg: c, bg: None }
}

fn checker_light(x: u16, y: u16, tile: u16) -> bool {
    let tx = x / tile;
    let ty = y / tile;
    (tx + ty) % 2 == 0
}

fn heatmap(lum: f32) -> Color {
    if lum < 0.25 {
        let t = lum / 0.25;
        Color::Rgb {
            r: 0,
            g: (t * 100.0) as u8,
            b: (180.0 + t * 75.0) as u8,
        }
    } else if lum < 0.5 {
        let t = (lum - 0.25) / 0.25;
        Color::Rgb {
            r: (t * 180.0) as u8,
            g: (200.0 + t * 55.0) as u8,
            b: (255.0 - t * 200.0) as u8,
        }
    } else if lum < 0.75 {
        let t = (lum - 0.5) / 0.25;
        Color::Rgb {
            r: (180.0 + t * 75.0) as u8,
            g: 255,
            b: 0,
        }
    } else {
        let t = (lum - 0.75) / 0.25;
        Color::Rgb {
            r: 255,
            g: (255.0 - t * 200.0) as u8,
            b: 0,
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

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let (ar, ag, ab) = rgb(a);
    let (br, bg, bb) = rgb(b);
    Color::Rgb {
        r: lerp_u8(ar, br, t),
        g: lerp_u8(ag, bg, t),
        b: lerp_u8(ab, bb, t),
    }
}

fn scale_color(c: Color, s: f32) -> Color {
    let (r, g, b) = rgb(c);
    Color::Rgb {
        r: (r as f32 * s).min(255.0) as u8,
        g: (g as f32 * s).min(255.0) as u8,
        b: (b as f32 * s).min(255.0) as u8,
    }
}

fn invert_color(c: Color) -> Color {
    let (r, g, b) = rgb(c);
    Color::Rgb {
        r: 255 - r,
        g: 255 - g,
        b: 255 - b,
    }
}

fn rgb(c: Color) -> (u8, u8, u8) {
    match c {
        Color::Rgb { r, g, b } => (r, g, b),
        Color::Black => (0, 0, 0),
        Color::White => (255, 255, 255),
        _ => (128, 128, 128),
    }
}

fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t).round() as u8
}

fn hash01(x: u16, y: u16, seed: u32) -> f32 {
    let mut n = x as u32 ^ ((y as u32) << 16) ^ seed.wrapping_mul(0x9E37_79B9);
    n ^= n >> 16;
    n = n.wrapping_mul(0x7FEB_352D);
    n ^= n >> 15;
    n = n.wrapping_mul(0x846C_AEB9);
    n ^= n >> 16;
    (n & 0xFFFF) as f32 / 65535.0
}
