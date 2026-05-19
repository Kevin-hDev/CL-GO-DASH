import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, cleanup, fireEvent } from "@testing-library/react";

afterEach(cleanup);
import { ToolBubble } from "../tool-bubble";

vi.mock("@phosphor-icons/react", () => ({
  Spinner: () => <span data-testid="spinner" />,
}));
vi.mock("@/components/ui/icons", () => ({
  Copy: () => <span />,
  Check: () => <span data-testid="check-icon" />,
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


describe("ToolBubble", () => {
  it("retourne null si tools est vide", () => {
    const { container } = render(<ToolBubble tools={[]} />);
    expect(container.innerHTML).toBe("");
  });

  it("affiche le nom de l'outil bash", () => {
    const { container } = render(
      <ToolBubble tools={[{ name: "bash", args: { command: "ls -la" } }]} />,
    );
    expect(container.textContent).toContain("bash");
  });

  it("affiche le résumé (command) pour bash", () => {
    const { container } = render(
      <ToolBubble tools={[{ name: "bash", args: { command: "echo hello" } }]} />,
    );
    expect(container.textContent).toContain("echo hello");
  });

  it("affiche le spinner quand result est absent (en cours)", () => {
    const { getByTestId } = render(
      <ToolBubble tools={[{ name: "bash", args: { command: "sleep 5" } }]} />,
    );
    expect(getByTestId("spinner")).toBeTruthy();
  });

  it("affiche l'état terminé quand result est présent et pas d'erreur", () => {
    const { getByTestId } = render(
      <ToolBubble
        tools={[{ name: "bash", args: { command: "ls" }, result: "fichier.txt", isError: false }]}
      />,
    );
    expect(getByTestId("check-icon")).toBeTruthy();
  });

  it("affiche x quand isError vaut true", () => {
    const { container } = render(
      <ToolBubble
        tools={[{ name: "bash", args: { command: "exit 1" }, result: "erreur", isError: true }]}
      />,
    );
    expect(container.textContent).toContain("x");
  });

  it("affiche ContentPreview pour write_file avec content string", () => {
    const { container, getByTestId, queryByTestId } = render(
      <ToolBubble
        tools={[{ name: "write_file", args: { path: "/tmp/foo.ts", content: "const x = 1;" }, result: "ok" }]}
      />,
    );
    expect(queryByTestId("content-preview")).toBeNull();
    const toggle = container.querySelector(".tb-toggle");
    if (!toggle) throw new Error("toggle absent");
    fireEvent.click(toggle);
    expect(getByTestId("content-preview")).toBeTruthy();
  });

  it("affiche DiffPreview pour edit_file avec old_string et new_string", () => {
    const { container, getByTestId, queryByTestId } = render(
      <ToolBubble
        tools={[{
          name: "edit_file",
          args: { path: "/tmp/foo.ts", old_string: "const x = 1;", new_string: "const x = 2;" },
          result: "ok",
        }]}
      />,
    );
    expect(queryByTestId("diff-preview")).toBeNull();
    const toggle = container.querySelector(".tb-toggle");
    if (!toggle) throw new Error("toggle absent");
    fireEvent.click(toggle);
    expect(getByTestId("diff-preview")).toBeTruthy();
  });

  it("n'affiche pas ContentPreview pour write_file si précédé d'edit_file sur le même path (skipWrite)", () => {
    const { queryByTestId } = render(
      <ToolBubble
        tools={[
          { name: "edit_file", args: { path: "/tmp/foo.ts", old_string: "a", new_string: "b" }, result: "ok" },
          { name: "write_file", args: { path: "/tmp/foo.ts", content: "const x = 2;" }, result: "ok" },
        ]}
      />,
    );
    expect(queryByTestId("content-preview")).toBeNull();
  });

  it("affiche WebResultsPreview pour web_search avec un résultat", () => {
    const { container, getByTestId, queryByTestId } = render(
      <ToolBubble
        tools={[{ name: "web_search", args: { query: "vitest jsdom" }, result: '{"results":[]}' }]}
      />,
    );
    expect(queryByTestId("web-preview")).toBeNull();
    const toggle = container.querySelector(".tb-toggle");
    if (!toggle) throw new Error("toggle absent");
    fireEvent.click(toggle);
    expect(getByTestId("web-preview")).toBeTruthy();
  });

  it("garde les previews fermées par défaut", () => {
    const { container } = render(
      <ToolBubble
        tools={[{ name: "write_file", args: { path: "/tmp/foo.ts", content: "const x = 1;" }, result: "ok" }]}
      />,
    );
    expect(container.querySelector(".tb-accordion.tb-open")).toBeNull();

    const toggle = container.querySelector(".tb-toggle");
    expect(toggle).toBeTruthy();
    if (!toggle) throw new Error("toggle absent");
    fireEvent.click(toggle);
    expect(container.querySelector(".tb-accordion.tb-open")).toBeTruthy();
  });
});
