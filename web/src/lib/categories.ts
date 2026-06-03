// Translation map for the 10 default English seed categories provisioned at
// register. The DB stores them canonically in English ("Food", "Home"…). When
// the user toggles language the SPA renders the localized label; when the user
// renames a category, it no longer matches the seed map and the user's
// custom name takes over.

import type { Lang } from "./i18n";

const SEED_PT: Record<string, string> = {
  Food: "Comida",
  Leisure: "Lazer",
  Transport: "Transporte",
  Health: "Saúde",
  Education: "Educação",
  Clothes: "Roupas",
  Home: "Casa",
  Pet: "Pet",
  Subscriptions: "Assinaturas",
  Other: "Outros",
};

const SEED_EN: Record<string, string> = Object.fromEntries(
  Object.keys(SEED_PT).map((k) => [k, k]),
);

export function localizeCategoryName(rawName: string, lang: Lang): string {
  const dict = lang === "pt" ? SEED_PT : SEED_EN;
  return dict[rawName] ?? rawName;
}

/** Backwards lookup — accept either an English seed name or any PT seed name. */
export function isSeedName(name: string): boolean {
  return name in SEED_PT || Object.values(SEED_PT).includes(name);
}
