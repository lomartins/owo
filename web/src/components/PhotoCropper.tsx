import { createSignal, onCleanup, onMount, type JSX } from "solid-js";
import { useI18n } from "../lib/i18n";

interface Props {
  /** Source image data URL (typically from FileReader.readAsDataURL). */
  src: string;
  /** Called with a SQUARE JPEG data URL of the cropped region. */
  onConfirm: (croppedDataUrl: string) => void;
  /** Called when the user dismisses the cropper. */
  onCancel: () => void;
  /** Output dimension in px (square). Defaults to 512. */
  size?: number;
}

/** Side of the square viewport on desktop. Mobile breakpoint goes full-bleed via CSS. */
const STAGE_PX = 320;

/** Zoom bounds applied on top of the cover-fit base scale. */
const MIN_ZOOM = 1;
const MAX_ZOOM = 4;

interface Pointer {
  id: number;
  x: number;
  y: number;
}

/**
 * PhotoCropper — drag + pinch / wheel zoom inside a circular mask, exports a
 * SQUARE JPEG data URL. The image is always constrained to fully cover the
 * viewport so no transparent edge can leak into the output.
 *
 * The transform stored in state is `(offsetX, offsetY, zoom)` where:
 *   - `zoom = 1` means the image fits the viewport edge-to-edge in cover mode.
 *   - `(offsetX, offsetY)` are pixel translations of the image center relative
 *     to the viewport center, in viewport (stage) coordinates.
 */
export function PhotoCropper(props: Props): JSX.Element {
  const { t } = useI18n();

  const [zoom, setZoom] = createSignal(1);
  const [offsetX, setOffsetX] = createSignal(0);
  const [offsetY, setOffsetY] = createSignal(0);
  /** Natural image dimensions, once loaded. */
  const [imgW, setImgW] = createSignal(0);
  const [imgH, setImgH] = createSignal(0);
  const [loaded, setLoaded] = createSignal(false);
  const [busy, setBusy] = createSignal(false);

  let stageEl: HTMLDivElement | undefined;
  let confirmBtn: HTMLButtonElement | undefined;
  const imgEl = new Image();

  // Active pointers — used to compute pinch ratio. We keep at most 2.
  const pointers = new Map<number, Pointer>();
  let pinchStartDist = 0;
  let pinchStartZoom = 1;

  /**
   * Cover-fit base scale: the multiplier that makes the natural image just
   * fill the square viewport. Then zoom multiplies on top. The displayed
   * image dimensions are therefore `(imgW * baseScale * zoom, ...)` and
   * they're guaranteed to be ≥ STAGE_PX on both axes whenever zoom ≥ 1.
   */
  function baseScale(): number {
    const w = imgW();
    const h = imgH();
    if (!w || !h) return 1;
    return Math.max(STAGE_PX / w, STAGE_PX / h);
  }

  /** Max absolute offset on each axis so the image still covers the viewport. */
  function maxOffset(): { x: number; y: number } {
    const s = baseScale() * zoom();
    const w = imgW() * s;
    const h = imgH() * s;
    return {
      x: Math.max(0, (w - STAGE_PX) / 2),
      y: Math.max(0, (h - STAGE_PX) / 2),
    };
  }

  /** Clamp the current offsets to whatever the current zoom allows. */
  function clampOffsets(): void {
    const m = maxOffset();
    setOffsetX((v) => Math.max(-m.x, Math.min(m.x, v)));
    setOffsetY((v) => Math.max(-m.y, Math.min(m.y, v)));
  }

  function recenter(): void {
    setZoom(1);
    setOffsetX(0);
    setOffsetY(0);
  }

  function applyZoom(next: number, anchor?: { x: number; y: number }): void {
    const clamped = Math.max(MIN_ZOOM, Math.min(MAX_ZOOM, next));
    const prev = zoom();
    if (clamped === prev) return;
    // Zoom about the anchor point (stage-local coords, origin at center)
    if (anchor) {
      const ratio = clamped / prev;
      setOffsetX((v) => anchor.x + (v - anchor.x) * ratio);
      setOffsetY((v) => anchor.y + (v - anchor.y) * ratio);
    }
    setZoom(clamped);
    clampOffsets();
  }

  // ------- Load source image -------
  imgEl.onload = () => {
    setImgW(imgEl.naturalWidth);
    setImgH(imgEl.naturalHeight);
    setLoaded(true);
  };
  imgEl.onerror = () => {
    // Surface as a no-op confirm path; parent handles upload errors.
    setLoaded(false);
  };
  imgEl.src = props.src;

  // ------- Pointer / wheel handlers -------
  function distance(a: Pointer, b: Pointer): number {
    return Math.hypot(a.x - b.x, a.y - b.y);
  }

  function onPointerDown(e: PointerEvent): void {
    if (!stageEl) return;
    (e.currentTarget as HTMLElement).setPointerCapture?.(e.pointerId);
    pointers.set(e.pointerId, { id: e.pointerId, x: e.clientX, y: e.clientY });
    if (pointers.size === 2) {
      const [a, b] = [...pointers.values()];
      pinchStartDist = distance(a, b);
      pinchStartZoom = zoom();
    }
    e.preventDefault();
  }

  function onPointerMove(e: PointerEvent): void {
    if (!pointers.has(e.pointerId)) return;
    const prev = pointers.get(e.pointerId)!;
    const cur = { id: e.pointerId, x: e.clientX, y: e.clientY };
    pointers.set(e.pointerId, cur);

    if (pointers.size === 2) {
      // Pinch zoom: ratio of current pair distance vs. pair distance at gesture start.
      const [a, b] = [...pointers.values()];
      const dist = distance(a, b);
      if (pinchStartDist > 0) {
        applyZoom(pinchStartZoom * (dist / pinchStartDist));
      }
      return;
    }

    // Single pointer drag
    const dx = cur.x - prev.x;
    const dy = cur.y - prev.y;
    setOffsetX((v) => v + dx);
    setOffsetY((v) => v + dy);
    clampOffsets();
  }

  function onPointerUp(e: PointerEvent): void {
    pointers.delete(e.pointerId);
    if (pointers.size < 2) {
      pinchStartDist = 0;
    }
  }

  function onWheel(e: WheelEvent): void {
    if (!stageEl) return;
    e.preventDefault();
    const rect = stageEl.getBoundingClientRect();
    const ax = e.clientX - (rect.left + rect.width / 2);
    const ay = e.clientY - (rect.top + rect.height / 2);
    // Soft factor; trackpads emit small deltas, mice large ones.
    const factor = Math.exp(-e.deltaY * 0.0015);
    applyZoom(zoom() * factor, { x: ax, y: ay });
  }

  function onKeyDown(e: KeyboardEvent): void {
    if (e.key === "Escape") {
      e.preventDefault();
      props.onCancel();
    }
  }

  // ------- Export -------
  function exportCropped(): void {
    if (!loaded() || busy()) return;
    setBusy(true);
    try {
      const out = props.size ?? 512;
      const canvas = document.createElement("canvas");
      canvas.width = out;
      canvas.height = out;
      const ctx = canvas.getContext("2d");
      if (!ctx) {
        props.onCancel();
        return;
      }
      // The displayed image in the stage has:
      //   size  = (imgW, imgH) * baseScale * zoom
      //   center = (STAGE_PX/2 + offsetX, STAGE_PX/2 + offsetY)
      // We want to render the STAGE_PX × STAGE_PX viewport into `out × out`.
      // Equivalent: for every stage pixel `p`, source pixel is
      //   src = (p - centerStage) / (baseScale * zoom) + (imgW/2, imgH/2)
      // So drawImage src rect is centered on (imgW/2 - offsetX/s, imgH/2 - offsetY/s)
      // with size (STAGE_PX/s, STAGE_PX/s), where s = baseScale*zoom.
      const s = baseScale() * zoom();
      const srcW = STAGE_PX / s;
      const srcH = STAGE_PX / s;
      const srcX = imgW() / 2 - offsetX() / s - srcW / 2;
      const srcY = imgH() / 2 - offsetY() / s - srcH / 2;
      // Clamp tightly within the natural image — should already be inside thanks
      // to maxOffset(), but guard against subpixel float drift.
      const sx = Math.max(0, Math.min(imgW() - srcW, srcX));
      const sy = Math.max(0, Math.min(imgH() - srcH, srcY));
      ctx.imageSmoothingEnabled = true;
      ctx.imageSmoothingQuality = "high";
      ctx.drawImage(imgEl, sx, sy, srcW, srcH, 0, 0, out, out);
      const dataUrl = canvas.toDataURL("image/jpeg", 0.85);
      props.onConfirm(dataUrl);
    } finally {
      setBusy(false);
    }
  }

  onMount(() => {
    document.addEventListener("keydown", onKeyDown);
    // Focus the primary action so Enter/Space confirms and screen readers land here.
    queueMicrotask(() => confirmBtn?.focus());
  });
  onCleanup(() => {
    document.removeEventListener("keydown", onKeyDown);
  });

  // CSS transform: translate the image so its center sits at stage-center + offset,
  // then scale. Using translate(-50%, -50%) to anchor by the image's own center.
  function imgStyle(): JSX.CSSProperties {
    const s = baseScale() * zoom();
    return {
      position: "absolute",
      left: "50%",
      top: "50%",
      width: `${imgW()}px`,
      height: `${imgH()}px`,
      "transform-origin": "center center",
      transform: `translate(-50%, -50%) translate(${offsetX()}px, ${offsetY()}px) scale(${s})`,
      "user-select": "none",
      "pointer-events": "none",
      "will-change": "transform",
    };
  }

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-label={t("profile.crop.title")}
      class="fixed inset-0 z-50 flex flex-col items-center justify-center bg-black/70 p-4"
    >
      <div class="card flex w-full max-w-sm flex-col gap-4 p-5">
        <h3 class="h-section">{t("profile.crop.title")}</h3>

        <div
          ref={stageEl}
          class="relative mx-auto touch-none overflow-hidden bg-[color:var(--color-neutral-900)] select-none"
          style={{
            width: `${STAGE_PX}px`,
            height: `${STAGE_PX}px`,
            "border-radius": "8px",
            "clip-path": "inset(0 round 8px)",
          }}
          onPointerDown={onPointerDown}
          onPointerMove={onPointerMove}
          onPointerUp={onPointerUp}
          onPointerCancel={onPointerUp}
          onWheel={onWheel}
        >
          {/* The image fills the stage; pointer events live on the stage wrapper. */}
          {loaded() ? <img src={props.src} alt="" style={imgStyle()} draggable={false} /> : null}

          {/* Circular mask overlay: dim outside the circle, leave inside fully visible. */}
          <div
            aria-hidden="true"
            class="pointer-events-none absolute inset-0"
            style={{
              background: "rgba(0, 0, 0, 0.55)",
              "-webkit-mask":
                "radial-gradient(circle at center, transparent 0, transparent calc(50% - 0.5px), black calc(50% + 0.5px))",
              mask:
                "radial-gradient(circle at center, transparent 0, transparent calc(50% - 0.5px), black calc(50% + 0.5px))",
            }}
          />
          {/* Crisp circular outline to reinforce the crop boundary. */}
          <div
            aria-hidden="true"
            class="pointer-events-none absolute inset-0"
            style={{
              border: "1px solid rgba(255, 255, 255, 0.65)",
              "border-radius": "50%",
            }}
          />
        </div>

        <label class="flex items-center gap-3">
          <span class="label mb-0 shrink-0">{t("profile.crop.zoom")}</span>
          <input
            type="range"
            min={MIN_ZOOM}
            max={MAX_ZOOM}
            step={0.01}
            value={zoom()}
            onInput={(e) => applyZoom(Number(e.currentTarget.value))}
            class="flex-1"
            aria-label={t("profile.crop.zoom")}
          />
        </label>

        <div class="flex flex-wrap items-center justify-between gap-2">
          <button type="button" class="btn btn-ghost" onClick={recenter}>
            {t("profile.crop.recenter")}
          </button>
          <div class="flex items-center gap-2">
            <button type="button" class="btn btn-ghost" onClick={() => props.onCancel()}>
              {t("profile.crop.cancel")}
            </button>
            <button
              ref={confirmBtn}
              type="button"
              class="btn btn-primary"
              onClick={exportCropped}
              disabled={!loaded() || busy()}
            >
              {t("profile.crop.confirm")}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

export default PhotoCropper;
