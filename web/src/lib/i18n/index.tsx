import { createContext, createMemo, createSignal, useContext, type JSX, type ParentProps } from "solid-js";
import { flatten } from "@solid-primitives/i18n";
import { en, type Dict } from "./en";
import { pt } from "./pt";

export type Lang = "en" | "pt";

const DICTS: Record<Lang, Dict> = { en, pt };
const STORAGE_KEY = "owo.lang";

function detectLang(): Lang {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored === "en" || stored === "pt") return stored;
  } catch {
    // ignore
  }
  const nav = (typeof navigator !== "undefined" && navigator.language) || "en";
  return nav.toLowerCase().startsWith("pt") ? "pt" : "en";
}

interface I18nValue {
  lang: () => Lang;
  setLang: (l: Lang) => void;
  t: (path: string, params?: Record<string, string | number>) => string;
  locale: () => string;
}

const I18nContext = createContext<I18nValue>();

export function I18nProvider(props: ParentProps): JSX.Element {
  const [lang, setLangSignal] = createSignal<Lang>(detectLang());

  const flat = createMemo(() => flatten(DICTS[lang()] as unknown as Record<string, unknown>) as Record<string, string>);

  const setLang = (l: Lang): void => {
    try {
      localStorage.setItem(STORAGE_KEY, l);
    } catch {
      // ignore
    }
    setLangSignal(l);
    if (typeof document !== "undefined") {
      document.documentElement.lang = l === "pt" ? "pt-BR" : "en";
    }
  };

  // Initialize <html lang>
  if (typeof document !== "undefined") {
    document.documentElement.lang = lang() === "pt" ? "pt-BR" : "en";
  }

  const t: I18nValue["t"] = (path, params) => {
    const tpl = flat()[path] ?? path;
    if (!params) return tpl;
    return tpl.replace(/\{\{(\w+)\}\}/g, (_match, key: string) => String(params[key] ?? `{{${key}}}`));
  };

  const locale = () => (lang() === "pt" ? "pt-BR" : "en-US");

  return (
    <I18nContext.Provider value={{ lang, setLang, t, locale }}>
      {props.children}
    </I18nContext.Provider>
  );
}

export function useI18n(): I18nValue {
  const ctx = useContext(I18nContext);
  if (!ctx) throw new Error("useI18n must be called inside <I18nProvider>");
  return ctx;
}
