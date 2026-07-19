export type GitCreateBranchErrorKind =
  | "invalid_name"
  | "name_too_long"
  | "already_exists"
  | "unborn_head"
  | "github_auth_required"
  | "internal_error";

export type GitCreateBranchResult =
  | { ok: true }
  | { ok: false; reason?: "github_auth_required"; kind?: GitCreateBranchErrorKind };

const CREATE_BRANCH_ERROR_KINDS = new Set<GitCreateBranchErrorKind>([
  "invalid_name",
  "name_too_long",
  "already_exists",
  "unborn_head",
  "github_auth_required",
  "internal_error",
]);

export function parseCreateBranchError(error: unknown): GitCreateBranchErrorKind | null {
  const fromObject = readCreateBranchKind(error);
  if (fromObject) return fromObject;
  if (typeof error !== "string") return null;
  if (error.includes("GITHUB_AUTH_REQUIRED")) return "github_auth_required";
  try {
    return readCreateBranchKind(JSON.parse(error));
  } catch {
    return null;
  }
}

function readCreateBranchKind(value: unknown): GitCreateBranchErrorKind | null {
  if (!value || typeof value !== "object") return null;
  const kind = (value as { kind?: unknown }).kind;
  if (typeof kind !== "string") return null;
  return CREATE_BRANCH_ERROR_KINDS.has(kind as GitCreateBranchErrorKind)
    ? kind as GitCreateBranchErrorKind
    : null;
}
