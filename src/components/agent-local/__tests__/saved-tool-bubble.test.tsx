import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, cleanup } from "@testing-library/react";

afterEach(cleanup);
import { SavedToolBubble } from "../tool-bubble";

vi.mock("@phosphor-icons/react", () => ({
  Spinner: () => <span data-testid="spinner" />,
}));
vi.mock("@/components/ui/icons", () => ({
  Copy: () => <span />,
  Check: () => <span />,
}));
vi.mock("../tool-previews", () => ({
  ContentPreview: () => <div data-testid="content-preview" />,
  DiffPreview: () => <div data-testid="diff-preview" />,
  WebResultsPreview: () => <div data-testid="web-preview" />,
}));
vi.mock("../tool-office-previews", () => ({
  ReadSpreadsheetPreview: () => <div data-testid="read-spreadsheet-preview" />,
  WriteSpreadsheetPreview: () => <div data-testid="write-spreadsheet-preview" />,
  DocumentResultPreview: () => <div data-testid="document-preview" />,
  WriteDocumentPreview: () => <div data-testid="write-document-preview" />,
}));
vi.mock("../tool-bubble.css", () => ({}));
vi.mock("@/lib/tool-file-path", () => ({
  isFileTool: (name: string) =>
    ["read_file", "write_file", "edit_file", "read_spreadsheet", "read_document",
      "read_image", "write_spreadsheet", "write_document", "process_image"].includes(name),
}));

beforeEach(() => vi.clearAllMocks());

describe("SavedToolBubble", () => {
  it("retourne null si tools est vide", () => {
    const { container } = render(<SavedToolBubble tools={[]} />);
    expect(container.innerHTML).toBe("");
  });

  it("affiche le nom et le summary", () => {
    const { container } = render(
      <SavedToolBubble
        tools={[{ name: "bash", summary: "npm run build", result: "ok" }]}
      />,
    );
    expect(container.textContent).toContain("bash");
    expect(container.textContent).toContain("npm run build");
  });

  it("affiche ContentPreview pour write_file avec content", () => {
    const { getByTestId } = render(
      <SavedToolBubble
        tools={[{ name: "write_file", summary: "/tmp/bar.ts", content: "export const x = 1;", result: "ok" }]}
      />,
    );
    expect(getByTestId("content-preview")).toBeTruthy();
  });

  it("affiche DiffPreview quand old_text et new_text sont présents", () => {
    const { getByTestId } = render(
      <SavedToolBubble
        tools={[{
          name: "edit_file",
          summary: "/tmp/bar.ts",
          old_text: "const a = 1;",
          new_text: "const a = 2;",
          result: "ok",
        }]}
      />,
    );
    expect(getByTestId("diff-preview")).toBeTruthy();
  });

  it("n'applique pas skipWrite si l'edit_file a un summary différent du write_file", () => {
    const { getByTestId } = render(
      <SavedToolBubble
        tools={[
          { name: "edit_file", summary: "/tmp/autre.ts", old_text: "a", new_text: "b", result: "ok" },
          { name: "write_file", summary: "/tmp/bar.ts", content: "export const x = 1;", result: "ok" },
        ]}
      />,
    );
    expect(getByTestId("content-preview")).toBeTruthy();
  });
});
