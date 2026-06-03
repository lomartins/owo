import { createSignal, Show, type JSX } from "solid-js";
import { useNavigate } from "@solidjs/router";
import { Cowrie } from "../components/Cowrie";
import { LangToggle } from "../components/LangToggle";
import { auth } from "../api";
import { ApiError } from "../api/client";
import { setSession } from "../lib/session";
import { useI18n } from "../lib/i18n";

export default function Login(): JSX.Element {
  const navigate = useNavigate();
  const { t, locale } = useI18n();
  const [mode, setMode] = createSignal<"login" | "register">("login");
  const [email, setEmail] = createSignal("");
  const [password, setPassword] = createSignal("");
  const [name, setName] = createSignal("");
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  async function submit(e: Event): Promise<void> {
    e.preventDefault();
    setError(null);
    setBusy(true);
    try {
      const res = mode() === "login"
        ? await auth.login(email().trim(), password())
        : await auth.register({
            email: email().trim(),
            password: password(),
            display_name: name().trim() || email().split("@")[0]!,
            default_currency: "BRL",
            locale: locale(),
          });
      setSession(res.session.token, {
        id: res.user.id,
        email: res.user.email,
        display_name: res.user.display_name,
        default_currency: res.user.default_currency,
        locale: res.user.locale,
        partner_name: res.user.partner_name ?? null,
        photo_url: res.user.photo_url ?? null,
      });
      navigate("/", { replace: true });
    } catch (err) {
      if (err instanceof ApiError) {
        setError(err.code === "UNAUTHENTICATED" ? t("auth.invalidCredentials") : `${err.code}: ${err.message}`);
      } else {
        setError(t("auth.networkError"));
      }
    } finally {
      setBusy(false);
    }
  }

  return (
    <main
      class="grid min-h-dvh place-items-center px-4"
      style={{ background: "linear-gradient(180deg, #fdf9f6 0%, #fbeee9 100%)" }}
    >
      <div class="w-full max-w-sm">
        <div class="mb-10 flex flex-col items-center gap-4 text-center">
          <Cowrie class="h-20 w-20 rounded-2xl" />
          <h1 class="h-display">owó</h1>
          <p class="meta">{t("auth.tagline")}</p>
        </div>

        <form class="card flex flex-col gap-4 p-6" onSubmit={submit} novalidate>
          <div class="chip-row self-center">
            <button
              type="button"
              class="chip"
              aria-pressed={mode() === "login"}
              classList={{ "is-active": mode() === "login" }}
              onClick={() => setMode("login")}
            >
              {t("auth.signIn")}
            </button>
            <button
              type="button"
              class="chip"
              aria-pressed={mode() === "register"}
              classList={{ "is-active": mode() === "register" }}
              onClick={() => setMode("register")}
            >
              {t("auth.createAccount")}
            </button>
          </div>

          <Show when={mode() === "register"}>
            <label>
              <span class="label">{t("auth.displayName")}</span>
              <input
                class="field"
                value={name()}
                onInput={(e) => setName(e.currentTarget.value)}
                placeholder={t("auth.displayNamePlaceholder")}
                autocomplete="name"
              />
            </label>
          </Show>

          <label>
            <span class="label">{t("auth.email")}</span>
            <input
              type="email"
              class="field"
              required
              value={email()}
              onInput={(e) => setEmail(e.currentTarget.value)}
              placeholder="you@example.com"
              autocomplete="email"
            />
          </label>

          <label>
            <span class="label">{t("auth.password")}</span>
            <input
              type="password"
              class="field"
              required
              minLength={mode() === "register" ? 12 : 1}
              value={password()}
              onInput={(e) => setPassword(e.currentTarget.value)}
              placeholder={mode() === "register" ? t("auth.passwordHintRegister") : "••••••••"}
              autocomplete={mode() === "login" ? "current-password" : "new-password"}
            />
          </label>

          <Show when={error()}>
            {(msg) => <p class="meta text-[color:var(--color-danger-fg)]">{msg()}</p>}
          </Show>

          <button class="btn btn-primary" type="submit" disabled={busy()}>
            {busy() ? "…" : mode() === "login" ? t("auth.submitSignIn") : t("auth.submitRegister")}
          </button>
        </form>

        <div class="mt-6 flex justify-center">
          <LangToggle />
        </div>

        <p class="caption mt-4 text-center">
          {t("app.self_hosted")}
        </p>
      </div>
    </main>
  );
}
