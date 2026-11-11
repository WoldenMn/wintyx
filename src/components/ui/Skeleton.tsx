interface SkeletonProps {
  width?: string | number;
  height?: string | number;
  className?: string;
  rounded?: boolean;
}

export function Skeleton({
  width = "100%",
  height = "1rem",
  className = "",
  rounded = false,
}: SkeletonProps) {
  return (
    <span
      aria-hidden="true"
      className={`inline-block bg-bg-elevated animate-pulse ${rounded ? "rounded-full" : "rounded"} ${className}`}
      style={{ width, height }}
    />
  );
}

interface CardSkeletonProps {
  lines?: number;
  className?: string;
}

export function CardSkeleton({ lines = 2, className = "" }: CardSkeletonProps) {
  return (
    <div
      className={`bg-bg-card border border-border rounded-lg p-5 theme-card ${className}`}
      aria-hidden="true"
    >
      <Skeleton width="30%" height="0.75rem" className="mb-3" />
      {Array.from({ length: lines }).map((_, i) => (
        <Skeleton
          key={i}
          width={i === lines - 1 ? "60%" : "100%"}
          height="0.875rem"
          className="mb-2 last:mb-0"
        />
      ))}
    </div>
  );
}

interface ListSkeletonProps {
  count?: number;
  itemHeight?: string;
  className?: string;
}

export function ListSkeleton({
  count = 5,
  itemHeight = "5rem",
  className = "",
}: ListSkeletonProps) {
  return (
    <div className={`space-y-3 ${className}`} aria-busy="true" aria-live="polite">
      {Array.from({ length: count }).map((_, i) => (
        <div
          key={i}
          className="bg-bg-card border border-border rounded-lg p-5 theme-card flex items-center gap-4"
          style={{ minHeight: itemHeight }}
        >
          <Skeleton width="40%" height="1rem" />
          <Skeleton width="20%" height="0.75rem" />
          <Skeleton width="15%" height="1.5rem" className="ml-auto" />
        </div>
      ))}
    </div>
  );
}

interface PageSkeletonProps {
  kind?: "list" | "grid" | "dashboard";
  count?: number;
}

export function PageSkeleton({ kind = "list", count = 6 }: PageSkeletonProps) {
  if (kind === "dashboard") {
    return (
      <div className="p-7 h-full" aria-busy="true" aria-live="polite">
        <div className="mb-6">
          <Skeleton width="40%" height="1.5rem" className="mb-2" />
          <Skeleton width="60%" height="0.875rem" />
        </div>
        <div className="bg-bg-card border border-border rounded-lg p-6 mb-6 flex items-center gap-6">
          <Skeleton width="5rem" height="3.5rem" />
          <div className="flex-1">
            <Skeleton width="50%" height="1rem" className="mb-2" />
            <Skeleton width="90%" height="0.875rem" />
          </div>
        </div>
        <div className="grid grid-cols-5 gap-4 mb-6">
          {Array.from({ length: 5 }).map((_, i) => (
            <CardSkeleton key={i} lines={1} />
          ))}
        </div>
        <div className="grid grid-cols-2 gap-4">
          {Array.from({ length: 4 }).map((_, i) => (
            <CardSkeleton key={i} lines={2} />
          ))}
        </div>
      </div>
    );
  }
  if (kind === "grid") {
    return (
      <div className="p-7" aria-busy="true" aria-live="polite">
        <div className="grid grid-cols-3 gap-4">
          {Array.from({ length: count }).map((_, i) => (
            <CardSkeleton key={i} lines={2} />
          ))}
        </div>
      </div>
    );
  }
  return (
    <div className="p-7" aria-busy="true" aria-live="polite">
      <div className="mb-6">
        <Skeleton width="30%" height="1.5rem" className="mb-2" />
        <Skeleton width="50%" height="0.875rem" />
      </div>
      <ListSkeleton count={count} />
    </div>
  );
}
