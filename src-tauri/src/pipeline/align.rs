//! Mise à l'échelle (écart des yeux), redressement et recadrage en une seule
//! transformation : le milieu des yeux tombe toujours au même point du cadre.

use crate::params::Params;
use image::{imageops, imageops::FilterType, RgbImage};
use rayon::prelude::*;
use std::borrow::Cow;

pub struct Aligned {
    pub image: RgbImage,
    /// Pixel issu de la photo (faux = bord complété).
    pub inside: Vec<bool>,
    /// Part du cadre couverte par la photo d'origine.
    pub coverage: f32,
    /// Facteur d'agrandissement (> 1 = photo trop petite).
    pub scale: f32,
}

pub fn align(src: &RgbImage, left: [f32; 2], right: [f32; 2], p: &Params) -> Aligned {
    let (dx, dy) = (right[0] - left[0], right[1] - left[1]);
    let distance = (dx * dx + dy * dy).sqrt().max(1.0);
    let scale = p.eye_distance / distance;
    let (sin, cos) = dy.atan2(dx).sin_cos();

    // Grosse réduction : on réduit d'abord avec un bon filtre (anti-crénelage),
    // la transformation finale reste alors proche de l'échelle 1.
    let (work, k) = if scale < 0.9 {
        let nw = ((src.width() as f32 * scale).round() as u32).max(1);
        let nh = ((src.height() as f32 * scale).round() as u32).max(1);
        let k = nw as f32 / src.width() as f32;
        (Cow::Owned(imageops::resize(src, nw, nh, FilterType::CatmullRom)), k)
    } else {
        (Cow::Borrowed(src), 1.0)
    };
    let s = scale / k;
    let mid = [(left[0] + right[0]) / 2.0 * k, (left[1] + right[1]) / 2.0 * k];
    let (ww, wh) = (work.width() as f32, work.height() as f32);

    let (w, h) = (p.width as usize, p.height as usize);
    let (ox, oy) = (p.eye_x * w as f32, p.eye_y * h as f32);
    let mut pixels = vec![0u8; w * h * 3];
    let mut inside = vec![false; w * h];
    pixels
        .par_chunks_mut(w * 3)
        .zip(inside.par_chunks_mut(w))
        .enumerate()
        .for_each(|(v, (row, row_inside))| {
            let dv = (v as f32 + 0.5 - oy) / s;
            for u in 0..w {
                let du = (u as f32 + 0.5 - ox) / s;
                let sx = mid[0] + du * cos - dv * sin;
                let sy = mid[1] + du * sin + dv * cos;
                row_inside[u] = sx >= 0.0 && sy >= 0.0 && sx <= ww && sy <= wh;
                row[u * 3..u * 3 + 3].copy_from_slice(&bilinear(&work, sx - 0.5, sy - 0.5));
            }
        });

    let coverage = inside.iter().filter(|&&i| i).count() as f32 / inside.len() as f32;
    let image = RgbImage::from_raw(p.width, p.height, pixels).expect("taille cohérente");
    Aligned { image, inside, coverage, scale }
}

/// Échantillonnage bilinéaire, bords prolongés hors de l'image.
fn bilinear(img: &RgbImage, x: f32, y: f32) -> [u8; 3] {
    let (w, h) = (img.width() as i64, img.height() as i64);
    let (x0, y0) = (x.floor(), y.floor());
    let (fx, fy) = (x - x0, y - y0);
    let at = |xi: i64, yi: i64| img.get_pixel(xi.clamp(0, w - 1) as u32, yi.clamp(0, h - 1) as u32).0;
    let (x0, y0) = (x0 as i64, y0 as i64);
    let (a, b, c, d) = (at(x0, y0), at(x0 + 1, y0), at(x0, y0 + 1), at(x0 + 1, y0 + 1));
    let mut out = [0u8; 3];
    for i in 0..3 {
        let top = a[i] as f32 * (1.0 - fx) + b[i] as f32 * fx;
        let bottom = c[i] as f32 * (1.0 - fx) + d[i] as f32 * fx;
        out[i] = (top * (1.0 - fy) + bottom * fy).round().clamp(0.0, 255.0) as u8;
    }
    out
}
