// @vitest-environment jsdom
import { describe, it, expect } from "vitest";
import { render } from "@solidjs/testing-library";
import { Avatar } from "./Avatar";

const baseUser = {
  id: "u1",
  email: "l@m.com",
  display_name: "Luisa",
  default_currency: "BRL",
  locale: "pt-BR",
};

describe("Avatar", () => {
  it("renders initials when no photo_url", () => {
    const { container } = render(() => <Avatar user={{ ...baseUser }} />);
    expect(container.textContent).toContain("L");
    expect(container.querySelector("img")).toBeNull();
  });

  it("combines initials of user + partner", () => {
    const { container } = render(() => (
      <Avatar user={{ ...baseUser, partner_name: "Mavê" }} />
    ));
    expect(container.textContent).toContain("L");
    expect(container.textContent).toContain("M");
  });

  it("renders <img> when photo_url present", () => {
    const { container } = render(() => (
      <Avatar user={{ ...baseUser, photo_url: "/uploads/u/test.jpg" }} />
    ));
    const img = container.querySelector("img");
    expect(img).not.toBeNull();
    expect(img!.getAttribute("src")).toBe("/uploads/u/test.jpg");
  });

  it("falls back to · when user is null", () => {
    const { container } = render(() => <Avatar user={null} />);
    expect(container.textContent).toContain("·");
  });

  it("applies correct size class for size='lg'", () => {
    const { container } = render(() => <Avatar user={baseUser} size="lg" />);
    const node = container.firstElementChild as HTMLElement;
    expect(node.className).toMatch(/h-24/);
    expect(node.className).toMatch(/w-24/);
  });
});
