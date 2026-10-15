# Kadrix

Application de bureau (Windows) qui traite un dossier de portraits pour obtenir un trombinoscope uniforme, prêt pour le web.

## Pipeline

1. **Détection** du visage et des yeux : YuNet (ONNX), avec une seconde passe sur un recadrage autour du visage pour gagner en précision.
2. **Normalisation** : mise à l'échelle sur l'écart entre les yeux et redressement de la tête.
3. **Alignement et recadrage** : le milieu des yeux tombe toujours au même point du cadre.
4. **Détourage** : BiRefNet entraîné sur des portraits (ONNX).
5. **Luminosité** : la luminance L* moyenne (Lab), mesurée sur le visage ou sur tout le sujet, est ramenée vers une cible commune.
6. **Fond** : couleur unie, dégradé ou image.
7. **Export** : AVIF et WebP, plus un PNG détouré en option, et un rapport `rapport-kadrix.json`.

Les photos à problème (aucun visage, visage tourné, flou, yeux très sombres, photo trop petite ou trop serrée…) sont signalées dans l'interface et dans le rapport. Elles ne passent jamais en silence.

Les originaux ne sont jamais modifiés : le dossier de sortie doit être différent du dossier d'entrée. Les fichiers exportés ne contiennent aucune métadonnée de la photo d'origine (EXIF, GPS).

## Stack

- Front : React, TypeScript, Tailwind CSS, Vite
- Cœur : Rust (Tauri 2), ONNX Runtime (`ort`), `image`, `rayon`

## Prérequis

- Node.js et pnpm
- Rust (via rustup) et les outils de compilation C++ de Visual Studio (charge de travail « Développement Desktop en C++ »)

## Développement

```powershell
pnpm install
pnpm tauri dev      # lance l'application
pnpm tauri build    # installateur NSIS
cd src-tauri; cargo test
```

## Modèles

Les modèles sont téléchargés au premier lancement, depuis l'application (environ 1 Go), dans le dossier de données de l'app. Chaque fichier est vérifié par son empreinte SHA-256 au téléchargement, puis à chaque chargement.

| Modèle | Rôle | Licence |
| --- | --- | --- |
| [YuNet](https://huggingface.co/opencv/face_detection_yunet) (2023mar) | visage + yeux | MIT |
| [BiRefNet portrait](https://huggingface.co/onnx-community/BiRefNet-portrait-ONNX) | détourage | MIT |

## RGPD

Publier la photo d'un membre sur le web exige son consentement : à recueillir avant toute mise en ligne.
