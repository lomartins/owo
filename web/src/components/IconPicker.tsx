import { For, type JSX } from "solid-js";
import { ICON_NAMES, COLOR_PALETTE } from "../lib/categoryIcons";
import { useI18n } from "../lib/i18n";

interface IconPickerProps {
  value: string | null;
  color: string | null;
  onChange: (icon: string, color: string) => void;
}

/**
 * Live preview + grid of icons + colour swatches.
 *
 * - Preview: 48x48 square that mirrors `.tx-icon` proportions; background uses
 *   the selected colour, glyph is white at 24px.
 * - Icon grid: 8 columns. Selected icon ringed with `--color-primary-600`.
 * - Colour row: pill swatches. Selected swatch ringed with `--color-primary-600`.
 *
 * Reuses `.tx-icon` for the per-button thumbnails so spacing/centering stays
 * identical to the rest of the app — backgrounds are overridden inline.
 */
export function IconPicker(props: IconPickerProps): JSX.Element {
  const { t } = useI18n();
  const icon = (): string => props.value ?? ICON_NAMES[0]!;
  const color = (): string => props.color ?? COLOR_PALETTE[0]!;

  return (
    <div class="flex flex-col gap-4">
      <div class="flex items-center gap-3">
        <span
          class="inline-grid place-items-center"
          style={{
            width: "48px",
            height: "48px",
            "border-radius": "var(--radius-md)",
            background: color(),
            color: "#ffffff",
            "flex-shrink": "0",
          }}
          aria-hidden
        >
          <span class="material-symbols-rounded" style={{ "font-size": "24px", "line-height": "1" }}>
            {icon()}
          </span>
        </span>
        <div class="min-w-0 flex-1">
          <p class="label" style={{ "margin-bottom": "2px" }}>{t("categoriesPage.pickIcon")}</p>
          <p class="meta truncate">{icon()}</p>
        </div>
      </div>

      <div>
        <div
          class="grid gap-2"
          style={{ "grid-template-columns": "repeat(8, minmax(0, 1fr))" }}
        >
          <For each={ICON_NAMES}>
            {(name) => {
              const selected = (): boolean => icon() === name;
              return (
                <button
                  type="button"
                  class="tx-icon"
                  aria-label={name}
                  aria-pressed={selected()}
                  onClick={() => props.onChange(name, color())}
                  style={{
                    background: selected() ? color() : "var(--color-surface-muted)",
                    color: selected() ? "#ffffff" : "var(--color-text-muted)",
                    border: selected()
                      ? "2px solid var(--color-primary-600)"
                      : "2px solid transparent",
                    cursor: "pointer",
                  }}
                >
                  <span class="material-symbols-rounded" style={{ "font-size": "18px" }}>
                    {name}
                  </span>
                </button>
              );
            }}
          </For>
        </div>
      </div>

      <div>
        <p class="label">{t("categoriesPage.pickColor")}</p>
        <div class="flex flex-wrap gap-2">
          <For each={COLOR_PALETTE}>
            {(hex) => {
              const selected = (): boolean => color().toLowerCase() === hex.toLowerCase();
              return (
                <button
                  type="button"
                  aria-label={hex}
                  aria-pressed={selected()}
                  onClick={() => props.onChange(icon(), hex)}
                  style={{
                    width: "28px",
                    height: "28px",
                    "border-radius": "999px",
                    background: hex,
                    border: selected()
                      ? "2px solid var(--color-primary-600)"
                      : "2px solid transparent",
                    "box-shadow": "var(--shadow-hairline)",
                    cursor: "pointer",
                  }}
                />
              );
            }}
          </For>
        </div>
      </div>
    </div>
  );
}
