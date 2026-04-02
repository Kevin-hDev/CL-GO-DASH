import "./skeleton.css";

interface SkeletonProps {
  width?: string;
  height?: string;
  count?: number;
}

export function Skeleton({
  width = "100%",
  height = "16px",
  count = 1,
}: SkeletonProps) {
  return (
    <div className="skeleton-container">
      {Array.from({ length: count }).map((_, i) => (
        <div
          key={i}
          className="skeleton-line"
          style={{ width, height }}
        />
      ))}
    </div>
  );
}
