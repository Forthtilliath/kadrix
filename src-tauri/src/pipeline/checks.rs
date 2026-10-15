//! Repérage des photos problématiques : elles sont signalées, jamais
//! traitées en silence. Les seuils sont à ajuster sur un lot réel.

use super::align::Aligned;
use super::color::LightResult;
use super::face::Face;
use crate::params::Params;
use serde::Serialize;

/// Variance du laplacien sous laquelle le visage est jugé flou.
const BLUR_MIN: f32 = 30.0;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Level {
    /// Photo non traitée.
    Error,
    /// Photo traitée, à vérifier à l'œil.
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Issue {
    pub level: Level,
    pub code: &'static str,
    pub message: String,
}

impl Issue {
    pub fn error(code: &'static str, message: String) -> Self {
        Self { level: Level::Error, code, message }
    }
    fn warning(code: &'static str, message: String) -> Self {
        Self { level: Level::Warning, code, message }
    }
    fn info(code: &'static str, message: String) -> Self {
        Self { level: Level::Info, code, message }
    }
}

/// `faces` est non vide, trié du plus grand au plus petit.
pub fn face_checks(faces: &[Face]) -> Vec<Issue> {
    let mut out = Vec::new();
    let main = &faces[0];
    let others = faces[1..].iter().filter(|f| f.area() > main.area() * 0.2).count();
    if others > 0 {
        out.push(Issue::warning(
            "multipleFaces",
            format!("{others} autre(s) visage(s) détecté(s) : le plus grand a été utilisé."),
        ));
    }
    if main.score < 0.8 {
        out.push(Issue::warning(
            "lowConfidence",
            format!(
                "Détection peu sûre ({:.0} %) : visage en partie caché, lunettes de soleil, contre-jour… ?",
                main.score * 100.0
            ),
        ));
    }
    let (l, r) = main.eyes();
    let d = ((r[0] - l[0]).powi(2) + (r[1] - l[1]).powi(2)).sqrt().max(1.0);
    let axis = [(r[0] - l[0]) / d, (r[1] - l[1]) / d];
    let mid = [(l[0] + r[0]) / 2.0, (l[1] + r[1]) / 2.0];
    let nose = main.nose();
    // Décalage du nez le long de l'axe des yeux : ~0 de face, grand de profil.
    let offset = ((nose[0] - mid[0]) * axis[0] + (nose[1] - mid[1]) * axis[1]) / d;
    if offset.abs() > 0.25 || d < main.w * 0.28 {
        out.push(Issue::warning(
            "turned",
            "Visage tourné (trois-quarts ou profil) : cadrage approximatif.".into(),
        ));
    }
    let tilt = axis[1].atan2(axis[0]).to_degrees();
    if tilt.abs() > 12.0 {
        out.push(Issue::info("tilted", format!("Tête penchée de {:.0}° : redressée.", tilt.abs())));
    }
    out
}

pub fn aligned_checks(al: &Aligned, p: &Params) -> Vec<Issue> {
    let mut out = Vec::new();
    if al.scale > 1.25 {
        out.push(Issue::warning(
            "lowResolution",
            format!("Photo de faible résolution : visage agrandi ×{:.1}, rendu flou possible.", al.scale),
        ));
    }
    if al.coverage < 0.97 {
        out.push(Issue::warning(
            "outOfFrame",
            format!(
                "Photo trop serrée : {:.0} % du cadre est hors de la photo (bords coupés).",
                (1.0 - al.coverage) * 100.0
            ),
        ));
    }

    let gray = Gray::new(al);
    let (ox, oy, d) = (p.eye_x * p.width as f32, p.eye_y * p.height as f32, p.eye_distance);
    let sharpness = gray.laplacian_variance([ox - d, oy - 0.6 * d, ox + d, oy + 1.6 * d]);
    if sharpness.is_some_and(|v| v < BLUR_MIN) {
        out.push(Issue::warning("blurry", "Visage flou (mise au point, bougé ou compression).".into()));
    }

    // Lunettes de soleil : zones des yeux bien plus sombres que les joues.
    let patch = |cx: f32, cy: f32, r: f32| gray.mean([cx - r, cy - r, cx + r, cy + r]);
    let eyes = patch(ox - d / 2.0, oy, 0.18 * d).zip(patch(ox + d / 2.0, oy, 0.18 * d));
    let cheeks = patch(ox - 0.45 * d, oy + 0.75 * d, 0.15 * d).zip(patch(ox + 0.45 * d, oy + 0.75 * d, 0.15 * d));
    if let (Some((el, er)), Some((cl, cr))) = (eyes, cheeks) {
        let (eyes, cheeks) = ((el + er) / 2.0, (cl + cr) / 2.0);
        if eyes < 70.0 && eyes < 0.5 * cheeks {
            out.push(Issue::warning("darkEyes", "Yeux très sombres : lunettes de soleil ?".into()));
        }
    }
    out
}

pub fn light_checks(light: Option<&LightResult>) -> Vec<Issue> {
    match light {
        None => vec![Issue::warning(
            "lightSkipped",
            "Zone de mesure trop petite : luminosité non ajustée.".into(),
        )],
        Some(l) if l.clamped => vec![Issue::warning(
            "lightClamped",
            format!("Photo très sombre ou très claire (L* {:.0}) : correction limitée.", l.before),
        )],
        Some(_) => Vec::new(),
    }
}

/// Niveaux de gris (0-255) de l'image alignée.
struct Gray {
    w: usize,
    h: usize,
    px: Vec<f32>,
}

impl Gray {
    fn new(al: &Aligned) -> Self {
        let px = al
            .image
            .pixels()
            .map(|p| 0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32)
            .collect();
        Self { w: al.image.width() as usize, h: al.image.height() as usize, px }
    }

    /// Rectangle [x0, y0, x1, y1] ramené dans l'image, marge de 1 px.
    fn clip(&self, r: [f32; 4]) -> Option<(usize, usize, usize, usize)> {
        let x0 = r[0].max(1.0) as usize;
        let y0 = r[1].max(1.0) as usize;
        let x1 = (r[2].min((self.w - 1) as f32)) as usize;
        let y1 = (r[3].min((self.h - 1) as f32)) as usize;
        (x1 > x0 + 2 && y1 > y0 + 2).then_some((x0, y0, x1, y1))
    }

    fn mean(&self, r: [f32; 4]) -> Option<f32> {
        let (x0, y0, x1, y1) = self.clip(r)?;
        let sum: f32 = (y0..y1).flat_map(|y| (x0..x1).map(move |x| (x, y))).map(|(x, y)| self.px[y * self.w + x]).sum();
        Some(sum / ((x1 - x0) * (y1 - y0)) as f32)
    }

    fn laplacian_variance(&self, r: [f32; 4]) -> Option<f32> {
        let (x0, y0, x1, y1) = self.clip(r)?;
        let at = |x: usize, y: usize| self.px[y * self.w + x];
        let values: Vec<f32> = (y0..y1)
            .flat_map(|y| (x0..x1).map(move |x| (x, y)))
            .map(|(x, y)| 4.0 * at(x, y) - at(x - 1, y) - at(x + 1, y) - at(x, y - 1) - at(x, y + 1))
            .collect();
        let n = values.len() as f32;
        let mean = values.iter().sum::<f32>() / n;
        Some(values.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / n)
    }
}
