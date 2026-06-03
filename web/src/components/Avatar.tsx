import { Show, type JSX } from "solid-js";
import type { CachedUser } from "../api/client";

interface Props {
  user: CachedUser | null;
  /** sm = header, md = dashboard greeting, lg = profile page. */
  size?: "sm" | "md" | "lg";
  class?: string;
}

const PX: Record<NonNullable<Props["size"]>, { box: string; font: string }> = {
  sm: { box: "h-9 w-9", font: "12px" },
  md: { box: "h-16 w-16", font: "22px" },
  lg: { box: "h-24 w-24", font: "32px" },
};

export function Avatar(props: Props): JSX.Element {
  const size = () => props.size ?? "sm";
  const dims = () => PX[size()];

  const initials = (): string => {
    const u = props.user;
    if (!u) return "·";
    const names = [u.display_name, u.partner_name].filter(Boolean) as string[];
    const letters = names.map((n) => n.trim()[0]?.toUpperCase() ?? "").join("");
    return letters || "·";
  };

  return (
    <Show
      when={props.user?.photo_url}
      fallback={
        <span
          class={`${dims().box} grid place-items-center rounded-full bg-[color:var(--color-primary-50)] text-[color:var(--color-primary-600)] font-medium ${props.class ?? ""}`}
          style={{ "font-size": dims().font }}
          aria-hidden
        >
          {initials()}
        </span>
      }
    >
      {(url) => (
        <img
          src={url()}
          alt=""
          loading="lazy"
          class={`${dims().box} rounded-full object-cover shadow-[var(--shadow-card)] ${props.class ?? ""}`}
        />
      )}
    </Show>
  );
}
