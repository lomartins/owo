import { Show, type JSX } from "solid-js";

interface Props {
  icon: string | null;
  color: string | null;
  /** Letter to render when icon is absent. */
  fallback: string;
  /** Tile edge in px. Defaults to 32 (matches `.tx-icon`). */
  size?: number;
  /** Glyph font-size in px. Defaults to ~58% of `size`. */
  glyphSize?: number;
}

/**
 * Category thumbnail. When `icon` is set, paints a coloured tile with the
 * Material Symbol in white; otherwise falls back to the monogram letter on
 * the default `.tx-icon` pink background.
 *
 * At the default size this uses the `.tx-icon` class to share centering and
 * radius with the rest of the app. For smaller chip-sized thumbs (16-20 px)
 * the caller can pass `size` and `glyphSize` to scale inline.
 */
export function CategoryThumb(props: Props): JSX.Element {
  const size = (): number => props.size ?? 32;
  const glyph = (): number => props.glyphSize ?? Math.round(size() * 0.58);
  const isDefault = (): boolean => size() === 32;

  return (
    <Show
      when={props.icon}
      fallback={
        <span
          class={isDefault() ? "tx-icon" : undefined}
          aria-hidden
          style={
            isDefault()
              ? undefined
              : {
                  width: `${size()}px`,
                  height: `${size()}px`,
                  "border-radius": "var(--radius-xs)",
                  background: "var(--color-primary-50)",
                  color: "var(--color-primary-600)",
                  display: "grid",
                  "place-items": "center",
                  "font-size": `${glyph()}px`,
                  "font-weight": "500",
                  "line-height": "1",
                  "flex-shrink": "0",
                }
          }
        >
          {props.fallback}
        </span>
      }
    >
      <span
        class={isDefault() ? "tx-icon" : undefined}
        aria-hidden
        style={
          isDefault()
            ? {
                background: props.color ?? "var(--color-primary-50)",
                color: "#ffffff",
              }
            : {
                width: `${size()}px`,
                height: `${size()}px`,
                "border-radius": "var(--radius-xs)",
                background: props.color ?? "var(--color-primary-50)",
                color: "#ffffff",
                display: "grid",
                "place-items": "center",
                "flex-shrink": "0",
              }
        }
      >
        <span class="material-symbols-rounded" style={{ "font-size": `${glyph()}px`, "line-height": "1" }}>
          {props.icon}
        </span>
      </span>
    </Show>
  );
}
