import type { JSX } from "solid-js";
import { useI18n } from "../lib/i18n";

export function LangToggle(): JSX.Element {
  const { lang, setLang, t } = useI18n();
  return (
    <div class="chip-row" role="group" aria-label={t("language.label")}>
      <button
        type="button"
        class="chip"
        classList={{ "is-active": lang() === "pt" }}
        aria-pressed={lang() === "pt"}
        onClick={() => setLang("pt")}
      >
        PT
      </button>
      <button
        type="button"
        class="chip"
        classList={{ "is-active": lang() === "en" }}
        aria-pressed={lang() === "en"}
        onClick={() => setLang("en")}
      >
        EN
      </button>
    </div>
  );
}
