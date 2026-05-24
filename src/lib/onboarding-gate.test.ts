import { describe, expect, it } from "vitest";
import {
  hasCompletedOnboarding,
  onboardingCompletedPatch,
  ONBOARDING_COMPLETED_KEY,
} from "./onboarding-gate";

describe("onboarding gate", () => {
  it("detecte uniquement le booleen true", () => {
    expect(hasCompletedOnboarding({ [ONBOARDING_COMPLETED_KEY]: true })).toBe(true);
    expect(hasCompletedOnboarding({ [ONBOARDING_COMPLETED_KEY]: "true" })).toBe(false);
    expect(hasCompletedOnboarding(null)).toBe(false);
  });

  it("genere le patch de completion", () => {
    expect(onboardingCompletedPatch()).toEqual({ [ONBOARDING_COMPLETED_KEY]: true });
    expect(onboardingCompletedPatch(false)).toEqual({ [ONBOARDING_COMPLETED_KEY]: false });
  });
});
