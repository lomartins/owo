import type { JSX } from "solid-js";

interface Props {
  title: string;
  hint?: string;
  action?: JSX.Element;
  /** Optional Material Symbol name (rounded set). Defaults to none. */
  icon?: string;
}

export function EmptyState(props: Props): JSX.Element {
  return (
    <div class="flex flex-col items-center gap-3 px-6 py-12 text-center">
      {props.icon && (
        <span
          class="material-symbols-rounded inline-grid place-items-center text-[color:var(--color-primary-600)]"
          style={{
            "font-size": "32px",
            width: "56px",
            height: "56px",
            "border-radius": "12px",
            background: "var(--color-primary-50)",
          }}
        >
          {props.icon}
        </span>
      )}
      <h3 class="h-title">{props.title}</h3>
      {props.hint && <p class="meta max-w-sm">{props.hint}</p>}
      {props.action}
    </div>
  );
}
