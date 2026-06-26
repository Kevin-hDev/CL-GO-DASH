import { cleanup, fireEvent, render } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ChatPlusMenu } from "../chat-plus-menu";

vi.mock("@/hooks/use-connectors", () => ({
  useConnectors: () => ({
    configured: [],
    toggleChatEnabled: vi.fn(),
  }),
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => {
      if (key === "chatMenu.addFile") return "Add file";
      if (key === "chatMenu.planMode") return "Plan mode";
      if (key === "chatMenu.planModeDesc") return "Prepare before editing";
      if (key === "chatMenu.connectors") return "Connectors";
      if (key === "chatMenu.plugins") return "Plugins";
      return key;
    },
  }),
}));

vi.mock("../chat-plus-menu.css", () => ({}));

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

describe("ChatPlusMenu plan mode", () => {
  it("affiche et active le toggle Plan mode", () => {
    const onPlanModeChange = vi.fn();
    const { getByText, getAllByRole } = render(
      <ChatPlusMenu
        onFileImport={vi.fn()}
        planModeEnabled={false}
        onPlanModeChange={onPlanModeChange}
      />,
    );

    fireEvent.click(getAllByRole("button")[0]);
    fireEvent.click(getByText("Plan mode"));

    expect(getByText("Prepare before editing")).toBeTruthy();
    expect(onPlanModeChange).toHaveBeenCalledWith(true);
  });

  it("désactive le toggle Plan mode quand il est actif", () => {
    const onPlanModeChange = vi.fn();
    const { getByText, getAllByRole } = render(
      <ChatPlusMenu
        onFileImport={vi.fn()}
        planModeEnabled
        onPlanModeChange={onPlanModeChange}
      />,
    );

    fireEvent.click(getAllByRole("button")[0]);
    fireEvent.click(getByText("Plan mode"));

    expect(onPlanModeChange).toHaveBeenCalledWith(false);
  });
});
