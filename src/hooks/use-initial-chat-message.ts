import { useEffect, useRef } from "react";
import type { DroppedFile } from "@/hooks/use-file-drop";

interface InitialChatParams {
  initialMessage?: string;
  initialWorkingDir?: string;
  initialSkills?: { name: string; content: string }[];
  initialFiles?: DroppedFile[];
  selectedProjectPath?: string;
  selectedProjectId?: string | null;
  sendMessage: (
    text: string,
    files?: { name: string; path?: string; preview?: string }[],
    workingDir?: string,
    projectId?: string,
    skills?: { name: string; content: string }[],
  ) => Promise<void>;
  onSent?: () => void;
}

export function useInitialChatMessage(params: InitialChatParams) {
  const initialSent = useRef(false);

  useEffect(() => {
    const hasInitialContent = params.initialMessage
      || (params.initialFiles && params.initialFiles.length > 0)
      || (params.initialSkills && params.initialSkills.length > 0);
    if (!hasInitialContent || initialSent.current) return;

    initialSent.current = true;
    const files = params.initialFiles?.map((file) => ({
      name: file.name,
      path: file.path,
      preview: file.preview,
    }));
    const workingDir = params.initialWorkingDir ?? params.selectedProjectPath;
    void params.sendMessage(
      params.initialMessage || "",
      files,
      workingDir,
      params.selectedProjectId ?? undefined,
      params.initialSkills,
    ).then(() => params.onSent?.());
  }, [params]);
}
