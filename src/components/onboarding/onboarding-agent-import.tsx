import { AgentImportWizard } from "@/components/agent-import/agent-import-wizard";

interface OnboardingAgentImportProps {
  onNext: () => void;
}

export function OnboardingAgentImport({ onNext }: OnboardingAgentImportProps) {
  return (
    <div className="ob-page">
      <div className="aim-onboarding-shell">
        <AgentImportWizard onContinue={onNext} />
      </div>
    </div>
  );
}
