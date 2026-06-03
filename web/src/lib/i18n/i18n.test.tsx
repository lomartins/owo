// @vitest-environment jsdom
import { describe, it, expect, beforeEach } from "vitest";
import { render } from "@solidjs/testing-library";
import { I18nProvider, useI18n } from "./index";

function Probe(): import("solid-js").JSX.Element {
  const { t, lang, setLang, locale } = useI18n();
  return (
    <div>
      <p data-testid="lang">{lang()}</p>
      <p data-testid="locale">{locale()}</p>
      <p data-testid="dashboard">{t("nav.dashboard")}</p>
      <p data-testid="greeting">{t("dashboard.greetingPair", { name: "Luisa", partner: "Mavê" })}</p>
      <button data-testid="to-pt" onClick={() => setLang("pt")}>pt</button>
      <button data-testid="to-en" onClick={() => setLang("en")}>en</button>
    </div>
  );
}

describe("I18nProvider", () => {
  beforeEach(() => {
    // Avoid persisted lang bleeding across tests
    if (typeof localStorage !== "undefined") localStorage.removeItem("owo.lang");
  });

  it("renders EN strings by default in jsdom (navigator.language = en)", () => {
    const { getByTestId } = render(() => (
      <I18nProvider>
        <Probe />
      </I18nProvider>
    ));
    expect(getByTestId("lang").textContent).toBe("en");
    expect(getByTestId("locale").textContent).toBe("en-US");
    expect(getByTestId("dashboard").textContent).toBe("Dashboard");
  });

  it("switches strings reactively when setLang('pt') is called", async () => {
    const { getByTestId } = render(() => (
      <I18nProvider>
        <Probe />
      </I18nProvider>
    ));
    getByTestId("to-pt").click();
    expect(getByTestId("lang").textContent).toBe("pt");
    expect(getByTestId("locale").textContent).toBe("pt-BR");
    expect(getByTestId("dashboard").textContent).toBe("Painel");
  });

  it("interpolates {{name}} / {{partner}} placeholders", () => {
    const { getByTestId } = render(() => (
      <I18nProvider>
        <Probe />
      </I18nProvider>
    ));
    expect(getByTestId("greeting").textContent).toBe("Hi, Luisa & Mavê");
    getByTestId("to-pt").click();
    expect(getByTestId("greeting").textContent).toBe("Olá, Luisa & Mavê");
  });

  it("persists choice across remounts when localStorage is available", () => {
    if (typeof localStorage === "undefined") return; // jsdom build without Storage
    const { getByTestId, unmount } = render(() => (
      <I18nProvider>
        <Probe />
      </I18nProvider>
    ));
    getByTestId("to-pt").click();
    expect(localStorage.getItem("owo.lang")).toBe("pt");
    unmount();
    const r2 = render(() => (
      <I18nProvider>
        <Probe />
      </I18nProvider>
    ));
    expect(r2.getByTestId("lang").textContent).toBe("pt");
  });
});
