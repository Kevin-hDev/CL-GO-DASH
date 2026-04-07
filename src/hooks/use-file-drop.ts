import { useState, useCallback } from "react";
import { readFile, stat } from "@tauri-apps/plugin-fs";

const MAX_FILES = 15;
const MAX_SIZE = 20 * 1024 * 1024;
const IMAGE_EXTS = ["png", "jpg", "jpeg", "gif", "webp"];

export interface DroppedFile {
  name: string;
  path?: string;
  type: string;
  size: number;
  preview?: string;
}

export function useFileDrop() {
  const [files, setFiles] = useState<DroppedFile[]>([]);
  const [dragging, setDragging] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const addFiles = useCallback(async (fileList: FileList | File[]) => {
    const newFiles = Array.from(fileList);
    if (files.length + newFiles.length > MAX_FILES) {
      setError(`Maximum ${MAX_FILES} fichiers`);
      return;
    }
    setError(null);
    const dropped: DroppedFile[] = await Promise.all(
      newFiles.map(async (f) => {
        const isImage = f.type.startsWith("image/");
        const preview = isImage ? await readAsDataUrl(f) : undefined;
        return { name: f.name, type: f.type, size: f.size, preview };
      }),
    );
    setFiles((prev) => [...prev, ...dropped]);
  }, [files.length]);

  const addByPaths = useCallback(async (paths: string[]) => {
    if (files.length + paths.length > MAX_FILES) {
      setError(`Maximum ${MAX_FILES} fichiers`);
      return;
    }
    setError(null);
    const dropped: DroppedFile[] = [];
    for (const p of paths) {
      try {
        const meta = await stat(p);
        const name = p.split("/").pop() ?? p;
        const ext = name.split(".").pop()?.toLowerCase() ?? "";
        const size = meta.size ?? 0;
        if (size > MAX_SIZE) {
          setError(`${name} dépasse 20MB`);
          continue;
        }
        let preview: string | undefined;
        if (IMAGE_EXTS.includes(ext)) {
          const bytes = await readFile(p);
          const blob = new Blob([bytes]);
          preview = URL.createObjectURL(blob);
        }
        dropped.push({ name, path: p, type: ext, size, preview });
      } catch (e: unknown) {
        console.error("Erreur ajout fichier:", e);
      }
    }
    setFiles((prev) => [...prev, ...dropped]);
  }, [files.length]);

  const removeFile = useCallback((index: number) => {
    setFiles((prev) => prev.filter((_, i) => i !== index));
  }, []);

  const clearFiles = useCallback(() => {
    setFiles([]);
    setError(null);
  }, []);

  return { files, dragging, error, addFiles, addByPaths, removeFile, clearFiles, setDragging };
}

function readAsDataUrl(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result as string);
    reader.onerror = reject;
    reader.readAsDataURL(file);
  });
}
