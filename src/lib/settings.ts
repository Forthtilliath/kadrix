import { z } from "zod";

const STORAGE_KEY = "kadrix.settings.v1";

const hexColor = z.string().regex(/^#[0-9a-fA-F]{6}$/);
const path = z.string().max(4096);

export const paramsSchema = z.object({
  width: z.number().int().min(128).max(4096),
  height: z.number().int().min(128).max(4096),
  eyeDistance: z.number().min(16).max(2400),
  eyeX: z.number().min(0.2).max(0.8),
  eyeY: z.number().min(0.15).max(0.75),
  background: z.discriminatedUnion("kind", [
    z.object({ kind: z.literal("color"), color: hexColor }),
    z.object({ kind: z.literal("gradient"), top: hexColor, bottom: hexColor }),
    z.object({ kind: z.literal("image"), path }),
  ]),
  light: z.object({
    enabled: z.boolean(),
    target: z.number().min(10).max(95),
    region: z.enum(["face", "subject"]),
  }),
  export: z.object({
    avif: z.boolean(),
    avifQuality: z.number().int().min(1).max(100),
    webp: z.boolean(),
    webpQuality: z.number().int().min(1).max(100),
    pngTransparent: z.boolean(),
  }),
});

/** Réglages mémorisés : relus et vérifiés champ par champ à chaque lancement. */
export const settingsSchema = z.object({
  inputDir: path,
  outputDir: path,
  params: paramsSchema,
});

export type Settings = z.infer<typeof settingsSchema>;
export type Params = Settings["params"];
export type BackgroundParams = Params["background"];

export const DEFAULT_PARAMS: Params = {
  width: 600,
  height: 750,
  eyeDistance: 120,
  eyeX: 0.5,
  eyeY: 0.4,
  background: { kind: "gradient", top: "#e8edf2", bottom: "#c3ccd6" },
  light: { enabled: true, target: 62, region: "face" },
  export: { avif: true, avifQuality: 55, webp: true, webpQuality: 78, pngTransparent: false },
};

export const DEFAULT_SETTINGS: Settings = { inputDir: "", outputDir: "", params: DEFAULT_PARAMS };

/** Données illisibles ou hors bornes : réglages par défaut, rien d'appliqué. */
export function loadSettings(): Settings {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return DEFAULT_SETTINGS;
    const parsed = settingsSchema.safeParse(JSON.parse(raw));
    return parsed.success ? parsed.data : DEFAULT_SETTINGS;
  } catch {
    return DEFAULT_SETTINGS;
  }
}

export function saveSettings(settings: Settings): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
  } catch {
    // Stockage indisponible : les réglages ne seront simplement pas mémorisés.
  }
}
