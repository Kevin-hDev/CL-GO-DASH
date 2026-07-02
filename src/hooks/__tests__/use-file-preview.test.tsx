import { act, renderHook } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { useFilePreview } from "../use-file-preview";
import type { FileOperation } from "@/types/file-preview";

afterEach(() => {
  localStorage.clear();
});

describe("useFilePreview", () => {
  it("ouvre le fichier complet sans réutiliser une diff du même chemin", () => {
    const path = "/repo/src/test_ui_card.tsx";
    const operations = [
      operation({ id: "write-large", path, additions: 37, deletions: 0, type: "write" }),
      operation({ id: "edit-small", path, additions: 3, deletions: 3, type: "edit" }),
    ];

    const { result } = renderHook(() => useFilePreview("session-1", operations));

    act(() => {
      result.current.openFullPath(path);
    });

    expect(result.current.activeTab).toBe(`read:${path}`);
    expect(result.current.tabs[0]).toEqual(expect.objectContaining({
      id: `read:${path}`,
      path,
      type: "read",
      additions: 0,
      deletions: 0,
    }));
  });
});

function operation(overrides: Partial<FileOperation>): FileOperation {
  return {
    id: "op",
    path: "/repo/src/file.ts",
    name: "file.ts",
    type: "edit",
    timestamp: "2026-07-02T00:00:00Z",
    additions: 1,
    deletions: 1,
    oldText: "old",
    newText: "new",
    ...overrides,
  };
}
