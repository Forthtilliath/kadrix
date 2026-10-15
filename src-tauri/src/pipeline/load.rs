use anyhow::{Context, Result};
use image::{DynamicImage, ImageDecoder, ImageReader, Limits, RgbImage};
use std::path::Path;

/// Ouvre une image en respectant l'orientation EXIF (photos de téléphone),
/// avec des limites contre les fichiers piégés (dimensions, mémoire).
pub fn open(path: &Path) -> Result<RgbImage> {
    let mut reader = ImageReader::open(path)
        .context("lecture du fichier")?
        .with_guessed_format()?;
    let mut limits = Limits::default();
    limits.max_image_width = Some(20_000);
    limits.max_image_height = Some(20_000);
    limits.max_alloc = Some(1 << 30);
    reader.limits(limits);
    let mut decoder = reader.into_decoder().context("format d'image non reconnu")?;
    let orientation = decoder.orientation()?;
    let mut image = DynamicImage::from_decoder(decoder).context("décodage de l'image")?;
    image.apply_orientation(orientation);
    Ok(image.into_rgb8())
}
