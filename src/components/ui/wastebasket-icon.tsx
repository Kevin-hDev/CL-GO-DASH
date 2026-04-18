interface WastebasketIconProps {
  size?: number;
  className?: string;
}

export function WastebasketIcon({ size = 14, className }: WastebasketIconProps) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
      strokeLinecap="round"
      strokeLinejoin="round"
      className={className}
    >
      <path
        d="M9 3.5L8 8h2.5L9.5 3l-.5.5z"
        fill="currentColor"
        opacity="0.35"
        strokeWidth="1"
      />
      <path
        d="M14.5 2.5L15.5 8H13l1-5 .5-.5z"
        fill="currentColor"
        opacity="0.35"
        strokeWidth="1"
      />
      <ellipse cx="12" cy="8.5" rx="7.5" ry="2" />
      <path d="M4.5 8.5L7 21.5h10l2.5-13" />
      <path
        d="M6.5 12l4.5 9.5M10.5 8.5l4 13M15 8.5l2.5 10"
        strokeWidth="1"
        opacity="0.5"
      />
      <path
        d="M17.5 12l-4.5 9.5M13.5 8.5l-4 13M9 8.5L6.5 18.5"
        strokeWidth="1"
        opacity="0.5"
      />
    </svg>
  );
}
