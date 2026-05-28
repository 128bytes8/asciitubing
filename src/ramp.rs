use opencv::core::{self, Mat, CV_32F};
use opencv::imgproc;
use opencv::prelude::*;

pub const RAMP_FULL: &[u8] = b" .'`^\":;Il!i><~+_-?][}{1)(|\\/tfjrxnuvczXYUJCLQ0OZmwqpdbkhao*#MW&8%B@$";
pub const RAMP_SPARSE: &[u8] = b" .:-=+*#%@";

const BAYER_4: [[u8; 4]; 4] = [
    [0, 8, 2, 10],
    [12, 4, 14, 6],
    [3, 11, 1, 9],
    [15, 7, 13, 5],
];

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RampMode {
    Normal,
    Dense,
    Sparse,
    Inverted,
    Block4,
    Block8,
    Edge,
    Dither,
}

impl RampMode {
    pub const ALL: &[RampMode] = &[
        RampMode::Normal,
        RampMode::Dense,
        RampMode::Sparse,
        RampMode::Inverted,
        RampMode::Block4,
        RampMode::Block8,
        RampMode::Edge,
        RampMode::Dither,
    ];

    pub fn next(self) -> Self {
        let i = Self::ALL.iter().position(|&m| m == self).unwrap_or(0);
        Self::ALL[(i + 1) % Self::ALL.len()]
    }

    pub fn name(self) -> &'static str {
        match self {
            RampMode::Normal => "normal",
            RampMode::Dense => "dense",
            RampMode::Sparse => "sparse",
            RampMode::Inverted => "inverted",
            RampMode::Block4 => "block4",
            RampMode::Block8 => "block8",
            RampMode::Edge => "edge",
            RampMode::Dither => "dither",
        }
    }
}

pub fn compute_edges(gray: &Mat) -> Option<Mat> {
    let mut gx = Mat::default();
    let mut gy = Mat::default();
    let mut mag = Mat::default();
    imgproc::sobel(gray, &mut gx, CV_32F, 1, 0, 3, 1.0, 0.0, opencv::core::BORDER_DEFAULT).ok()?;
    imgproc::sobel(gray, &mut gy, CV_32F, 0, 1, 3, 1.0, 0.0, opencv::core::BORDER_DEFAULT).ok()?;
    core::magnitude(&gx, &gy, &mut mag).ok()?;
    Some(mag)
}

pub fn edge_strength(mag: &Mat, x: i32, y: i32) -> f32 {
    mag.at_2d::<f32>(y, x).map(|v| (*v / 255.0).min(1.0)).unwrap_or(0.0)
}

pub fn char_for(lum: f32, mode: RampMode, x: u16, y: u16, edge_mag: f32) -> char {
    let lum = lum.clamp(0.0, 1.0);

    if mode == RampMode::Edge {
        if edge_mag < 0.12 {
            return ' ';
        }
        return idx_to_char(((edge_mag * 0.7 + lum * 0.3) * (RAMP_FULL.len() - 1) as f32).round() as usize, RAMP_FULL);
    }

    let effective_lum = match mode {
        RampMode::Inverted => 1.0 - lum,
        _ => lum,
    };

    let idx = match mode {
        RampMode::Normal | RampMode::Dense | RampMode::Inverted => {
            (effective_lum * (RAMP_FULL.len() - 1) as f32).round() as usize
        }
        RampMode::Sparse => (effective_lum * (RAMP_SPARSE.len() - 1) as f32).round() as usize,
        RampMode::Block4 => {
            let level = (effective_lum * 4.0).floor() as usize;
            level * (RAMP_FULL.len() - 1) / 4
        }
        RampMode::Block8 => {
            let level = (effective_lum * 8.0).floor() as usize;
            level * (RAMP_FULL.len() - 1) / 8
        }
        RampMode::Dither => {
            let threshold = BAYER_4[y as usize % 4][x as usize % 4] as f32 / 16.0;
            let adjusted = if effective_lum > threshold {
                effective_lum
            } else {
                effective_lum * 0.5
            };
            (adjusted * (RAMP_FULL.len() - 1) as f32).round() as usize
        }
        RampMode::Edge => 0,
    };

    let ramp = match mode {
        RampMode::Sparse => RAMP_SPARSE,
        _ => RAMP_FULL,
    };

    idx_to_char(idx.min(ramp.len().saturating_sub(1)), ramp)
}

fn idx_to_char(idx: usize, ramp: &[u8]) -> char {
    ramp[idx] as char
}
