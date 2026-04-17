import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Project } from "@/types/agent";

export function useProjects() {
  const [projects, setProjects] = useState<Project[]>([]);

  const refresh = useCallback(async () => {
    const list = await invoke<Project[]>("list_projects");
    setProjects(list.sort((a, b) => a.order - b.order));
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const add = useCallback(
    async (path: string): Promise<Project> => {
      const project = await invoke<Project>("add_project", { path });
      await refresh();
      return project;
    },
    [refresh],
  );

  const rename = useCallback(
    async (id: string, name: string) => {
      await invoke("rename_project", { id, name });
      await refresh();
    },
    [refresh],
  );

  const remove = useCallback(
    async (id: string) => {
      await invoke("delete_project", { id });
      await refresh();
    },
    [refresh],
  );

  const reorder = useCallback(
    async (ids: string[]) => {
      await invoke("reorder_projects", { ids });
      await refresh();
    },
    [refresh],
  );

  const openFolder = useCallback(async (path: string) => {
    await invoke("open_project_folder", { path });
  }, []);

  return { projects, refresh, add, rename, remove, reorder, openFolder };
}
