//! Fond (couleur, dégradé ou image) et composition du sujet détouré.

use super::load;
use crate::params::{parse_hex, Background};
use anyhow::{anyhow, Context, Result};
use image::{imageops, imageops::FilterType, RgbImage, RgbaImage};
use rayon::prelude::*;
use std::path::Path;

/// Fond prêt à l'emploi, calculé une seule fois pour tout le lot.
pub struct Canvas {
    width: u32,
    height: u32,
    pixels: Vec<[f32; 3]>,
}

impl Canvas {
    pub fn render(bg: &Background, w: u32, h: u32) -> Result<Self> {
        let n = (w * h) as usize;
        let pixels = match bg {
            Background::Color { color } => vec![rgb(color)?; n],
            Background::Gradient { top, bottom } => {
                let (a, b) = (rgb(top)?, rgb(bottom)?);
                (0..h)
                    .flat_map(|y| {
                        let t = if h > 1 { y as f32 / (h - 1) as f32 } else { 0.0 };
                        let c = [0, 1, 2].map(|i| a[i] + (b[i] - a[i]) * t);
                        std::iter::repeat_n(c, w as usize)
                    })
                    .collect()
            }
            Background::Image { path } => {
                let img = load::open(Path::new(path)).context("image de fond")?;
                cover(&img, w, h)
                    .pixels()
                    .map(|p| [p[0] as f32 / 255.0, p[1] as f32 / 255.0, p[2] as f32 / 255.0])
                    .collect()
            }
        };
        Ok(Self { width: w, height: h, pixels })
    }
}

fn rgb(hex: &str) -> Result<[f32; 3]> {
    let c = parse_hex(hex).map_err(|e| anyhow!(e))?;
    Ok(c.map(|v| v as f32 / 255.0))
}

/// Remplit tout le cadre sans déformer (recadrage centré).
fn cover(img: &RgbImage, w: u32, h: u32) -> RgbImage {
    let (iw, ih) = (img.width() as f32, img.height() as f32);
    let s = (w as f32 / iw).max(h as f32 / ih);
    let nw = ((iw * s).ceil() as u32).max(w);
    let nh = ((ih * s).ceil() as u32).max(h);
    let resized = imageops::resize(img, nw, nh, FilterType::Lanczos3);
    imageops::crop_imm(&resized, (nw - w) / 2, (nh - h) / 2, w, h).to_image()
}

fn to_u8(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8
}

pub fn composite(fg: &[[f32; 3]], alpha: &[f32], canvas: &Canvas) -> RgbImage {
    let pixels: Vec<u8> = fg
        .par_iter()
        .zip(alpha.par_iter())
        .zip(canvas.pixels.par_iter())
        .flat_map_iter(|((f, a), b)| (0..3).map(move |c| to_u8(f[c] * a + b[c] * (1.0 - a))))
        .collect();
    RgbImage::from_raw(canvas.width, canvas.height, pixels).expect("taille cohérente")
}

pub fn transparent(fg: &[[f32; 3]], alpha: &[f32], w: u32, h: u32) -> RgbaImage {
    let pixels: Vec<u8> = fg
        .iter()
        .zip(alpha)
        .flat_map(|(f, a)| [to_u8(f[0]), to_u8(f[1]), to_u8(f[2]), to_u8(*a)])
        .collect();
    RgbaImage::from_raw(w, h, pixels).expect("taille cohérente")
}
