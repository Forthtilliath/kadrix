//! Traitement d'un dossier entier (entrée → sortie, originaux intouchés).

use crate::params::Params;
use crate::pipeline::{self, background::Canvas, export, Issue, Models};
use crate::report::{self, BatchEvent, ItemReport};
use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

const EXTENSIONS: [&str; 7] = ["jpg", "jpeg", "png", "webp", "tif", "tiff", "bmp"];

pub fn is_image(path: &Path) -> bool {
    path.is_file()
        && path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| EXTENSIONS.contains(&e.to_ascii_lowercase().as_str()))
}

/// Images du dossier (sans les sous-dossiers), triées par nom.
pub fn list_images(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| is_image(p))
        .collect();
    files.sort();
    Ok(files)
}

/// Vérifie les dossiers ; la sortie est créée si besoin et ne peut jamais
/// être le dossier des originaux.
pub fn check_dirs(input: &str, output: &str) -> Result<(PathBuf, PathBuf), String> {
    let input = fs::canonicalize(input).map_err(|_| "Dossier d'entrée introuvable.".to_string())?;
    if !input.is_dir() {
        return Err("Le dossier d'entrée n'est pas un dossier.".into());
    }
    if output.trim().is_empty() {
        return Err("Choisissez un dossier de sortie.".into());
    }
    fs::create_dir_all(output).map_err(|e| format!("Création du dossier de sortie impossible : {e}"))?;
    let output = fs::canonicalize(output).map_err(|e| format!("Dossier de sortie inaccessible : {e}"))?;
    if output == input {
        return Err("Le dossier de sortie doit être différent du dossier des originaux.".into());
    }
    Ok((input, output))
}

pub fn run(
    models: &mut Models,
    input: &Path,
    output: &Path,
    params: &Params,
    cancel: &AtomicBool,
    emit: impl Fn(BatchEvent),
) -> Result<PathBuf> {
    let files = list_images(input)?;
    let canvas = Canvas::render(&params.background, params.width, params.height)?;
    emit(BatchEvent::Started { total: files.len() });

    let mut stems = Stems::default();
    let mut items = Vec::with_capacity(files.len());
    let mut cancelled = false;
    for (index, path) in files.iter().enumerate() {
        if cancel.load(Ordering::Relaxed) {
            cancelled = true;
            break;
        }
        let started = Instant::now();
        let outcome = pipeline::process(models, path, params, &canvas);
        let mut issues = outcome.issues;
        let (mut outputs, mut thumbnail) = (Vec::new(), None);
        if let Some(out) = outcome.output {
            match export::save(output, &stems.next(path), &out, &params.export) {
                Ok(names) => outputs = names,
                Err(e) => issues.push(Issue::error("export", format!("Écriture impossible : {e:#}"))),
            }
            thumbnail = export::data_url(&out.composed, 200).ok();
        }
        let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        let millis = started.elapsed().as_millis() as u64;
        let item = ItemReport::new(name, issues, outputs, thumbnail, millis);
        emit(BatchEvent::Item { index, item: item.clone() });
        items.push(ItemReport { thumbnail: None, ..item });
    }

    let report_path = report::write(input, output, params, &items, cancelled)?;
    emit(BatchEvent::Finished { report_path: report_path.display().to_string(), cancelled });
    Ok(report_path)
}

/// Noms de sortie uniques : « photo.jpg » et « photo.png » ne s'écrasent pas.
#[derive(Default)]
struct Stems(HashMap<String, usize>);

impl Stems {
    fn next(&mut self, path: &Path) -> String {
        let stem = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "photo".into());
        let count = self.0.entry(stem.to_lowercase()).or_insert(0);
        *count += 1;
        if *count == 1 { stem } else { format!("{stem}-{count}") }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stems_are_unique() {
        let mut stems = Stems::default();
        assert_eq!(stems.next(Path::new("a/Photo.jpg")), "Photo");
        assert_eq!(stems.next(Path::new("a/photo.png")), "photo-2");
        assert_eq!(stems.next(Path::new("a/autre.jpg")), "autre");
    }
}
