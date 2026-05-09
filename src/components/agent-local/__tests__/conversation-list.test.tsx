import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, fireEvent } from "@testing-library/react";
import { ConversationList } from "../conversation-list";
import type { AgentSessionMeta, Project } from "@/types/agent";

function makeSession(overrides: Partial<AgentSessionMeta> = {}): AgentSessionMeta {
  return { id: "s1", name: "Test", model: "llama3", provider: "ollama", message_count: 5, created_at: "2026-01-01", ...overrides };
}
function makeProject(overrides: Partial<Project> = {}): Project {
  return { id: "p1", name: "Mon Projet", path: "/tmp/proj", created_at: "2026-01-01", ...overrides };
}
const defaultProps = {
  sessions: [] as AgentSessionMeta[], projects: [] as Project[], selectedId: null as string | null,
  onSelect: vi.fn(), onCreate: vi.fn(), onRename: vi.fn(), onDelete: vi.fn(),
  onNewSessionInProject: vi.fn(), onRenameProject: vi.fn(), onDeleteProject: vi.fn(),
  onOpenFolder: vi.fn(), onReorderProjects: vi.fn(),
};

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

vi.mock("@/components/ui/icons", () => ({
  Pencil: () => <span />,
  DotsThreeVertical: () => <span data-testid="dots" />,
  ChatsCircle: (props: { weight?: string }) => (
    <span data-testid="chat-icon" data-weight={props.weight} />
  ),
  FolderOpen: () => <span />,
  FolderSimple: () => <span />,
  PencilSimple: () => <span />,
}));

vi.mock("@/components/ui/wastebasket-icon", () => ({
  WastebasketIcon: () => <span />,
}));

vi.mock("@/components/ui/compose-icon", () => ({
  ComposeIcon: () => <span data-testid="compose" />,
}));

vi.mock("@/components/ui/context-menu", () => ({
  ContextMenu: () => <div data-testid="context-menu" />,
}));

vi.mock("../project-section", () => ({
  ProjectSection: (props: { project: Project }) => (
    <div data-testid={`project-${props.project.id}`}>{props.project.name}</div>
  ),
}));

vi.mock("@/hooks/use-keyboard", () => ({
  useKeyboard: () => {},
}));

vi.mock("@/hooks/use-project-drag", () => ({
  useProjectDrag: () => ({
    draggingId: null,
    liveOrder: null,
    onGrab: vi.fn(),
    onHover: vi.fn(),
    onRelease: vi.fn(),
    onCancel: vi.fn(),
  }),
}));

vi.mock("../conversation.css", () => ({}));

describe("ConversationList", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("affiche le bouton nouveau avec la classe .conv-new-btn", () => {
    const { container } = render(<ConversationList {...defaultProps} />);
    expect(container.querySelector(".conv-new-btn")).not.toBeNull();
  });

  it("affiche le message vide si aucune session et aucun projet", () => {
    const { container } = render(<ConversationList {...defaultProps} />);
    const empty = container.querySelector(".hist-empty");
    expect(empty).not.toBeNull();
    expect(empty?.textContent).toContain("agentLocal.noConversations");
  });

  it("affiche les sessions orphelines sans project_id", () => {
    const session = makeSession({ id: "s1", name: "Session orpheline" });
    const { container } = render(
      <ConversationList {...defaultProps} sessions={[session]} />,
    );
    expect(container.querySelectorAll(".conv-session-indented").length).toBe(1);
  });

  it("n'affiche pas les sous-sessions avec parent_session_id", () => {
    const parent = makeSession({ id: "parent" });
    const child = makeSession({ id: "child", parent_session_id: "parent" });
    const { container } = render(
      <ConversationList {...defaultProps} sessions={[parent, child]} />,
    );
    expect(container.querySelectorAll(".conv-session-indented").length).toBe(1);
  });

  it("marque la session active avec la classe .active", () => {
    const session = makeSession({ id: "s1" });
    const { container } = render(
      <ConversationList {...defaultProps} sessions={[session]} selectedId="s1" />,
    );
    expect(container.querySelector(".conv-session-indented.active")).not.toBeNull();
  });

  it("ne marque pas les autres sessions comme .active", () => {
    const s1 = makeSession({ id: "s1" });
    const s2 = makeSession({ id: "s2", name: "Autre" });
    const { container } = render(
      <ConversationList {...defaultProps} sessions={[s1, s2]} selectedId="s1" />,
    );
    const allItems = container.querySelectorAll(".conv-session-indented");
    const inactives = Array.from(allItems).filter((el) => !el.classList.contains("active"));
    expect(inactives.length).toBe(1);
  });

  it("appelle onSelect au clic sur une session", () => {
    const onSelect = vi.fn();
    const session = makeSession({ id: "s42" });
    const { container } = render(
      <ConversationList {...defaultProps} sessions={[session]} onSelect={onSelect} />,
    );
    fireEvent.click(container.querySelector(".conv-session-indented") as HTMLElement);
    expect(onSelect).toHaveBeenCalledWith("s42");
  });

  it("appelle onCreate au clic sur le bouton nouveau", () => {
    const onCreate = vi.fn();
    const { container } = render(
      <ConversationList {...defaultProps} onCreate={onCreate} />,
    );
    fireEvent.click(container.querySelector(".conv-new-btn") as HTMLElement);
    expect(onCreate).toHaveBeenCalledOnce();
  });

  it("affiche les ProjectSection pour chaque projet", () => {
    const p1 = makeProject({ id: "p1", name: "Projet A" });
    const p2 = makeProject({ id: "p2", name: "Projet B" });
    const { getByTestId } = render(
      <ConversationList {...defaultProps} projects={[p1, p2]} />,
    );
    expect(getByTestId("project-p1")).not.toBeNull();
    expect(getByTestId("project-p2")).not.toBeNull();
  });

  it("affiche comme orpheline une session dont le project_id ne correspond à aucun projet", () => {
    const session = makeSession({ id: "s1", project_id: "projet-inconnu" });
    const { container } = render(
      <ConversationList {...defaultProps} sessions={[session]} projects={[]} />,
    );
    expect(container.querySelectorAll(".conv-session-indented").length).toBe(1);
  });

  it("n'affiche pas dans les orphelins une session assignée à un projet existant", () => {
    const projet = makeProject({ id: "p1" });
    const session = makeSession({ id: "s1", project_id: "p1" });
    const { container } = render(
      <ConversationList {...defaultProps} sessions={[session]} projects={[projet]} />,
    );
    expect(container.querySelectorAll(".conv-session-indented").length).toBe(0);
  });

  it("affiche la clé i18n pour une session nommée 'Nouvelle session'", () => {
    const session = makeSession({ id: "s1", name: "Nouvelle session" });
    const { container } = render(
      <ConversationList {...defaultProps} sessions={[session]} />,
    );
    expect(container.querySelector(".conv-name")?.textContent).toBe("agentLocal.newSession");
  });

  it("affiche l'icône chat avec weight=fill pour la session active", () => {
    const session = makeSession({ id: "s1" });
    const { container } = render(
      <ConversationList {...defaultProps} sessions={[session]} selectedId="s1" />,
    );
    // L'icône de la session active doit avoir weight=fill
    const activeItem = container.querySelector(".conv-session-indented.active");
    const icon = activeItem?.querySelector("[data-testid='chat-icon']");
    expect(icon?.getAttribute("data-weight")).toBe("fill");
  });

  it("affiche l'icône chat avec weight=regular pour les sessions inactives", () => {
    const s1 = makeSession({ id: "s1" });
    const s2 = makeSession({ id: "s2", name: "Autre" });
    const { getAllByTestId } = render(
      <ConversationList {...defaultProps} sessions={[s1, s2]} selectedId="s1" />,
    );
    const weights = getAllByTestId("chat-icon").map((el) => el.getAttribute("data-weight"));
    expect(weights).toContain("fill");
    expect(weights).toContain("regular");
  });

  it("n'affiche pas les sous-agents (parent_session_id défini) même sans project_id", () => {
    // Un sous-agent est filtré de la liste visible (mainSessions exclut parent_session_id)
    // Mais sessions.length > 0 donc le message "aucune conversation" n'apparaît pas
    const parent = makeSession({ id: "parent" });
    const subAgent = makeSession({ id: "sub", parent_session_id: "parent" });
    const { container } = render(
      <ConversationList {...defaultProps} sessions={[parent, subAgent]} />,
    );
    // Seul le parent apparaît dans la liste orpheline, pas le sous-agent
    const items = container.querySelectorAll(".conv-session-indented");
    expect(items.length).toBe(1);
    expect(items[0].getAttribute("data-session-id") ?? items[0].textContent).not.toContain("sub");
  });
});
