import { open } from "@tauri-apps/plugin-dialog";
import type { BackgroundParams, Params, Settings } from "../lib/settings";
import { ColorField, NumberField, PathField, Section, Segmented, Toggle } from "./ui";

interface Props {
  settings: Settings;
  onChange: (s: Settings) => void;
}

async function pickFolder(): Promise<string | null> {
  const picked = await open({ directory: true, multiple: false });
  return typeof picked === "string" ? picked : null;
}

export function SettingsPanel({ settings, onChange }: Props) {
  const p = settings.params;
  const setParams = (patch: Partial<Params>) => onChange({ ...settings, params: { ...p, ...patch } });

  return (
    <aside className="w-80 shrink-0 overflow-y-auto border-r border-zinc-200 bg-white dark:border-zinc-800 dark:bg-zinc-900/40">
      <Section title="Dossiers">
        <PathField
          label="Photos d'origine"
          value={settings.inputDir}
          placeholder="Aucun dossier"
          onPick={async () => {
            const dir = await pickFolder();
            if (dir) onChange({ ...settings, inputDir: dir });
          }}
        />
        <PathField
          label="Dossier de sortie"
          value={settings.outputDir}
          placeholder="Aucun dossier"
          onPick={async () => {
            const dir = await pickFolder();
            if (dir) onChange({ ...settings, outputDir: dir });
          }}
        />
        <p className="text-xs text-zinc-500">Les originaux ne sont jamais modifiés.</p>
      </Section>

      <Section title="Cadrage">
        <NumberField label="Largeur" value={p.width} min={128} max={4096} suffix="px" onChange={(width) => setParams({ width })} />
        <NumberField label="Hauteur" value={p.height} min={128} max={4096} suffix="px" onChange={(height) => setParams({ height })} />
        <NumberField
          label="Écart entre les yeux"
          value={p.eyeDistance}
          min={16}
          max={Math.round(p.width * 0.6)}
          suffix="px"
          onChange={(eyeDistance) => setParams({ eyeDistance })}
        />
        <NumberField
          label="Yeux : position horizontale"
          value={Math.round(p.eyeX * 100)}
          min={20}
          max={80}
          suffix="%"
          onChange={(v) => setParams({ eyeX: v / 100 })}
        />
        <NumberField
          label="Yeux : hauteur depuis le haut"
          value={Math.round(p.eyeY * 100)}
          min={15}
          max={75}
          suffix="%"
          onChange={(v) => setParams({ eyeY: v / 100 })}
        />
      </Section>

      <BackgroundSection value={p.background} onChange={(background) => setParams({ background })} />

      <Section title="Luminosité">
        <Toggle label="Uniformiser la luminosité" checked={p.light.enabled} onChange={(enabled) => setParams({ light: { ...p.light, enabled } })} />
        {p.light.enabled && (
          <>
            <NumberField
              label="Luminance cible (L*)"
              value={p.light.target}
              min={10}
              max={95}
              onChange={(target) => setParams({ light: { ...p.light, target } })}
            />
            <Segmented
              value={p.light.region}
              options={[
                { value: "face", label: "Mesure : visage" },
                { value: "subject", label: "Sujet entier" },
              ]}
              onChange={(region) => setParams({ light: { ...p.light, region } })}
            />
          </>
        )}
      </Section>

      <Section title="Export">
        <Toggle label="AVIF" checked={p.export.avif} onChange={(avif) => setParams({ export: { ...p.export, avif } })} />
        {p.export.avif && (
          <NumberField label="Qualité AVIF" value={p.export.avifQuality} min={1} max={100} onChange={(avifQuality) => setParams({ export: { ...p.export, avifQuality } })} />
        )}
        <Toggle label="WebP" checked={p.export.webp} onChange={(webp) => setParams({ export: { ...p.export, webp } })} />
        {p.export.webp && (
          <NumberField label="Qualité WebP" value={p.export.webpQuality} min={1} max={100} onChange={(webpQuality) => setParams({ export: { ...p.export, webpQuality } })} />
        )}
        <Toggle
          label="PNG détouré (fond transparent)"
          checked={p.export.pngTransparent}
          onChange={(pngTransparent) => setParams({ export: { ...p.export, pngTransparent } })}
        />
      </Section>
    </aside>
  );
}

function BackgroundSection({ value, onChange }: { value: BackgroundParams; onChange: (b: BackgroundParams) => void }) {
  const switchKind = (kind: BackgroundParams["kind"]) => {
    if (kind === value.kind) return;
    if (kind === "color") onChange({ kind, color: "#e8edf2" });
    if (kind === "gradient") onChange({ kind, top: "#e8edf2", bottom: "#c3ccd6" });
    if (kind === "image") onChange({ kind, path: "" });
  };

  return (
    <Section title="Fond">
      <Segmented
        value={value.kind}
        options={[
          { value: "color", label: "Couleur" },
          { value: "gradient", label: "Dégradé" },
          { value: "image", label: "Image" },
        ]}
        onChange={switchKind}
      />
      {value.kind === "color" && <ColorField label="Couleur" value={value.color} onChange={(color) => onChange({ ...value, color })} />}
      {value.kind === "gradient" && (
        <>
          <ColorField label="Haut" value={value.top} onChange={(top) => onChange({ ...value, top })} />
          <ColorField label="Bas" value={value.bottom} onChange={(bottom) => onChange({ ...value, bottom })} />
        </>
      )}
      {value.kind === "image" && (
        <PathField
          label="Image de fond"
          value={value.path}
          placeholder="Aucune image"
          onPick={async () => {
            const picked = await open({
              multiple: false,
              filters: [{ name: "Images", extensions: ["jpg", "jpeg", "png", "webp"] }],
            });
            if (typeof picked === "string") onChange({ kind: "image", path: picked });
          }}
        />
      )}
    </Section>
  );
}
