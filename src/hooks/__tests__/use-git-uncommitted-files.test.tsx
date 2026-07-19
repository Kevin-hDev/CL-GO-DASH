import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useGitUncommittedFiles } from "../use-git-uncommitted-files";

let gitChanged: (() => void) | undefined;

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn((_event: string, callback: () => void) => {
    gitChanged = callback;
    return Promise.resolve(() => {});
  }),
}));

beforeEach(() => {
  gitChanged = undefined;
  vi.clearAllMocks();
});

describe("useGitUncommittedFiles", () => {
  it("actualise les fichiers après un nouvel événement Git", async () => {
    const listUncommittedFiles = vi.fn()
      .mockResolvedValueOnce(snapshot("first.ts"))
      .mockResolvedValueOnce(snapshot("edited.ts"));
    const git = {
      isGitRepo: true,
      currentBranch: "main",
      dirtyCount: 1,
      listUncommittedFiles,
    };
    const { result } = renderHook(() => useGitUncommittedFiles(git));

    await waitFor(() => expect(result.current[0]?.path).toBe("first.ts"));
    act(() => gitChanged?.());
    await waitFor(() => expect(result.current[0]?.path).toBe("edited.ts"));

    expect(listUncommittedFiles).toHaveBeenCalledTimes(2);
  });
});

function snapshot(path: string) {
  return {
    head_commit: "a".repeat(40),
    files: [{ path, status: "modified", additions: 1, deletions: 0 }],
  };
}
