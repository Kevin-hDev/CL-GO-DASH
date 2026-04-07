import { useState, useCallback } from "react";

const MAX_FILES = 15;
const MAX_SIZE = 20 * 1024 * 1024;
const IMAGE_TYPES = ["image/png", "image/jpeg", "image/gif", "image/webp"];

export interface DroppedFile {
  file: File;
  name: string;
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
    const total = files.length + newFiles.length;

    if (total > MAX_FILES) {
      setError(`Maximum ${MAX_FILES} fichiers`);
      return;
    }

    const oversized = newFiles.find((f) => f.size > MAX_SIZE);
    if (oversized) {
      setError(`${oversized.name} dépasse 20MB`);
      return;
    }

    setError(null);
    const dropped: DroppedFile[] = await Promise.all(
      newFiles.map(async (f) => {
        const isImage = IMAGE_TYPES.includes(f.type);
        const preview = isImage ? await readAsDataUrl(f) : undefined;
        return { file: f, name: f.name, type: f.type, size: f.size, preview };
      }),
    );
    setFiles((prev) => [...prev, ...dropped]);
  }, [files.length]);

  const removeFile = useCallback((index: number) => {
    setFiles((prev) => prev.filter((_, i) => i !== index));
  }, []);

  const clearFiles = useCallback(() => {
    setFiles([]);
    setError(null);
  }, []);

  return { files, dragging, error, addFiles, removeFile, clearFiles, setDragging };
}

function readAsDataUrl(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result as string);
    reader.onerror = reject;
    reader.readAsDataURL(file);
  });
}
