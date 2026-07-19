import { describe, expect, it } from "vitest";
import { commitFileOperation, uncommittedFileOperations } from "../git-file-preview";

describe("git-file-preview", () => {
  it("lit un fichier supprimé depuis le parent du commit", () => {
    const operation = commitFileOperation(
      { id: "a".repeat(40), short_id: "aaaaaaaa", message: "delete", timestamp: 1 },
      { path: "src/old.ts", status: "deleted", additions: 0, deletions: 4 },
      "main",
    );

    expect(operation.source).toEqual(expect.objectContaining({
      filePath: "src/old.ts",
      expectedBranch: "main",
      useParent: true,
    }));
    expect(operation.gitDiff).toEqual(expect.objectContaining({
      mode: "commit",
      commitId: "a".repeat(40),
      filePath: "src/old.ts",
    }));
  });

  it("borne et mappe les changements non commit", () => {
    const files = Array.from({ length: 205 }, (_, index) => ({
      path: `file-${index}.txt`,
      status: "modified",
      additions: index,
      deletions: 0,
    }));
    const operations = uncommittedFileOperations({ head_commit: "b".repeat(40), files }, "feature");

    expect(operations).toHaveLength(200);
    expect(operations[0]).toEqual(expect.objectContaining({
      id: "git-uncommitted:feature:file-0.txt",
      path: "file-0.txt",
    }));
    expect(operations[0]?.gitDiff).toEqual(expect.objectContaining({
      mode: "working",
      commitId: "b".repeat(40),
      expectedBranch: "feature",
    }));
  });

  it("conserve l'ancien chemin d'un fichier renommé", () => {
    const [operation] = uncommittedFileOperations({
      head_commit: "c".repeat(40),
      files: [{
        path: "src/new.ts",
        previous_path: "src/old.ts",
        status: "renamed",
        additions: 1,
        deletions: 1,
      }],
    }, "feature");

    expect(operation.gitDiff?.previousPath).toBe("src/old.ts");
  });
});
