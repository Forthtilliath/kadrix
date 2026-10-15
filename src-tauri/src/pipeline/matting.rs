//! Détourage BiRefNet (portrait) : renvoie l'opacité du sujet, de 0 à 1.

use super::onnx;
use anyhow::{bail, Result};
use image::{imageops, imageops::FilterType, ImageBuffer, Luma, RgbImage};
use ort::session::Session;
use ort::value::Tensor;
use std::path::Path;

const SIZE: usize = 1024;
const MEAN: [f32; 3] = [0.485, 0.456, 0.406];
const STD: [f32; 3] = [0.229, 0.224, 0.225];

pub struct Matting {
    session: Session,
}

impl Matting {
    pub fn new(path: &Path) -> Result<Self> {
        Ok(Self { session: onnx::session(path)? })
    }

    /// Opacité par pixel, à la taille de `img`.
    pub fn alpha(&mut self, img: &RgbImage) -> Result<Vec<f32>> {
        let small = imageops::resize(img, SIZE as u32, SIZE as u32, FilterType::Triangle);
        let plane = SIZE * SIZE;
        let mut data = vec![0f32; 3 * plane];
        for (i, p) in small.pixels().enumerate() {
            for c in 0..3 {
                data[c * plane + i] = (p[c] as f32 / 255.0 - MEAN[c]) / STD[c];
            }
        }
        let input = Tensor::from_array(([1usize, 3, SIZE, SIZE], data))?;
        let outputs = self.session.run(ort::inputs![input])?;
        let (_, raw) = outputs[0].try_extract_tensor::<f32>()?;
        if raw.len() < plane {
            bail!("sortie du modèle de détourage inattendue");
        }
        let mut mask = raw[..plane].to_vec();
        drop(outputs);

        // Selon l'export, la sortie est brute (logits) ou déjà entre 0 et 1.
        if mask.iter().any(|v| !(0.0..=1.0).contains(v)) {
            mask.iter_mut().for_each(|v| *v = 1.0 / (1.0 + (-*v).exp()));
        }
        let (min, max) = mask.iter().fold((f32::MAX, f32::MIN), |(lo, hi), &v| (lo.min(v), hi.max(v)));
        if max - min > 1e-6 {
            mask.iter_mut().for_each(|v| *v = (*v - min) / (max - min));
        }

        let buffer = ImageBuffer::<Luma<f32>, Vec<f32>>::from_raw(SIZE as u32, SIZE as u32, mask)
            .expect("taille cohérente");
        let resized = imageops::resize(&buffer, img.width(), img.height(), FilterType::Triangle);
        Ok(resized.into_raw().into_iter().map(|v| v.clamp(0.0, 1.0)).collect())
    }
}
