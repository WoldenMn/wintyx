/// <reference types="node" />
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AdminProvider } from "../../../contexts/AdminContext";
import { ActionButton } from "../ActionButton";
import type { AdminStatus } from "../../../lib/types";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const mockedInvoke = vi.mocked(invoke);

function mockStatus(status: AdminStatus) {
  mockedInvoke.mockImplementation(async (cmd: string) => {
    if (cmd === "get_admin_status") return status;
    return null;
  });
}

function Wrapper({ children }: { children: ReactNode }) {
  return <AdminProvider>{children}</AdminProvider>;
}

beforeEach(() => {
  vi.clearAllMocks();
});

describe("ActionButton", () => {
  it("renders children and is enabled by default", () => {
    render(<ActionButton>Go</ActionButton>);
    const btn = screen.getByRole("button", { name: "Go" });
    expect(btn).toBeEnabled();
  });

  it("is disabled when loading", () => {
    render(<ActionButton loading>Go</ActionButton>);
    expect(screen.getByRole("button", { name: "Go" })).toBeDisabled();
  });

  it("is disabled when disabled prop is set", () => {
    render(<ActionButton disabled>Go</ActionButton>);
    expect(screen.getByRole("button", { name: "Go" })).toBeDisabled();
  });

  it("is NOT blocked by requiresAdmin without AdminProvider (fallback safe)", () => {
    render(<ActionButton requiresAdmin>Admin</ActionButton>);
    const btn = screen.getByRole("button", { name: "Admin" });
    expect(btn).toBeEnabled();
  });

  it("is disabled + tooltip when requiresAdmin and user is not admin", async () => {
    mockStatus({ isAdmin: false, canElevate: true });
    render(
      <Wrapper>
        <ActionButton requiresAdmin>Admin</ActionButton>
      </Wrapper>,
    );
    await waitFor(() => {
      const btn = screen.getByRole("button", { name: "Admin" });
      expect(btn).toBeDisabled();
    });
    const btn = screen.getByRole("button", { name: "Admin" });
    expect(btn.getAttribute("title")).toMatch(/administrateur/i);
    expect(btn.getAttribute("aria-disabled")).toBe("true");
  });

  it("is enabled when requiresAdmin and user IS admin", async () => {
    mockStatus({ isAdmin: true, canElevate: true });
    render(
      <Wrapper>
        <ActionButton requiresAdmin>Admin</ActionButton>
      </Wrapper>,
    );
    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith("get_admin_status");
    });
    const btn = screen.getByRole("button", { name: "Admin" });
    await waitFor(() => expect(btn).toBeEnabled());
  });

  it("preserves custom title when not admin-blocked", () => {
    render(<ActionButton title="custom tip">X</ActionButton>);
    expect(screen.getByRole("button", { name: "X" }).getAttribute("title")).toBe(
      "custom tip",
    );
  });

  it("states the reason visibly, in the tooltip and in an accessible description", async () => {
    mockStatus({ isAdmin: false, canElevate: true });
    render(
      <Wrapper>
        <ActionButton requiresAdmin>Admin</ActionButton>
      </Wrapper>,
    );
    const btn = await screen.findByRole("button", { name: "Admin" });
    await waitFor(() => expect(btn).toBeDisabled());
    expect(btn.getAttribute("title")).toBe("Droits administrateur requis");
    expect(btn).toHaveAccessibleDescription("Droits administrateur requis");
  });

  it("does not describe a button disabled for another reason as admin-only", () => {
    render(<ActionButton disabled>Go</ActionButton>);
    expect(screen.getByRole("button", { name: "Go" })).not.toHaveAccessibleDescription();
  });
});

function luminance(hex: string): number {
  const channels = [1, 3, 5].map((i) => {
    const c = parseInt(hex.slice(i, i + 2), 16) / 255;
    return c <= 0.03928 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4);
  });
  return 0.2126 * channels[0] + 0.7152 * channels[1] + 0.0722 * channels[2];
}

function contrast(a: string, b: string): number {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (hi + 0.05) / (lo + 0.05);
}

function token(block: string, name: string): string {
  const m = block.match(new RegExp(`--t-${name}: (#[0-9A-Fa-f]{6})`));
  if (!m) throw new Error(`token ${name} introuvable`);
  return m[1];
}

describe("ActionButton disabled contrast", () => {
  const css = readFileSync(resolve(process.cwd(), "src/index.css"), "utf-8");
  const daylightStart = css.indexOf('[data-theme="daylight"]');
  const themes: Record<string, string> = {
    obsidian: css.slice(css.indexOf('[data-theme="obsidian"]'), daylightStart),
    daylight: css.slice(daylightStart),
  };

  for (const [name, block] of Object.entries(themes)) {
    it(`keeps the disabled label at AA (4.5:1) in ${name}`, () => {
      const ratio = contrast(token(block, "fg-dim"), token(block, "bg-elevated"));
      expect(ratio).toBeGreaterThanOrEqual(4.5);
    });
  }

  it("styles the disabled state with the text/surface pair measured above, not a 50% opacity", () => {
    render(<ActionButton disabled variant="accent">Go</ActionButton>);
    const cls = screen.getByRole("button", { name: "Go" }).className;
    expect(cls).toContain("disabled:text-fg-dim");
    expect(cls).toContain("disabled:bg-bg-elevated");
    expect(cls).not.toContain("disabled:opacity");
  });
});
