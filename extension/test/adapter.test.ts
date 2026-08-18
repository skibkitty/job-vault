import { describe, it, expect } from "vitest";
import { GenericAdapter } from "../src/adapters/adapter";

describe("GenericAdapter", () => {
  it("canHandle returns true for any URL", () => {
    const adapter = new GenericAdapter();
    expect(adapter.canHandle("https://example.com")).toBe(true);
  });
});
