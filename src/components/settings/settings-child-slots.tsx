import { memo, useEffect } from "react";
import { createPortal } from "react-dom";
import { OllamaTab } from "@/components/ollama/ollama-tab";
import { ApiKeysTab } from "@/components/api-keys/api-keys-tab";
import { ConnectorsTab } from "@/components/connectors/connectors-tab";
import { ChannelsTab } from "@/components/channels/channels-tab";
import { ForecastTab } from "@/components/forecast/model-browser/forecast-tab";
import type { TabSlots } from "@/components/agent-local/agent-local-tab-types";
import type { DeepPartial, SettingsNavState, SettingsSubTab } from "@/types/navigation";
import { recordFrontendDiagnostic } from "@/lib/frontend-diagnostics";

interface SlotPortalProps {
  navState: SettingsNavState;
  onNavChange: (partial: DeepPartial<SettingsNavState>) => void;
  onNavReplace: (partial: DeepPartial<SettingsNavState>) => void;
  listTarget: HTMLElement;
  detailTarget: HTMLElement;
}

interface SettingsChildSlotsProps extends Omit<SlotPortalProps, "listTarget" | "detailTarget"> {
  subTab: SettingsSubTab;
  listTarget: HTMLElement | null;
  detailTarget: HTMLElement | null;
}

type SlotFactory = (props: Omit<SlotPortalProps, "listTarget" | "detailTarget">) => TabSlots;

const OllamaSlotPortal = createSlotPortal("ollama", OllamaTab);
const ForecastSlotPortal = createSlotPortal("forecast", ForecastTab);
const ConnectorsSlotPortal = createSlotPortal("connectors", ConnectorsTab);
const ChannelsSlotPortal = createSlotPortal("channels", ChannelsTab);
const ApiKeysSlotPortal = createSlotPortal("api-keys", ApiKeysTab);

export function usesSettingsChildSlots(subTab: SettingsSubTab): boolean {
  return subTab === "ollama"
    || subTab === "connectors"
    || subTab === "channels"
    || subTab === "api-keys"
    || subTab === "forecast";
}

export function SettingsChildSlots({
  subTab,
  listTarget,
  detailTarget,
  ...props
}: SettingsChildSlotsProps) {
  if (!listTarget || !detailTarget) return null;
  const portalProps = { ...props, listTarget, detailTarget };
  if (subTab === "ollama") return <OllamaSlotPortal {...portalProps} />;
  if (subTab === "connectors") return <ConnectorsSlotPortal {...portalProps} />;
  if (subTab === "channels") return <ChannelsSlotPortal {...portalProps} />;
  if (subTab === "api-keys") return <ApiKeysSlotPortal {...portalProps} />;
  if (subTab === "forecast") return <ForecastSlotPortal {...portalProps} />;
  return null;
}

function createSlotPortal(name: SettingsSubTab, factory: SlotFactory) {
  return memo(function SlotPortal({
    navState,
    onNavChange,
    onNavReplace,
    listTarget,
    detailTarget,
  }: SlotPortalProps) {
    const slots = factory({ navState, onNavChange, onNavReplace });
    useEffect(() => {
      recordFrontendDiagnostic("settings.child-portal-render", {
        subTab: name,
        hasList: Boolean(slots.list),
        hasDetail: Boolean(slots.detail),
      });
    }, [slots]);

    return (
      <>
        {createPortal(slots.list, listTarget)}
        {createPortal(slots.detail, detailTarget)}
      </>
    );
  });
}
