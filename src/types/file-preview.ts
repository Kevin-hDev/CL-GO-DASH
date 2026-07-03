export type FileOperationType = "read" | "write" | "edit";
export type FileOperationKind = "file" | "plan";

export interface FileOperation {
  id: string;
  path: string;
  name: string;
  type: FileOperationType;
  kind?: FileOperationKind;
  timestamp: string;
  content?: string;
  oldText?: string;
  newText?: string;
  startLine?: number;
  additions: number;
  deletions: number;
}

export interface FileOperationGroups {
  all: FileOperation[];
  latest: FileOperation[];
}

export interface PreviewEditor {
  name: string;
  path: string;
  is_default: boolean;
}

export type FilePreviewActiveTab = string;
export type FilePreviewListMode = "latest" | "all";
