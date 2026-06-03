// Month helpers: API uses "YYYY-MM" strings.

export function currentMonth(d = new Date()): string {
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, "0");
  return `${y}-${m}`;
}

export function shiftMonth(m: string, delta: number): string {
  const [y, mo] = m.split("-").map((p) => parseInt(p, 10));
  const date = new Date(y, mo - 1 + delta, 1);
  return currentMonth(date);
}

export function monthLabel(m: string, locale = "pt-BR"): string {
  const [y, mo] = m.split("-").map((p) => parseInt(p, 10));
  const date = new Date(y, mo - 1, 1);
  return new Intl.DateTimeFormat(locale, { month: "long", year: "numeric" }).format(date);
}

export function today(): string {
  const d = new Date();
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${y}-${m}-${day}`;
}

export function formatDate(iso: string, locale = "pt-BR"): string {
  const d = new Date(iso + (iso.length === 10 ? "T00:00:00" : ""));
  // dd/mm/yyyy for pt-BR; en-GB renders the same shape for English users.
  const out = locale === "pt-BR" ? "pt-BR" : "en-GB";
  return new Intl.DateTimeFormat(out, {
    day: "2-digit",
    month: "2-digit",
    year: "numeric",
  }).format(d);
}
