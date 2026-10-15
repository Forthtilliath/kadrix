//! Modèles ONNX : téléchargés au premier lancement, jamais utilisés sans
//! vérification de leur empreinte SHA-256 (épinglée ici, révision figée).

use anyhow::{bail, Context, Result};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub struct ModelSpec {
    pub id: &'static str,
    pub label: &'static str,
    pub license: &'static str,
    pub file_name: &'static str,
    pub url: &'static str,
    pub sha256: &'static str,
    pub size: u64,
}

/// YuNet (OpenCV Zoo) : visage + 5 points (yeux, nez, coins de la bouche).
pub const FACE: ModelSpec = ModelSpec {
    id: "yunet",
    label: "Détection du visage (YuNet)",
    license: "MIT",
    file_name: "face_detection_yunet_2023mar.onnx",
    url: "https://huggingface.co/opencv/face_detection_yunet/resolve/3cc26e7f1014a5ee5d74a42acee58bafc9d0a310/face_detection_yunet_2023mar.onnx",
    sha256: "8f2383e4dd3cfbb4553ea8718107fc0423210dc964f9f4280604804ed2552fa4",
    size: 232_589,
};

/// BiRefNet entraîné sur des portraits (bon sur les cheveux).
pub const MATTING: ModelSpec = ModelSpec {
    id: "birefnet-portrait",
    label: "Détourage (BiRefNet portrait)",
    license: "MIT",
    file_name: "birefnet-portrait.onnx",
    url: "https://huggingface.co/onnx-community/BiRefNet-portrait-ONNX/resolve/dd7167f6a8b54ff7efc29a4c988938d79866464f/onnx/model.onnx",
    sha256: "1ba1c8ff5a7bbfadc8d8d13fb11d7be793f91f23d9d466549e37a854f6668f99",
    size: 972_666_916,
};

pub const ALL: [&ModelSpec; 2] = [&FACE, &MATTING];

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    pub id: &'static str,
    pub label: &'static str,
    pub license: &'static str,
    pub size_bytes: u64,
    pub present: bool,
}

pub fn path(dir: &Path, spec: &ModelSpec) -> PathBuf {
    dir.join(spec.file_name)
}

/// Présence rapide (taille seulement) ; l'empreinte est vérifiée au chargement.
pub fn is_present(dir: &Path, spec: &ModelSpec) -> bool {
    fs::metadata(path(dir, spec)).is_ok_and(|m| m.len() == spec.size)
}

pub fn status(dir: &Path) -> Vec<ModelInfo> {
    ALL.iter()
        .map(|spec| ModelInfo {
            id: spec.id,
            label: spec.label,
            license: spec.license,
            size_bytes: spec.size,
            present: is_present(dir, spec),
        })
        .collect()
}

/// Télécharge un modèle dans `dir`, en vérifiant taille et empreinte avant de
/// le mettre en place. `progress` reçoit le nombre d'octets reçus.
pub async fn download(spec: &ModelSpec, dir: &Path, mut progress: impl FnMut(u64)) -> Result<()> {
    fs::create_dir_all(dir)?;
    let part = dir.join(format!("{}.part", spec.file_name));
    let client = reqwest::Client::builder().https_only(true).build()?;
    let mut response = client.get(spec.url).send().await?.error_for_status()?;
    let mut file = File::create(&part).context("création du fichier temporaire")?;
    let mut hasher = Sha256::new();
    let (mut received, mut last_report) = (0u64, 0u64);
    while let Some(chunk) = response.chunk().await? {
        received += chunk.len() as u64;
        if received > spec.size {
            drop(file);
            let _ = fs::remove_file(&part);
            bail!("fichier plus gros que prévu, téléchargement interrompu");
        }
        hasher.update(&chunk);
        file.write_all(&chunk)?;
        if received - last_report >= 1 << 20 || received == spec.size {
            progress(received);
            last_report = received;
        }
    }
    file.sync_all()?;
    drop(file);
    let digest = to_hex(&hasher.finalize());
    if received != spec.size || digest != spec.sha256 {
        let _ = fs::remove_file(&part);
        bail!("empreinte SHA-256 inattendue : fichier refusé");
    }
    fs::rename(&part, path(dir, spec))?;
    Ok(())
}

/// Recalcule l'empreinte d'un modèle déjà présent (fichier altéré = refusé).
pub fn verify(dir: &Path, spec: &ModelSpec) -> Result<PathBuf> {
    let file_path = path(dir, spec);
    let mut file = File::open(&file_path)
        .with_context(|| format!("modèle manquant : {}", spec.label))?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 1 << 20];
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    if to_hex(&hasher.finalize()) != spec.sha256 {
        bail!("le fichier du modèle « {} » a été modifié : supprimez-le et retéléchargez-le", spec.label);
    }
    Ok(file_path)
}

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
