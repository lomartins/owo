// owo mark — seven cowrie shells on a terracotta disc, one per orixá.
// Canonical mark from the owo design system (assets/owo_mark.svg).
// Inline SVG so it ships with the JS bundle and scales cleanly.

import type { JSX } from "solid-js";

interface Props {
  class?: string;
  title?: string;
  /** Render shells on a transparent background (use when parent already paints terracotta). */
  inverse?: boolean;
  /** Render bare shells in primary-600, no disc — for tinted surface contexts. */
  bare?: boolean;
}

const SHELL_ANGLES = [0, 51.43, 102.86, 154.29, 205.71, 257.14, 308.57];
const SHELL_SLIT =
  "M 0 -7.5 Q -1.5 -5.5 -1 -3.5 Q -2 -1.5 -1 0.5 Q -2 2.5 -1 4.5 Q -1.5 6 0 7.5 Q 1.5 6 1 4.5 Q 2 2.5 1 0.5 Q 2 -1.5 1 -3.5 Q 1.5 -5.5 0 -7.5 Z";

export function Cowrie(props: Props): JSX.Element {
  const discFill = props.inverse || props.bare ? "transparent" : "#ad4f32";
  const shellFill = props.bare ? "#ad4f32" : "#ffffff";
  const slitFill = props.bare ? "#fbeee9" : "#ad4f32";
  return (
    <svg
      class={props.class}
      viewBox="-70 -70 140 140"
      xmlns="http://www.w3.org/2000/svg"
      role="img"
      aria-label={props.title ?? "owo"}
    >
      <title>{props.title ?? "owo — sete búzios"}</title>
      <rect x="-70" y="-70" width="140" height="140" fill={discFill} />
      <g>
        {SHELL_ANGLES.map((deg) => (
          <g transform={`rotate(${deg}) translate(0 -44)`}>
            <ellipse rx="7.5" ry="11.5" fill={shellFill} />
            <path d={SHELL_SLIT} fill={slitFill} />
          </g>
        ))}
      </g>
    </svg>
  );
}
