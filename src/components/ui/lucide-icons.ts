import { createElement, forwardRef } from "react";
import type { LucideIcon, LucideProps } from "lucide-react";
import {
  Archive as LuArchive,
  BookOpen as LuBookOpen,
  Check as LuCheck,
  CheckCircle2 as LuCheckCircle2,
  ChevronDown as LuChevronDown,
  ChevronRight as LuChevronRight,
  Circle as LuCircle,
  Clock3 as LuClock3,
  FolderTree as LuFolderTree,
  HelpCircle as LuHelpCircle,
  List as LuList,
  ListChecks as LuListChecks,
  Maximize2 as LuMaximize2,
  Minimize2 as LuMinimize2,
  PanelRightClose as LuPanelRightClose,
  PanelRightOpen as LuPanelRightOpen,
  Plus as LuPlus,
  Search as LuSearch,
  TerminalSquare as LuTerminalSquare,
  X as LuX,
} from "lucide-react";

function withSafeCssSize(Component: LucideIcon): LucideIcon {
  const Wrapped = forwardRef<SVGSVGElement, LucideProps>(({ size, style, ...props }, ref) => {
    if (typeof size !== "string") {
      return createElement(Component, { ...props, ref, size, style });
    }
    return createElement(Component, {
      ...props,
      ref,
      style: { width: size, height: size, ...style },
    });
  });
  Wrapped.displayName = Component.displayName ?? Component.name ?? "LucideIcon";
  return Wrapped as LucideIcon;
}

export const Archive = withSafeCssSize(LuArchive);
export const BookOpen = withSafeCssSize(LuBookOpen);
export const Check = withSafeCssSize(LuCheck);
export const CheckCircle2 = withSafeCssSize(LuCheckCircle2);
export const ChevronDown = withSafeCssSize(LuChevronDown);
export const ChevronRight = withSafeCssSize(LuChevronRight);
export const Circle = withSafeCssSize(LuCircle);
export const Clock3 = withSafeCssSize(LuClock3);
export const FolderTree = withSafeCssSize(LuFolderTree);
export const HelpCircle = withSafeCssSize(LuHelpCircle);
export const List = withSafeCssSize(LuList);
export const ListChecks = withSafeCssSize(LuListChecks);
export const Maximize2 = withSafeCssSize(LuMaximize2);
export const Minimize2 = withSafeCssSize(LuMinimize2);
export const PanelRightClose = withSafeCssSize(LuPanelRightClose);
export const PanelRightOpen = withSafeCssSize(LuPanelRightOpen);
export const Plus = withSafeCssSize(LuPlus);
export const Search = withSafeCssSize(LuSearch);
export const TerminalSquare = withSafeCssSize(LuTerminalSquare);
export const X = withSafeCssSize(LuX);
