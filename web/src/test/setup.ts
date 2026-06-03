import "@testing-library/jest-dom/vitest";
import { afterEach, vi } from "vitest";
import { cleanup } from "@solidjs/testing-library";

afterEach(() => {
  cleanup();
  if (typeof localStorage !== "undefined") localStorage.clear();
  vi.restoreAllMocks();
});

// jsdom doesn't ship crypto.randomUUID
if (typeof globalThis.crypto === "undefined" || typeof globalThis.crypto.randomUUID === "undefined") {
  Object.defineProperty(globalThis, "crypto", {
    value: {
      randomUUID: () =>
        ([1e7] as unknown as string + -1e3 + -4e3 + -8e3 + -1e11).replace(
          /[018]/g,
          (c: string) =>
            (
              parseInt(c) ^
              (Math.random() * 16) >> (parseInt(c) / 4)
            ).toString(16),
        ),
    },
  });
}
