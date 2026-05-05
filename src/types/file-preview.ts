export type FileOperationType = "read" | "write" | "edit";

export interface FileOperation {
  id: string;
  path: string;
  name: string;
  type: FileOperationType;
  timestamp: string;
  content?: string;
  oldText?: string;
  newText?: string;
  startLine?: number;
  additions: number;
  deletions: number;
}

export interface PreviewEditor {
  name: string;
  path: string;
  is_default: boolean;
}

export type FilePreviewActiveTab = string;
