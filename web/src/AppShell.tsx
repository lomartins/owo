import { createResource, type JSX, Show, type ParentProps, onMount } from "solid-js";
import { useLocation, useNavigate } from "@solidjs/router";
import { AppHeader } from "./components/AppHeader";
import { BottomNav } from "./components/BottomNav";
import { SwipeNav } from "./components/SwipeNav";
import { useMonth } from "./lib/useMonth";
import { reports } from "./api";
import { currentUser } from "./lib/session";

/** Shell wrapping routed children with header + month context. */
export function AppShell(props: ParentProps): JSX.Element {
  const navigate = useNavigate();
  const location = useLocation();
  onMount(() => {
    if (!currentUser()) navigate("/login", { replace: true });
  });

  const { month, setMonth, refreshToken } = useMonth();
  const [report] = createResource(
    () => ({ m: month(), r: refreshToken() }),
    (args) => reports.monthly(args.m).catch(() => null),
  );

  return (
    <Show when={currentUser()} fallback={<div class="p-8 text-center text-sm">Redirecting…</div>}>
      <div class="flex min-h-dvh flex-col">
        <AppHeader month={month()} onMonthChange={setMonth} report={report() ?? null} />
        <main class="flex-1 pb-[88px] md:pb-0">
          <SwipeNav>
            {/* Key on pathname so each route mount triggers the enter animation.
                The keyed <Show> forces the wrapper div to remount on every path
                change, replaying the CSS keyframes in `.route-transition`. */}
            <Show when={location.pathname} keyed>
              {(path) => (
                <div class="route-transition" data-path={path}>
                  {props.children}
                </div>
              )}
            </Show>
          </SwipeNav>
        </main>
        <footer class="caption mx-auto hidden w-full max-w-4xl px-4 py-6 text-center md:block">
          owo · self-hosted · {new Date().getFullYear()}
        </footer>
        <BottomNav />
      </div>
    </Show>
  );
}
