//! Détection YuNet : boîte du visage + 5 points (2 yeux, nez, 2 coins de bouche).

use super::onnx;
use anyhow::{anyhow, Result};
use image::{imageops, imageops::FilterType, RgbImage};
use ort::session::{Session, SessionOutputs};
use ort::value::Tensor;
use std::path::Path;

const SIZE: u32 = 640;
const STRIDES: [u32; 3] = [8, 16, 32];
const SCORE_MIN: f32 = 0.6;
const NMS_IOU: f32 = 0.3;

#[derive(Debug, Clone, Copy)]
pub struct Face {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub score: f32,
    marks: [[f32; 2]; 5],
}

impl Face {
    /// (œil à gauche de l'image, œil à droite de l'image).
    pub fn eyes(&self) -> ([f32; 2], [f32; 2]) {
        let (a, b) = (self.marks[0], self.marks[1]);
        if a[0] <= b[0] { (a, b) } else { (b, a) }
    }

    pub fn nose(&self) -> [f32; 2] {
        self.marks[2]
    }

    pub fn area(&self) -> f32 {
        self.w * self.h
    }

    fn center(&self) -> [f32; 2] {
        [self.x + self.w / 2.0, self.y + self.h / 2.0]
    }

    fn shifted(mut self, dx: f32, dy: f32) -> Self {
        self.x += dx;
        self.y += dy;
        for m in &mut self.marks {
            m[0] += dx;
            m[1] += dy;
        }
        self
    }
}

pub struct FaceDetector {
    session: Session,
}

impl FaceDetector {
    pub fn new(path: &Path) -> Result<Self> {
        Ok(Self { session: onnx::session(path)? })
    }

    /// Détecte sur l'image entière, puis affine sur un recadrage autour du
    /// visage principal (points des yeux bien plus précis sur les grandes photos).
    pub fn detect_refined(&mut self, img: &RgbImage) -> Result<Vec<Face>> {
        let mut faces = self.detect(img)?;
        let Some(main) = faces.first().copied() else { return Ok(faces) };
        let (w, h) = (img.width() as f32, img.height() as f32);
        let side = main.w.max(main.h) * 2.2;
        let [cx, cy] = main.center();
        let x0 = (cx - side / 2.0).max(0.0).floor();
        let y0 = (cy - side / 2.0).max(0.0).floor();
        let x1 = (cx + side / 2.0).min(w).ceil();
        let y1 = (cy + side / 2.0).min(h).ceil();
        let (cw, ch) = (x1 - x0, y1 - y0);
        // Inutile si le visage occupe déjà presque toute la photo.
        if cw < 64.0 || ch < 64.0 || cw.max(ch) > 0.8 * w.max(h) {
            return Ok(faces);
        }
        let crop = imageops::crop_imm(img, x0 as u32, y0 as u32, cw as u32, ch as u32).to_image();
        let best = self
            .detect(&crop)?
            .into_iter()
            .map(|f| f.shifted(x0, y0))
            .filter(|f| iou(f, &main) > 0.5)
            .max_by(|a, b| a.score.total_cmp(&b.score));
        if let Some(refined) = best {
            faces[0] = refined;
        }
        Ok(faces)
    }

    /// Visages triés du plus grand au plus petit.
    pub fn detect(&mut self, img: &RgbImage) -> Result<Vec<Face>> {
        let (w, h) = img.dimensions();
        let k = SIZE as f32 / w.max(h) as f32;
        let rw = ((w as f32 * k).round() as u32).clamp(1, SIZE);
        let rh = ((h as f32 * k).round() as u32).clamp(1, SIZE);
        let small = imageops::resize(img, rw, rh, FilterType::Triangle);

        // Entrée BGR, valeurs 0-255, NCHW, image calée en haut à gauche.
        let plane = (SIZE * SIZE) as usize;
        let mut data = vec![0f32; 3 * plane];
        for (x, y, p) in small.enumerate_pixels() {
            let i = (y * SIZE + x) as usize;
            data[i] = p[2] as f32;
            data[plane + i] = p[1] as f32;
            data[2 * plane + i] = p[0] as f32;
        }
        let input = Tensor::from_array(([1usize, 3, SIZE as usize, SIZE as usize], data))?;
        let outputs = self.session.run(ort::inputs![input])?;

        let mut faces = Vec::new();
        for stride in STRIDES {
            let cls = tensor(&outputs, &format!("cls_{stride}"))?;
            let obj = tensor(&outputs, &format!("obj_{stride}"))?;
            let bbox = tensor(&outputs, &format!("bbox_{stride}"))?;
            let kps = tensor(&outputs, &format!("kps_{stride}"))?;
            decode(stride, &cls, &obj, &bbox, &kps, k, &mut faces);
        }
        Ok(nms(faces))
    }
}

fn tensor(outputs: &SessionOutputs, name: &str) -> Result<Vec<f32>> {
    let value = outputs.get(name).ok_or_else(|| anyhow!("sortie YuNet absente : {name}"))?;
    let (_, data) = value.try_extract_tensor::<f32>()?;
    Ok(data.to_vec())
}

fn decode(stride: u32, cls: &[f32], obj: &[f32], bbox: &[f32], kps: &[f32], k: f32, out: &mut Vec<Face>) {
    let cols = (SIZE / stride) as usize;
    let s = stride as f32;
    for idx in 0..(cols * cols).min(cls.len()) {
        let score = (cls[idx].clamp(0.0, 1.0) * obj[idx].clamp(0.0, 1.0)).sqrt();
        if score < SCORE_MIN {
            continue;
        }
        let (r, c) = ((idx / cols) as f32, (idx % cols) as f32);
        let b = &bbox[idx * 4..idx * 4 + 4];
        let (cx, cy) = ((c + b[0]) * s, (r + b[1]) * s);
        let (bw, bh) = (b[2].exp() * s, b[3].exp() * s);
        let mut marks = [[0f32; 2]; 5];
        for (m, mark) in marks.iter_mut().enumerate() {
            let p = &kps[idx * 10 + 2 * m..idx * 10 + 2 * m + 2];
            *mark = [(p[0] + c) * s / k, (p[1] + r) * s / k];
        }
        out.push(Face {
            x: (cx - bw / 2.0) / k,
            y: (cy - bh / 2.0) / k,
            w: bw / k,
            h: bh / k,
            score,
            marks,
        });
    }
}

fn iou(a: &Face, b: &Face) -> f32 {
    let x0 = a.x.max(b.x);
    let y0 = a.y.max(b.y);
    let x1 = (a.x + a.w).min(b.x + b.w);
    let y1 = (a.y + a.h).min(b.y + b.h);
    let inter = (x1 - x0).max(0.0) * (y1 - y0).max(0.0);
    inter / (a.area() + b.area() - inter).max(1e-6)
}

fn nms(mut faces: Vec<Face>) -> Vec<Face> {
    faces.sort_by(|a, b| b.score.total_cmp(&a.score));
    let mut kept: Vec<Face> = Vec::new();
    for face in faces {
        if kept.iter().all(|k| iou(k, &face) < NMS_IOU) {
            kept.push(face);
        }
    }
    kept.sort_by(|a, b| b.area().total_cmp(&a.area()));
    kept
}
