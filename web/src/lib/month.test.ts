import { describe, it, expect } from "vitest";
import { currentMonth, shiftMonth, monthLabel, today, formatDate } from "./month";

describe("currentMonth", () => {
  it("formats a Date as YYYY-MM", () => {
    expect(currentMonth(new Date(2026, 4, 27))).toBe("2026-05");
    expect(currentMonth(new Date(2026, 0, 1))).toBe("2026-01");
    expect(currentMonth(new Date(2026, 11, 31))).toBe("2026-12");
  });
});

describe("shiftMonth", () => {
  it("shifts +1 across same year", () => {
    expect(shiftMonth("2026-05", 1)).toBe("2026-06");
  });
  it("shifts +1 across year boundary", () => {
    expect(shiftMonth("2026-12", 1)).toBe("2027-01");
  });
  it("shifts -1 across year boundary", () => {
    expect(shiftMonth("2026-01", -1)).toBe("2025-12");
  });
  it("shifts +12 = same month next year", () => {
    expect(shiftMonth("2026-05", 12)).toBe("2027-05");
  });
});

describe("monthLabel", () => {
  it("renders pt-BR long form", () => {
    const label = monthLabel("2026-05", "pt-BR");
    expect(label.toLowerCase()).toContain("maio");
    expect(label).toContain("2026");
  });
  it("renders en-US long form", () => {
    const label = monthLabel("2026-05", "en-US");
    expect(label).toContain("May");
    expect(label).toContain("2026");
  });
});

describe("today", () => {
  it("returns ISO date string YYYY-MM-DD", () => {
    expect(today()).toMatch(/^\d{4}-\d{2}-\d{2}$/);
  });
});

describe("formatDate", () => {
  it("renders dd/mm/yyyy for pt-BR", () => {
    expect(formatDate("2026-05-20", "pt-BR")).toBe("20/05/2026");
  });
  it("renders dd/mm/yyyy shape for en (uses en-GB underneath)", () => {
    expect(formatDate("2026-05-20", "en-US")).toBe("20/05/2026");
  });
});
