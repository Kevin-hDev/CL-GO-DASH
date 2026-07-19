import { renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { useGitBranch } from "@/hooks/use-git-branch";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));

describe("useGitBranch", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("distingue une erreur de statut distant de l'absence de remote", async () => {
    mockGitCommands(true);
    const { result } = renderHook(() => useGitBranch("/repo"));

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    expect(result.current.isGitRepo).toBe(true);
    expect(result.current.hasRemote).toBe(false);
    expect(result.current.remoteStatusError).toBe(true);
  });

  it("arrête le watcher précédent au changement de projet et au démontage", async () => {
    mockGitCommands(false);
    const view = renderHook(
      ({ path }: { path?: string }) => useGitBranch(path),
      { initialProps: { path: "/repo" } },
    );

    await waitFor(() => expect(invoke).toHaveBeenCalledWith("start_git_watcher", { path: "/repo" }));
    view.rerender({ path: "/other" });
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("stop_git_watcher", { path: "/repo" });
      expect(invoke).toHaveBeenCalledWith("start_git_watcher", { path: "/other" });
    });

    view.unmount();
    expect(invoke).toHaveBeenCalledWith("stop_git_watcher", { path: "/other" });
  });
});

function mockGitCommands(remoteFailure: boolean) {
  vi.mocked(invoke).mockImplementation((command) => {
    switch (command) {
      case "list_git_branches": return Promise.resolve([]);
      case "get_git_context":
        return Promise.resolve({ branch: "main", is_detached: false, dirty_count: 0, is_git_repo: true });
      case "list_git_worktrees": return Promise.resolve([]);
      case "get_git_remote_status":
        return remoteFailure
          ? Promise.reject(new Error("unavailable"))
          : Promise.resolve({
            has_remote: true,
            is_github: true,
            has_remote_branch: true,
            ahead: 0,
            behind: 0,
          });
      case "start_git_watcher":
      case "stop_git_watcher":
        return Promise.resolve();
      default:
        return Promise.reject(new Error("unexpected command"));
    }
  });
}
