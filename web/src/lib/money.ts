// Money on the API boundary is a JSON string holding a Decimal — e.g. "1234.56".
// We never store money as a JS number. Parsing → BigInt cents; formatting →
// locale-aware string with grouping; arithmetic for client-side previews is in
// BigInt-cents space and converted back at the edges.

const HUNDRED = 100n;

export type Currency = "BRL" | "USD" | "EUR" | (string & {});

/** Parse a decimal API string to integer cents as a BigInt. Throws on garbage. */
export function parseCents(value: string | null | undefined): bigint {
  if (value == null || value === "") return 0n;
  const s = String(value).trim();
  const neg = s.startsWith("-");
  const body = neg ? s.slice(1) : s;
  if (!/^\d+(\.\d{1,2})?$/.test(body)) {
    throw new Error(`bad decimal: ${value}`);
  }
  const [int, frac = ""] = body.split(".");
  const f = (frac + "00").slice(0, 2);
  const v = BigInt(int) * HUNDRED + BigInt(f);
  return neg ? -v : v;
}

/** Cents → decimal string fit for posting back to the API. */
export function centsToApi(c: bigint): string {
  const neg = c < 0n;
  const abs = neg ? -c : c;
  const int = abs / HUNDRED;
  const frac = (abs % HUNDRED).toString().padStart(2, "0");
  return `${neg ? "-" : ""}${int}.${frac}`;
}

/** Format cents for display. Locale-aware grouping, fixed 2 decimals. */
export function formatMoney(
  c: bigint,
  currency: Currency = "BRL",
  locale: string = "pt-BR",
): string {
  const n = Number(c) / 100;
  return new Intl.NumberFormat(locale, {
    style: "currency",
    currency,
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(n);
}

/** Format a signed difference with a soft '+' for positive and '−' for negative. */
export function formatSigned(
  c: bigint,
  currency: Currency = "BRL",
  locale: string = "pt-BR",
): { text: string; tone: "pos" | "neg" | "zero" } {
  if (c === 0n) return { text: formatMoney(0n, currency, locale), tone: "zero" };
  const abs = c < 0n ? -c : c;
  const sign = c < 0n ? "−" : "+";
  return {
    text: `${sign} ${formatMoney(abs, currency, locale)}`,
    tone: c < 0n ? "neg" : "pos",
  };
}

/** Parse user input (free typing) → API decimal string. */
export function inputToApi(raw: string): string {
  if (!raw) return "0.00";
  // Accept "1234,56", "1.234,56", "1234.56", "1,234.56"
  const cleaned = raw.replace(/\s/g, "");
  const hasComma = cleaned.includes(",");
  const hasDot = cleaned.includes(".");
  let normalised: string;
  if (hasComma && hasDot) {
    // last separator is decimal
    const lastComma = cleaned.lastIndexOf(",");
    const lastDot = cleaned.lastIndexOf(".");
    if (lastComma > lastDot) {
      normalised = cleaned.replace(/\./g, "").replace(",", ".");
    } else {
      normalised = cleaned.replace(/,/g, "");
    }
  } else if (hasComma) {
    normalised = cleaned.replace(",", ".");
  } else {
    normalised = cleaned;
  }
  const n = Number(normalised);
  if (!isFinite(n)) return "0.00";
  return n.toFixed(2);
}
