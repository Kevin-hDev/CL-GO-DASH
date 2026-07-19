import { render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { GitDiffPreview } from "../git-diff-preview";
import type { GitDiffPreview as GitDiffData } from "@/types/file-preview";

const readGitDiffPreview = vi.fn<() => Promise<GitDiffData>>();

vi.mock("@/services/file-preview", () => ({
  readGitDiffPreview: () => readGitDiffPreview(),
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

describe("GitDiffPreview", () => {
  it("affiche les anciennes et nouvelles lignes du diff", async () => {
    readGitDiffPreview.mockResolvedValue({
      binary: false,
      truncated: true,
      hunks: [{
        old_start: 4,
        old_lines: 2,
        new_start: 4,
        new_lines: 2,
        lines: [
          { kind: "context", content: "same", old_line: 4, new_line: 4 },
          { kind: "deleted", content: "old", old_line: 5, new_line: null },
          { kind: "added", content: "new", old_line: null, new_line: 5 },
        ],
      }],
    });

    const { container } = render(
      <GitDiffPreview
        path="src/example.txt"
        baseDir="/repo"
        source={{
          kind: "git-diff",
          mode: "working",
          commitId: "a".repeat(40),
          filePath: "src/example.txt",
          expectedBranch: "main",
        }}
      />,
    );

    await waitFor(() => expect(screen.getByText("old")).toBeInTheDocument());
    expect(screen.getByText("new")).toBeInTheDocument();
    expect(container.querySelector(".gdp-hunk-header")).toHaveTextContent("@@ -4,2 +4,2 @@");
    expect(screen.getByText("filePreview.diffTruncated")).toBeInTheDocument();
  });

  it("affiche un renommage sans changement de contenu", async () => {
    readGitDiffPreview.mockResolvedValue({ binary: false, truncated: false, hunks: [] });

    render(
      <GitDiffPreview
        path="src/new.txt"
        baseDir="/repo"
        source={{
          kind: "git-diff",
          mode: "commit",
          commitId: "b".repeat(40),
          filePath: "src/new.txt",
          previousPath: "src/old.txt",
          expectedBranch: "main",
        }}
      />,
    );

    await waitFor(() => expect(screen.getByText("src/old.txt")).toBeInTheDocument());
    expect(screen.getByText("src/new.txt")).toBeInTheDocument();
    expect(screen.queryByText("filePreview.diffUnavailable")).not.toBeInTheDocument();
  });
});
