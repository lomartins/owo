// Curated icon + colour palette for categories.
//
// `ICON_NAMES` is the 32-icon set surfaced in `<IconPicker />`. Every name is a
// Material Symbols Rounded glyph (font is loaded globally via index.html).
//
// `COLOR_PALETTE` is a brand-friendly tinted ramp — warm hues that sit
// naturally next to `--color-primary-600` without ever pulling the eye away
// from the terracotta accent. 8 hues, hand-picked to feel like the warm-paper
// surface (no cool blues, no neon).
//
// `defaultIconForSeed` + `defaultColorForSeed` give each of the 10 default
// seed categories a sensible icon/colour pair so the first-launch SPA looks
// curated without a backend migration.

export const ICON_NAMES: readonly string[] = [
  "restaurant",
  "shopping_cart",
  "directions_car",
  "local_taxi",
  "flight",
  "home",
  "school",
  "pets",
  "savings",
  "sports_esports",
  "music_note",
  "fitness_center",
  "medication",
  "local_pharmacy",
  "beach_access",
  "theater_comedy",
  "checkroom",
  "child_care",
  "work",
  "gift",
  "redeem",
  "cake",
  "coffee",
  "local_bar",
  "local_florist",
  "local_grocery_store",
  "smartphone",
  "computer",
  "build",
  "cleaning_services",
  "account_balance_wallet",
  "sell",
];

// Warm palette — terracotta, soft red, ochre, olive, sage, teal, mauve,
// neutral. Each hex is comfortable as a tinted icon background with white
// foreground glyphs at weight 500.
export const COLOR_PALETTE: readonly string[] = [
  "#ad4f32", // terracotta (brand primary)
  "#c2624a", // soft red
  "#c98a2b", // ochre
  "#8a7a2f", // olive
  "#6e8a5a", // sage
  "#3f7f7a", // teal
  "#8a5a7a", // mauve
  "#6b6058", // neutral warm
];

const SEED_ICON_MAP: Record<string, string> = {
  Food: "restaurant",
  Home: "home",
  Pet: "pets",
  Health: "medication",
  Leisure: "sports_esports",
  Education: "school",
  Transport: "directions_car",
  Clothes: "checkroom",
  Subscriptions: "smartphone",
  Other: "sell",
};

// Deterministic colour per seed. Order chosen so neighbouring seeds in the
// default list look distinct rather than clustering into the same hue.
const SEED_COLOR_MAP: Record<string, string> = {
  Food: "#ad4f32",          // terracotta
  Home: "#6e8a5a",          // sage
  Pet: "#c98a2b",           // ochre
  Health: "#3f7f7a",        // teal
  Leisure: "#8a5a7a",       // mauve
  Education: "#8a7a2f",     // olive
  Transport: "#c2624a",     // soft red
  Clothes: "#8a5a7a",       // mauve
  Subscriptions: "#6b6058", // neutral warm
  Other: "#6b6058",         // neutral warm
};

export function defaultIconForSeed(name: string): string | null {
  return SEED_ICON_MAP[name] ?? null;
}

export function defaultColorForSeed(name: string): string {
  return SEED_COLOR_MAP[name] ?? COLOR_PALETTE[0]!;
}
