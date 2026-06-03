import { describe, it, expect } from "vitest";
import { parseCents, centsToApi, formatMoney, formatSigned, inputToApi } from "./money";

describe("parseCents", () => {
  it("returns 0n for null/empty", () => {
    expect(parseCents(null)).toBe(0n);
    expect(parseCents(undefined)).toBe(0n);
    expect(parseCents("")).toBe(0n);
  });

  it("parses positive decimals to cents", () => {
    expect(parseCents("0")).toBe(0n);
    expect(parseCents("0.00")).toBe(0n);
    expect(parseCents("1")).toBe(100n);
    expect(parseCents("1.5")).toBe(150n);
    expect(parseCents("1234.56")).toBe(123456n);
    expect(parseCents("9999999.99")).toBe(999999999n);
  });

  it("parses negative decimals", () => {
    expect(parseCents("-12.34")).toBe(-1234n);
  });

  it("throws on garbage input", () => {
    expect(() => parseCents("abc")).toThrow();
    expect(() => parseCents("1.234")).toThrow(); // 3 decimals
    expect(() => parseCents("1,234")).toThrow();
  });
});

describe("centsToApi", () => {
  it("round-trips with parseCents", () => {
    const cases = [0n, 100n, 123n, 999999999n, -1234n];
    for (const c of cases) {
      expect(parseCents(centsToApi(c))).toBe(c);
    }
  });

  it("pads fractional cents to two digits", () => {
    expect(centsToApi(5n)).toBe("0.05");
    expect(centsToApi(50n)).toBe("0.50");
    expect(centsToApi(100n)).toBe("1.00");
  });

  it("preserves sign on negative", () => {
    expect(centsToApi(-5n)).toBe("-0.05");
    expect(centsToApi(-12345n)).toBe("-123.45");
  });
});

describe("formatMoney", () => {
  it("formats BRL with pt-BR locale", () => {
    const s = formatMoney(123456n, "BRL", "pt-BR");
    // Result varies by Intl impl: usually "R$ 1.234,56" — assert structural bits.
    expect(s).toMatch(/R\$\s?1[. \s]234,56/);
  });

  it("formats USD with en-US locale", () => {
    expect(formatMoney(123456n, "USD", "en-US")).toBe("$1,234.56");
  });
});

describe("formatSigned", () => {
  it("returns zero tone for 0", () => {
    const r = formatSigned(0n);
    expect(r.tone).toBe("zero");
  });
  it("signs positive with +", () => {
    const r = formatSigned(100n, "BRL", "pt-BR");
    expect(r.tone).toBe("pos");
    expect(r.text.startsWith("+")).toBe(true);
  });
  it("signs negative with −", () => {
    const r = formatSigned(-100n, "BRL", "pt-BR");
    expect(r.tone).toBe("neg");
    expect(r.text.startsWith("−")).toBe(true);
  });
});

describe("inputToApi", () => {
  it("accepts BR formatting (comma decimal)", () => {
    expect(inputToApi("1234,56")).toBe("1234.56");
    expect(inputToApi("1.234,56")).toBe("1234.56");
  });
  it("accepts US formatting (dot decimal)", () => {
    expect(inputToApi("1234.56")).toBe("1234.56");
    expect(inputToApi("1,234.56")).toBe("1234.56");
  });
  it("returns 0.00 for empty / garbage", () => {
    expect(inputToApi("")).toBe("0.00");
    expect(inputToApi("not-a-number")).toBe("0.00");
  });
  it("rounds to 2 decimals", () => {
    expect(inputToApi("1.999")).toBe("2.00");
  });
});
