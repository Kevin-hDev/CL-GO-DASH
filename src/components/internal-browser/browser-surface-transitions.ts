interface TransitionGuards {
  moving: () => boolean;
  detach: () => void;
}

export function attachBrowserTransitionGuards(
  host: HTMLDivElement,
  onBegin: () => void,
  onEnd: () => void,
): TransitionGuards {
  const wrapper = host.closest<HTMLElement>(".asp-slide-wrapper");
  const panel = host.closest<HTMLElement>(".asp-panel");
  let slideMoving = false;
  let panelMoving = false;
  if (!wrapper || !panel) return { moving: () => false, detach: () => {} };

  const handles = (event: TransitionEvent, target: HTMLElement) => (
    event.target === target && event.propertyName === "transform"
  );
  const beginSlide = (event: TransitionEvent) => {
    if (!handles(event, wrapper)) return;
    slideMoving = true;
    onBegin();
  };
  const endSlide = (event: TransitionEvent) => {
    if (!handles(event, wrapper)) return;
    slideMoving = false;
    onEnd();
  };
  const beginPanel = (event: TransitionEvent) => {
    if (!handles(event, panel)) return;
    panelMoving = true;
    onBegin();
  };
  const endPanel = (event: TransitionEvent) => {
    if (!handles(event, panel)) return;
    panelMoving = false;
    onEnd();
  };
  wrapper.addEventListener("transitionrun", beginSlide);
  wrapper.addEventListener("transitionend", endSlide);
  wrapper.addEventListener("transitioncancel", endSlide);
  panel.addEventListener("transitionrun", beginPanel);
  panel.addEventListener("transitionend", endPanel);
  panel.addEventListener("transitioncancel", endPanel);

  return {
    moving: () => slideMoving || panelMoving,
    detach: () => {
      wrapper.removeEventListener("transitionrun", beginSlide);
      wrapper.removeEventListener("transitionend", endSlide);
      wrapper.removeEventListener("transitioncancel", endSlide);
      panel.removeEventListener("transitionrun", beginPanel);
      panel.removeEventListener("transitionend", endPanel);
      panel.removeEventListener("transitioncancel", endPanel);
    },
  };
}
