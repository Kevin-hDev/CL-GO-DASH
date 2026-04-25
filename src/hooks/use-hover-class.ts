import { useRef, useEffect, type RefObject } from "react";

export function useHoverClass(className = "msg-hovered"): RefObject<HTMLDivElement | null> {
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const el = ref.current;
    if (!el) return;

    const onEnter = () => el.classList.add(className);
    const onLeave = () => el.classList.remove(className);

    el.addEventListener("mouseenter", onEnter);
    el.addEventListener("mouseleave", onLeave);
    return () => {
      el.removeEventListener("mouseenter", onEnter);
      el.removeEventListener("mouseleave", onLeave);
      el.classList.remove(className);
    };
  }, [className]);

  return ref;
}
