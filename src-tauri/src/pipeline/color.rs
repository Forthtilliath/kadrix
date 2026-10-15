//! Uniformisation de la luminosité : luminance L* (Lab, D65) moyenne du
//! sujet ramenée vers une cible par une courbe gamma (noirs et blancs préservés).

use image::RgbImage;
use rayon::prelude::*;
use std::sync::OnceLock;

const GAMMA_MIN: f32 = 0.5;
const GAMMA_MAX: f32 = 2.0;
/// Poids minimal (en pixels opaques) pour que la mesure soit fiable.
const MIN_WEIGHT: f64 = 400.0;
const EPS: f32 = 216.0 / 24389.0;
const KAPPA: f32 = 24389.0 / 27.0;
const WHITE: [f32; 3] = [0.95047, 1.0, 1.08883];

pub struct LightResult {
    /// L* moyen mesuré avant correction.
    pub before: f32,
    /// Correction plafonnée (photo très sombre ou très claire).
    pub clamped: bool,
}

/// sRGB 8 bits → valeur linéaire.
fn linear() -> &'static [f32; 256] {
    static LUT: OnceLock<[f32; 256]> = OnceLock::new();
    LUT.get_or_init(|| {
        let mut table = [0f32; 256];
        for (i, v) in table.iter_mut().enumerate() {
            let c = i as f32 / 255.0;
            *v = if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) };
        }
        table
    })
}

fn encode(v: f32) -> f32 {
    let v = v.clamp(0.0, 1.0);
    if v <= 0.003_130_8 { v * 12.92 } else { 1.055 * v.powf(1.0 / 2.4) - 0.055 }
}

fn f(t: f32) -> f32 {
    if t > EPS { t.cbrt() } else { (KAPPA * t + 16.0) / 116.0 }
}

fn f_inv(t: f32) -> f32 {
    let t3 = t * t * t;
    if t3 > EPS { t3 } else { (116.0 * t - 16.0) / KAPPA }
}

fn to_lab(p: &[u8]) -> [f32; 3] {
    let lut = linear();
    let (r, g, b) = (lut[p[0] as usize], lut[p[1] as usize], lut[p[2] as usize]);
    let x = (0.412_456_4 * r + 0.357_576_1 * g + 0.180_437_5 * b) / WHITE[0];
    let y = 0.212_672_9 * r + 0.715_152_2 * g + 0.072_175 * b;
    let z = (0.019_333_9 * r + 0.119_192 * g + 0.950_304_1 * b) / WHITE[2];
    let (fx, fy, fz) = (f(x), f(y), f(z));
    [116.0 * fy - 16.0, 500.0 * (fx - fy), 200.0 * (fy - fz)]
}

/// Lab → sRGB (0-1).
fn from_lab(lab: [f32; 3]) -> [f32; 3] {
    let fy = (lab[0] + 16.0) / 116.0;
    let (fx, fz) = (fy + lab[1] / 500.0, fy - lab[2] / 200.0);
    let (x, y, z) = (f_inv(fx) * WHITE[0], f_inv(fy), f_inv(fz) * WHITE[2]);
    let r = 3.240_454_2 * x - 1.537_138_5 * y - 0.498_531_4 * z;
    let g = -0.969_266 * x + 1.876_010_8 * y + 0.041_556 * z;
    let b = 0.055_643_4 * x - 0.204_025_9 * y + 1.057_225_2 * z;
    [encode(r), encode(g), encode(b)]
}

pub fn to_float(img: &RgbImage) -> Vec<[f32; 3]> {
    img.as_raw()
        .chunks_exact(3)
        .map(|p| [p[0] as f32 / 255.0, p[1] as f32 / 255.0, p[2] as f32 / 255.0])
        .collect()
}

/// L* moyen des pixels opaques du sujet (et de `region` si fournie).
fn mean_lightness(img: &RgbImage, alpha: &[f32], region: Option<&[bool]>) -> Option<f32> {
    let (sum, weight) = img
        .as_raw()
        .par_chunks_exact(3)
        .zip(alpha.par_iter())
        .enumerate()
        .filter(|(i, (_, a))| **a >= 0.5 && region.is_none_or(|r| r[*i]))
        .map(|(_, (p, a))| (to_lab(p)[0] as f64 * *a as f64, *a as f64))
        .reduce(|| (0.0, 0.0), |x, y| (x.0 + y.0, x.1 + y.1));
    (weight >= MIN_WEIGHT).then(|| (sum / weight) as f32)
}

/// Pixels sRGB (0-1) corrigés ; `None` si la zone de mesure est trop petite.
pub fn normalize(
    img: &RgbImage,
    alpha: &[f32],
    region: Option<&[bool]>,
    target: f32,
) -> (Vec<[f32; 3]>, Option<LightResult>) {
    let Some(before) = mean_lightness(img, alpha, region) else {
        return (to_float(img), None);
    };
    let raw = (target / 100.0).ln() / (before.clamp(1.0, 99.0) / 100.0).ln();
    let gamma = raw.clamp(GAMMA_MIN, GAMMA_MAX);
    let pixels = img
        .as_raw()
        .par_chunks_exact(3)
        .zip(alpha.par_iter())
        .map(|(p, a)| {
            if *a <= 0.002 {
                // Invisible une fois le fond posé : inutile de convertir.
                return [p[0] as f32 / 255.0, p[1] as f32 / 255.0, p[2] as f32 / 255.0];
            }
            let mut lab = to_lab(p);
            lab[0] = 100.0 * (lab[0] / 100.0).max(0.0).powf(gamma);
            from_lab(lab)
        })
        .collect();
    (pixels, Some(LightResult { before, clamped: raw != gamma }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lab_round_trip() {
        for p in [[0u8, 0, 0], [255, 255, 255], [200, 120, 90], [30, 60, 200]] {
            let back = from_lab(to_lab(&p));
            for c in 0..3 {
                assert!((back[c] * 255.0 - p[c] as f32).abs() < 0.6, "{p:?} → {back:?}");
            }
        }
    }

    #[test]
    fn white_is_100() {
        assert!((to_lab(&[255, 255, 255])[0] - 100.0).abs() < 0.01);
    }
}
