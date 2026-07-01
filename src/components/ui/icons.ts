import { createElement, forwardRef } from "react";
import type { Icon, IconProps } from "@phosphor-icons/react";
import {
  Pulse as PhPulse,
  ClipboardText as PhClipboardText,
  UserCircle as PhUserCircle,
  ChatCircle as PhChatCircle,
  Sliders as PhSliders,
  Moon as PhMoon,
  Sun as PhSun,
  Fire as PhFire,
  Pencil as PhPencil,
  Trash as PhTrash,
  Play as PhPlay,
  Plus as PhPlus,
  Copy as PhCopy,
  Check as PhCheck,
  Warning as PhWarning,
  ShieldWarning as PhShieldWarning,
  ArrowsClockwise as PhArrowsClockwise,
  Gear as PhGear,
  X as PhX,
  CaretDown as PhCaretDown,
  CaretUp as PhCaretUp,
  CaretRight as PhCaretRight,
  DownloadSimple as PhDownloadSimple,
  FileText as PhFileText,
  Clock as PhClock,
  CaretLeft as PhCaretLeft,
  Key as PhKey,
  MagnifyingGlass as PhMagnifyingGlass,
  ArrowSquareOut as PhArrowSquareOut,
  Star as PhStar,
  FolderSimple as PhFolderSimple,
  FolderSimplePlus as PhFolderSimplePlus,
  FolderOpen as PhFolderOpen,
  DotsThreeVertical as PhDotsThreeVertical,
  Hand as PhHand,
  PencilSimple as PhPencilSimple,
  ChatCircleDots as PhChatCircleDots,
  ChatsCircle as PhChatsCircle,
  GearSix as PhGearSix,
  Info as PhInfo,
  Keyboard as PhKeyboard,
  BookOpenText as PhBookOpenText,
  Plugs as PhPlugs,
  PuzzlePiece as PhPuzzlePiece,
  Image as PhImage,
  ToggleLeft as PhToggleLeft,
  ToggleRight as PhToggleRight,
  GitBranch as PhGitBranch,
  GitFork as PhGitFork,
  GitMerge as PhGitMerge,
  Broadcast as PhBroadcast,
  ChatTeardropDots as PhChatTeardropDots,
  Hash as PhHash,
  ChartLineUp as PhChartLineUp,
  Brain as PhBrain,
} from "@phosphor-icons/react";

function withSafeCssSize(Component: Icon): Icon {
  const Wrapped = forwardRef<SVGSVGElement, IconProps>(({ size, style, ...props }, ref) => {
    if (typeof size !== "string") {
      return createElement(Component, { ...props, ref, size, style });
    }
    return createElement(Component, {
      ...props,
      ref,
      style: { width: size, height: size, ...style },
    });
  });
  Wrapped.displayName = Component.displayName ?? Component.name ?? "Icon";
  return Wrapped;
}

export const Pulse = withSafeCssSize(PhPulse);
export const ClipboardText = withSafeCssSize(PhClipboardText);
export const UserCircle = withSafeCssSize(PhUserCircle);
export const ChatCircle = withSafeCssSize(PhChatCircle);
export const Sliders = withSafeCssSize(PhSliders);
export const Moon = withSafeCssSize(PhMoon);
export const Sun = withSafeCssSize(PhSun);
export const Fire = withSafeCssSize(PhFire);
export const Pencil = withSafeCssSize(PhPencil);
export const Trash = withSafeCssSize(PhTrash);
export const Play = withSafeCssSize(PhPlay);
export const Plus = withSafeCssSize(PhPlus);
export const Copy = withSafeCssSize(PhCopy);
export const Check = withSafeCssSize(PhCheck);
export const Warning = withSafeCssSize(PhWarning);
export const ShieldWarning = withSafeCssSize(PhShieldWarning);
export const ArrowsClockwise = withSafeCssSize(PhArrowsClockwise);
export const Gear = withSafeCssSize(PhGear);
export const X = withSafeCssSize(PhX);
export const CaretDown = withSafeCssSize(PhCaretDown);
export const CaretUp = withSafeCssSize(PhCaretUp);
export const CaretRight = withSafeCssSize(PhCaretRight);
export const DownloadSimple = withSafeCssSize(PhDownloadSimple);
export const FileText = withSafeCssSize(PhFileText);
export const Clock = withSafeCssSize(PhClock);
export const CaretLeft = withSafeCssSize(PhCaretLeft);
export const Key = withSafeCssSize(PhKey);
export const MagnifyingGlass = withSafeCssSize(PhMagnifyingGlass);
export const ArrowSquareOut = withSafeCssSize(PhArrowSquareOut);
export const Star = withSafeCssSize(PhStar);
export const FolderSimple = withSafeCssSize(PhFolderSimple);
export const FolderSimplePlus = withSafeCssSize(PhFolderSimplePlus);
export const FolderOpen = withSafeCssSize(PhFolderOpen);
export const DotsThreeVertical = withSafeCssSize(PhDotsThreeVertical);
export const Hand = withSafeCssSize(PhHand);
export const PencilSimple = withSafeCssSize(PhPencilSimple);
export const ChatCircleDots = withSafeCssSize(PhChatCircleDots);
export const ChatsCircle = withSafeCssSize(PhChatsCircle);
export const GearSix = withSafeCssSize(PhGearSix);
export const Info = withSafeCssSize(PhInfo);
export const Keyboard = withSafeCssSize(PhKeyboard);
export const BookOpenText = withSafeCssSize(PhBookOpenText);
export const Plugs = withSafeCssSize(PhPlugs);
export const PuzzlePiece = withSafeCssSize(PhPuzzlePiece);
export const Image = withSafeCssSize(PhImage);
export const ToggleLeft = withSafeCssSize(PhToggleLeft);
export const ToggleRight = withSafeCssSize(PhToggleRight);
export const GitBranch = withSafeCssSize(PhGitBranch);
export const GitFork = withSafeCssSize(PhGitFork);
export const GitMerge = withSafeCssSize(PhGitMerge);
export const Broadcast = withSafeCssSize(PhBroadcast);
export const ChatTeardropDots = withSafeCssSize(PhChatTeardropDots);
export const Hash = withSafeCssSize(PhHash);
export const ChartLineUp = withSafeCssSize(PhChartLineUp);
export const Brain = withSafeCssSize(PhBrain);
