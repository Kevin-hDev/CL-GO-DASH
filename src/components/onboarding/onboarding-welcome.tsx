import { useTranslation } from "react-i18next";
import logoDark from "@/assets/logo-dark.png";
import logoLight from "@/assets/logo-light.png";
import { ThemedIcon } from "@/components/ui/themed-icon";
import { CaretRight } from "@/components/ui/icons";

interface OnboardingWelcomeProps {
  onNext: () => void;
}

export function OnboardingWelcome({ onNext }: OnboardingWelcomeProps) {
  const { t } = useTranslation();

  return (
    <div className="ob-page ob-page-centered">
      <ThemedIcon darkSrc={logoDark} lightSrc={logoLight} size="4.5rem" alt="CL-GO" />
      <div className="ob-copy">
        <h1 className="ob-title">{t("onboarding.welcome.title")}</h1>
        <p className="ob-description">{t("onboarding.welcome.description")}</p>
      </div>
      <button type="button" className="ob-primary-btn" onClick={onNext}>
        {t("onboarding.welcome.getStarted")}
        <CaretRight size="var(--icon-md)" weight="bold" />
      </button>
    </div>
  );
}
