import { describe, expect, it } from "vitest";
import { validateBranchName } from "./branch-name";

describe("validateBranchName", () => {
  it.each([
    ["foo..bar"],
    ["foo\0bar"],
    ["foo bar"],
    ["a".repeat(101)],
    ["foo\\bar"],
    ["foo:bar"],
    [".hidden"],
    ["foo/"],
    ["foo//bar"],
  ])("rejette %s", (name) => {
    expect(validateBranchName(name).valid).toBe(false);
  });

  it.each([
    ["feature"],
    ["feature/foo"],
  ])("accepte %s", (name) => {
    expect(validateBranchName(name)).toEqual({ valid: true });
  });
});
