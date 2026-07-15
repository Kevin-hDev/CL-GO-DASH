import { spawnSync } from "node:child_process";
import { mkdir, rename, rm, stat, writeFile } from "node:fs/promises";
import { posix, resolve } from "node:path";
import { randomBytes } from "node:crypto";
import { tauriDir } from "./cef-artifacts.mjs";

const MAX_ARCHIVE_ENTRIES = 20_000;
const MAX_LIST_BYTES = 16 * 1024 * 1024;

function runTar(args) {
  const result = spawnSync("tar", args, {
    encoding: "utf8",
    maxBuffer: MAX_LIST_BYTES,
    shell: false,
    timeout: 180_000,
    windowsHide: true,
  });
  if (result.status !== 0 || result.error)
    throw new Error("CEF extraction failed");
  return result.stdout;
}

function validateEntries(output, expectedRoot) {
  const entries = output.split(/\r?\n/u).filter(Boolean);
  if (entries.length === 0 || entries.length > MAX_ARCHIVE_ENTRIES) {
    throw new Error("CEF archive is invalid");
  }
  for (const entry of entries) {
    if (entry.length > 512 || entry.includes("\\") || posix.isAbsolute(entry)) {
      throw new Error("CEF archive is invalid");
    }
    const parts = entry.split("/").filter(Boolean);
    if (parts[0] !== expectedRoot || parts.some((part) => part === "..")) {
      throw new Error("CEF archive is invalid");
    }
  }
}

async function validCurrent(current, artifact) {
  try {
    const marker = await import("node:fs/promises").then(({ readFile }) =>
      readFile(resolve(current, ".clgo-sha256"), "utf8"),
    );
    const cmake = await stat(resolve(current, "CMakeLists.txt"));
    return marker.trim() === artifact.sha256 && cmake.isFile();
  } catch {
    return false;
  }
}

export async function extractVerifiedArchive(archive, artifact) {
  const base = resolve(tauriDir, ".cef-verified");
  const current = resolve(base, "current");
  if (await validCurrent(current, artifact)) return current;

  validateEntries(runTar(["-tjf", archive]), artifact.root);
  const nonce = randomBytes(12).toString("hex");
  const temporary = resolve(base, `.extract-${nonce}`);
  const backup = resolve(base, `.previous-${nonce}`);
  await mkdir(temporary, { recursive: false, mode: 0o700 });
  try {
    runTar(["-xjf", archive, "-C", temporary]);
    const extracted = resolve(temporary, artifact.root);
    if (!(await stat(resolve(extracted, "CMakeLists.txt"))).isFile()) {
      throw new Error("CEF extraction failed");
    }
    await writeFile(
      resolve(extracted, ".clgo-sha256"),
      `${artifact.sha256}\n`,
      { mode: 0o600 },
    );
    await writeFile(
      resolve(extracted, "archive.json"),
      `${JSON.stringify({ type: "minimal", name: artifact.root, sha1: "verified-by-sha256" }, null, 2)}\n`,
      { mode: 0o600 },
    );
    await rename(current, backup).catch(() => {});
    await rename(extracted, current);
    await writeFile(resolve(current, ".gitkeep"), "", { mode: 0o600 });
    await rm(backup, { recursive: true, force: true });
  } catch (error) {
    await rename(backup, current).catch(() => {});
    throw error;
  } finally {
    await rm(temporary, { recursive: true, force: true });
  }
  return current;
}
