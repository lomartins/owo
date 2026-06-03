import { describe, it, expect } from "vitest";
import { localizeCategoryName, isSeedName } from "./categories";

describe("localizeCategoryName", () => {
  it("returns the EN seed verbatim in 'en' mode", () => {
    expect(localizeCategoryName("Food", "en")).toBe("Food");
    expect(localizeCategoryName("Home", "en")).toBe("Home");
    expect(localizeCategoryName("Other", "en")).toBe("Other");
  });

  it("maps EN seed names to PT-BR in 'pt' mode", () => {
    expect(localizeCategoryName("Food", "pt")).toBe("Comida");
    expect(localizeCategoryName("Home", "pt")).toBe("Casa");
    expect(localizeCategoryName("Leisure", "pt")).toBe("Lazer");
    expect(localizeCategoryName("Transport", "pt")).toBe("Transporte");
    expect(localizeCategoryName("Subscriptions", "pt")).toBe("Assinaturas");
  });

  it("returns user-custom names unchanged in either language", () => {
    expect(localizeCategoryName("Aluguel", "pt")).toBe("Aluguel");
    expect(localizeCategoryName("Aluguel", "en")).toBe("Aluguel");
    expect(localizeCategoryName("Streaming services", "pt")).toBe("Streaming services");
  });
});

describe("isSeedName", () => {
  it("recognises EN seed names", () => {
    for (const n of ["Food", "Leisure", "Transport", "Health", "Education", "Clothes", "Home", "Pet", "Subscriptions", "Other"]) {
      expect(isSeedName(n)).toBe(true);
    }
  });
  it("recognises PT translations as seeds too (round-trip safety)", () => {
    expect(isSeedName("Comida")).toBe(true);
    expect(isSeedName("Casa")).toBe(true);
  });
  it("rejects user-custom names", () => {
    expect(isSeedName("Aluguel")).toBe(false);
    expect(isSeedName("Coffee")).toBe(false);
  });
});
