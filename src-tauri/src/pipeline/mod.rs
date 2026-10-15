//! Pipeline d'un portrait : détection → alignement → détourage →
//! luminosité → fond. L'export est fait par l'appelant.

mod align;
pub mod background;
mod checks;
mod color;
pub mod export;
mod face;
mod load;
mod matting;
mod onnx;

pub use checks::{Issue, Level};

use crate::params::{LightRegion, Params};
use anyhow::Result;
use background::Canvas;
use image::{RgbImage, RgbaImage};
use std::path::Path;

pub struct Models {
    face: face::FaceDetector,
    matting: matting::Matting,
}

impl Models {
    pub fn load(face_path: &Path, matting_path: &Path) -> Result<Self> {
        Ok(Self {
            face: face::FaceDetector::new(face_path)?,
            matting: matting::Matting::new(matting_path)?,
        })
    }
}

pub struct Output {
    pub composed: RgbImage,
    pub transparent: Option<RgbaImage>,
}

pub struct Outcome {
    pub output: Option<Output>,
    pub issues: Vec<Issue>,
}

/// Ne renvoie jamais d'erreur : tout problème devient un signalement.
pub fn process(models: &mut Models, path: &Path, params: &Params, canvas: &Canvas) -> Outcome {
    let mut issues = Vec::new();
    match run(models, path, params, canvas, &mut issues) {
        Ok(output) => Outcome { output, issues },
        Err(e) => {
            issues.push(Issue::error("failure", format!("Erreur : {e:#}")));
            Outcome { output: None, issues }
        }
    }
}

fn run(
    models: &mut Models,
    path: &Path,
    params: &Params,
    canvas: &Canvas,
    issues: &mut Vec<Issue>,
) -> Result<Option<Output>> {
    let source = load::open(path)?;
    let faces = models.face.detect_refined(&source)?;
    let Some(main) = faces.first() else {
        issues.push(Issue::error("noFace", "Aucun visage détecté.".into()));
        return Ok(None);
    };
    issues.extend(checks::face_checks(&faces));

    let (left, right) = main.eyes();
    let aligned = align::align(&source, left, right, params);
    issues.extend(checks::aligned_checks(&aligned, params));

    let mut alpha = models.matting.alpha(&aligned.image)?;
    // Hors de la photo d'origine (bords complétés) : jamais du sujet.
    for (a, inside) in alpha.iter_mut().zip(&aligned.inside) {
        if !inside {
            *a = 0.0;
        }
    }
    let subject_share = alpha.iter().sum::<f32>() / alpha.len() as f32;
    if subject_share < 0.03 {
        issues.push(Issue::error("emptyMatte", "Détourage vide : sujet non reconnu.".into()));
        return Ok(None);
    }

    let foreground = if params.light.enabled {
        let region = (params.light.region == LightRegion::Face).then(|| face_region(params));
        let (pixels, light) =
            color::normalize(&aligned.image, &alpha, region.as_deref(), params.light.target);
        issues.extend(checks::light_checks(light.as_ref()));
        pixels
    } else {
        color::to_float(&aligned.image)
    };

    let composed = background::composite(&foreground, &alpha, canvas);
    let transparent = params
        .export
        .png_transparent
        .then(|| background::transparent(&foreground, &alpha, params.width, params.height));
    Ok(Some(Output { composed, transparent }))
}

/// Ellipse couvrant le visage (sourcils → menton) dans l'image alignée.
fn face_region(p: &Params) -> Vec<bool> {
    let (w, h) = (p.width, p.height);
    let d = p.eye_distance;
    let (cx, cy) = (p.eye_x * w as f32, p.eye_y * h as f32 + 0.55 * d);
    let (rx, ry) = (1.0 * d, 1.45 * d);
    (0..h)
        .flat_map(|y| (0..w).map(move |x| (x, y)))
        .map(|(x, y)| {
            let nx = (x as f32 + 0.5 - cx) / rx;
            let ny = (y as f32 + 0.5 - cy) / ry;
            nx * nx + ny * ny <= 1.0
        })
        .collect()
}
