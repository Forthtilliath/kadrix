//! Export web (AVIF, WebP) et PNG transparent. Les fichiers produits ne
//! contiennent aucune métadonnée de la photo d'origine (EXIF, GPS…).

use super::Output;
use crate::params::ExportParams;
use anyhow::Result;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use image::codecs::{avif::AvifEncoder, jpeg::JpegEncoder};
use image::{imageops, ExtendedColorType, ImageEncoder, ImageFormat, RgbImage};
use std::fs;
use std::path::Path;

/// Vitesse d'encodage AVIF (1 = lent et compact, 10 = rapide).
const AVIF_SPEED: u8 = 6;

/// Écrit les formats demandés ; renvoie les noms des fichiers créés.
pub fn save(dir: &Path, stem: &str, out: &Output, ex: &ExportParams) -> Result<Vec<String>> {
    let img = &out.composed;
    let (w, h) = img.dimensions();
    let mut written = Vec::new();
    if ex.avif {
        let mut buffer = Vec::new();
        AvifEncoder::new_with_speed_quality(&mut buffer, AVIF_SPEED, ex.avif_quality)
            .write_image(img.as_raw(), w, h, ExtendedColorType::Rgb8)?;
        written.push(write(dir, &format!("{stem}.avif"), &buffer)?);
    }
    if ex.webp {
        let encoded = webp::Encoder::from_rgb(img.as_raw(), w, h).encode(ex.webp_quality as f32);
        written.push(write(dir, &format!("{stem}.webp"), &encoded)?);
    }
    if let Some(transparent) = &out.transparent {
        let name = format!("{stem}-detoure.png");
        transparent.save_with_format(dir.join(&name), ImageFormat::Png)?;
        written.push(name);
    }
    Ok(written)
}

fn write(dir: &Path, name: &str, bytes: &[u8]) -> Result<String> {
    fs::write(dir.join(name), bytes)?;
    Ok(name.to_string())
}

/// Miniature JPEG en data URL, pour l'affichage dans l'interface.
pub fn data_url(img: &RgbImage, max_side: u32) -> Result<String> {
    let (w, h) = img.dimensions();
    let k = (max_side as f32 / w.max(h) as f32).min(1.0);
    let (tw, th) = (((w as f32 * k) as u32).max(1), ((h as f32 * k) as u32).max(1));
    let thumb = imageops::thumbnail(img, tw, th);
    let mut buffer = Vec::new();
    JpegEncoder::new_with_quality(&mut buffer, 85).write_image(thumb.as_raw(), tw, th, ExtendedColorType::Rgb8)?;
    Ok(format!("data:image/jpeg;base64,{}", STANDARD.encode(buffer)))
}
