import { describe, expect, it } from "vitest";
import {
  INITIAL_AGENT_SIDEBAR_LAYOUT_STATE,
  autoHideAgentSidebar,
  setAgentPanelsTight,
  shouldCompactAgentChat,
  toggleAgentSidebar,
} from "../sidebar-compact-state";

describe("sidebar compact state", () => {
  it("garde le chat compact sur les reouvertures successives quand l'espace reste serre", () => {
    let state = autoHideAgentSidebar(INITIAL_AGENT_SIDEBAR_LAYOUT_STATE);

    state = toggleAgentSidebar(state);
    expect(state).toMatchObject({ sidebarOpen: true, manualReveal: true, panelsTight: true });
    expect(shouldCompactAgentChat(state)).toBe(true);

    state = toggleAgentSidebar(state);
    expect(state).toMatchObject({ sidebarOpen: false, manualReveal: false, panelsTight: true });

    state = toggleAgentSidebar(state);
    expect(state).toMatchObject({ sidebarOpen: true, manualReveal: true, panelsTight: true });
    expect(shouldCompactAgentChat(state)).toBe(true);
  });

  it("revient au comportement normal quand l'espace n'est plus serre", () => {
    let state = autoHideAgentSidebar(INITIAL_AGENT_SIDEBAR_LAYOUT_STATE);
    state = setAgentPanelsTight(state, false);
    state = toggleAgentSidebar(state);

    expect(state).toMatchObject({ sidebarOpen: true, manualReveal: false, panelsTight: false });
    expect(shouldCompactAgentChat(state)).toBe(false);
  });
});
