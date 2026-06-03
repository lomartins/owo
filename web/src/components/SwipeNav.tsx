import { onCleanup, onMount, type JSX, type ParentProps } from "solid-js";
import { useLocation, useNavigate } from "@solidjs/router";
import { SWIPE_TABS } from "./BottomNav";

/**
 * Mobile-only horizontal swipe navigation between primary tabs.
 *
 * - Listens to touch events on its root element (placed around <main>).
 * - Commits navigation on left/right swipes with sane thresholds:
 *   >=60px X, <30px Y deviation, >0.3 px/ms velocity, single touch.
 * - Only fires when the current path is one of SWIPE_TABS.
 * - Bails when the gesture originates on an interactive element (input,
 *   textarea, select, button, link, [role=button]) — prevents swallowing
 *   intent on form controls and chip rows.
 * - Disabled on desktop (>= 768px).
 * - Provides a tiny live nudge via `.swipe-feedback` so the gesture feels
 *   physical. Suppressed under prefers-reduced-motion via CSS.
 */
export function SwipeNav(props: ParentProps): JSX.Element {
  const location = useLocation();
  const navigate = useNavigate();

  let root: HTMLDivElement | undefined;
  let startX = 0;
  let startY = 0;
  let startT = 0;
  let active = false;     // a single-touch gesture is in progress
  let committed = false;  // we've decided this gesture is a horizontal swipe
  let desktop = false;

  const onTouchStart = (e: TouchEvent) => {
    if (desktop) return;
    if (e.touches.length !== 1) {
      active = false;
      return;
    }

    // Bail if the gesture originates on an interactive control.
    const target = e.target as Element | null;
    if (
      target &&
      typeof (target as Element).closest === "function" &&
      target.closest('input, textarea, select, button, a, [role="button"]')
    ) {
      active = false;
      return;
    }

    const idx = SWIPE_TABS.indexOf(location.pathname);
    if (idx === -1) {
      active = false;
      return;
    }

    const touch = e.touches[0];
    startX = touch.clientX;
    startY = touch.clientY;
    startT = e.timeStamp;
    active = true;
    committed = false;
  };

  const onTouchMove = (e: TouchEvent) => {
    if (!active || desktop) return;
    if (e.touches.length !== 1) {
      cancelGesture();
      return;
    }
    const touch = e.touches[0];
    const dx = touch.clientX - startX;
    const dy = touch.clientY - startY;

    if (!committed) {
      // Only commit if motion is clearly horizontal.
      if (Math.abs(dx) > 10 && Math.abs(dx) > Math.abs(dy) * 1.5) {
        committed = true;
        root?.classList.add("swipe-feedback");
      } else if (Math.abs(dy) > 10) {
        // Vertical scroll wins; let the page handle it.
        cancelGesture();
        return;
      }
    }

    if (committed && root) {
      // Resist past the ends so the user feels the boundary.
      const idx = SWIPE_TABS.indexOf(location.pathname);
      const atStart = idx === 0 && dx > 0;
      const atEnd = idx === SWIPE_TABS.length - 1 && dx < 0;
      const factor = atStart || atEnd ? 0.25 : 1;
      const nudge = Math.max(-12, Math.min(12, dx * 0.08)) * factor;
      root.style.transform = `translate3d(${nudge.toFixed(2)}px, 0, 0)`;
      // Prevent the browser from triggering horizontal page scroll/back gesture.
      if (e.cancelable) e.preventDefault();
    }
  };

  const onTouchEnd = (e: TouchEvent) => {
    if (!active || desktop) {
      cleanupFeedback();
      active = false;
      return;
    }

    const touch = e.changedTouches[0];
    const dx = touch.clientX - startX;
    const dy = touch.clientY - startY;
    const dt = Math.max(1, e.timeStamp - startT);
    const vx = Math.abs(dx) / dt;

    cleanupFeedback();
    active = false;

    if (!committed) return;
    if (Math.abs(dx) < 60) return;
    if (Math.abs(dy) >= 30) return;
    if (vx <= 0.3) return;

    const idx = SWIPE_TABS.indexOf(location.pathname);
    if (idx === -1) return;

    // Left-swipe (dx < 0) → next tab. Right-swipe (dx > 0) → previous.
    const nextIdx = dx < 0 ? idx + 1 : idx - 1;
    if (nextIdx < 0 || nextIdx >= SWIPE_TABS.length) return;

    navigate(SWIPE_TABS[nextIdx]);
  };

  const onTouchCancel = () => {
    cancelGesture();
  };

  const cancelGesture = () => {
    cleanupFeedback();
    active = false;
    committed = false;
  };

  const cleanupFeedback = () => {
    if (!root) return;
    root.classList.remove("swipe-feedback");
    root.style.transform = "";
  };

  onMount(() => {
    if (!root) return;
    const mql = window.matchMedia("(min-width: 768px)");
    desktop = mql.matches;
    const onMqlChange = (ev: MediaQueryListEvent) => {
      desktop = ev.matches;
      if (desktop) cancelGesture();
    };
    // Older Safari uses addListener.
    if (typeof mql.addEventListener === "function") {
      mql.addEventListener("change", onMqlChange);
    } else if (typeof (mql as MediaQueryList & { addListener?: (cb: (e: MediaQueryListEvent) => void) => void }).addListener === "function") {
      (mql as MediaQueryList & { addListener: (cb: (e: MediaQueryListEvent) => void) => void }).addListener(onMqlChange);
    }

    root.addEventListener("touchstart", onTouchStart, { passive: true });
    // touchmove must be active so we can preventDefault once committed.
    root.addEventListener("touchmove", onTouchMove, { passive: false });
    root.addEventListener("touchend", onTouchEnd, { passive: true });
    root.addEventListener("touchcancel", onTouchCancel, { passive: true });

    onCleanup(() => {
      root?.removeEventListener("touchstart", onTouchStart);
      root?.removeEventListener("touchmove", onTouchMove);
      root?.removeEventListener("touchend", onTouchEnd);
      root?.removeEventListener("touchcancel", onTouchCancel);
      if (typeof mql.removeEventListener === "function") {
        mql.removeEventListener("change", onMqlChange);
      } else if (typeof (mql as MediaQueryList & { removeListener?: (cb: (e: MediaQueryListEvent) => void) => void }).removeListener === "function") {
        (mql as MediaQueryList & { removeListener: (cb: (e: MediaQueryListEvent) => void) => void }).removeListener(onMqlChange);
      }
    });
  });

  return (
    <div ref={root} class="swipe-root">
      {props.children}
    </div>
  );
}
