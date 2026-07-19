type FileOperationType = "read" | "write" | "edit";
type FileOperationKind = "file" | "plan";

export interface GitFilePreviewSource {
  kind: "git";
  commitId: string;
  filePath: string;
  expectedBranch: string;
  useParent: boolean;
}

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
  source?: GitFilePreviewSource;
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
export type FilePreviewListMode = "latest" | "all" | "uncommitted";
