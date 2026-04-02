import { useState, useEffect, useCallback } from "react";
import type { PersonalityFile } from "@/types/personality";
import * as api from "@/services/personality";
import { PersonalityList } from "./personality-list";
import { MarkdownViewer } from "./markdown-viewer";

export function PersonalityTab(): { list: React.ReactNode; detail: React.ReactNode } {
  const [files, setFiles] = useState<PersonalityFile[]>([]);
  const [selectedPath, setSelectedPath] = useState<string | null>(null);
  const [content, setContent] = useState<string>("");
  const [fileName, setFileName] = useState<string>("");

  useEffect(() => {
    api.listFiles().then(setFiles).catch(console.error);
  }, []);

  const handleSelect = useCallback(async (path: string) => {
    setSelectedPath(path);
    try {
      const text = await api.readFile(path);
      setContent(text);
      const name = path.split("/").pop() ?? "";
      setFileName(name);
    } catch (e) {
      console.error("Failed to read:", e);
      setContent("Erreur de lecture");
    }
  }, []);

  const handleOpen = useCallback(() => {
    if (selectedPath) {
      api.openInEditor(selectedPath).catch(console.error);
    }
  }, [selectedPath]);

  const list = (
    <PersonalityList
      files={files}
      selectedPath={selectedPath}
      onSelect={handleSelect}
    />
  );

  let detail: React.ReactNode;
  if (!selectedPath) {
    detail = (
      <div style={{ padding: "var(--space-lg)", color: "var(--ink-faint)" }}>
        Sélectionne un fichier
      </div>
    );
  } else {
    detail = (
      <MarkdownViewer
        content={content}
        fileName={fileName}
        onOpenEditor={handleOpen}
      />
    );
  }

  return { list, detail };
}
