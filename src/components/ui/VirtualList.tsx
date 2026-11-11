import { useLayoutEffect, useRef, useState, type ReactNode } from "react";

interface VirtualListProps<T> {
  items: T[];
  itemHeight: number;
  render: (item: T, index: number) => ReactNode;
  /** Nombre d'items rendus hors du viewport pour lisser le scroll (default: 6) */
  overscan?: number;
  /** Seuil en dessous duquel on desactive la virtualisation (default: 80) */
  threshold?: number;
  className?: string;
  /** Hauteur max en px ou CSS string (default: "calc(100vh - 220px)") */
  maxHeight?: string | number;
  getKey?: (item: T, index: number) => string | number;
  ariaLabel?: string;
}

/**
 * Liste virtualisee avec hauteur d'item fixe.
 *
 * Sous `threshold` items, rend tout normalement (gain de virtualization nul).
 * Au-dessus, rend uniquement les items visibles + overscan.
 */
export function VirtualList<T>({
  items,
  itemHeight,
  render,
  overscan = 6,
  threshold = 80,
  className = "",
  maxHeight = "calc(100vh - 220px)",
  getKey,
  ariaLabel,
}: VirtualListProps<T>) {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const [scrollTop, setScrollTop] = useState(0);
  const [viewportHeight, setViewportHeight] = useState(0);

  useLayoutEffect(() => {
    const el = containerRef.current;
    if (!el) return;
    setViewportHeight(el.clientHeight);
    const onResize = () => {
      if (containerRef.current) {
        setViewportHeight(containerRef.current.clientHeight);
      }
    };
    window.addEventListener("resize", onResize);
    return () => window.removeEventListener("resize", onResize);
  }, []);

  function keyFor(item: T, index: number): string | number {
    return getKey ? getKey(item, index) : index;
  }

  // Sous le threshold : rendu naif
  if (items.length < threshold) {
    return (
      <div
        ref={containerRef}
        className={`overflow-y-auto ${className}`}
        style={{ maxHeight }}
        role="list"
        aria-label={ariaLabel}
      >
        {items.map((item, i) => (
          <div key={keyFor(item, i)} role="listitem" style={{ minHeight: itemHeight }}>
            {render(item, i)}
          </div>
        ))}
      </div>
    );
  }

  const totalHeight = items.length * itemHeight;
  const startIndex = Math.max(0, Math.floor(scrollTop / itemHeight) - overscan);
  const visibleCount = Math.ceil(viewportHeight / itemHeight) + overscan * 2;
  const endIndex = Math.min(items.length, startIndex + visibleCount);
  const visible = items.slice(startIndex, endIndex);
  const offsetY = startIndex * itemHeight;

  return (
    <div
      ref={containerRef}
      className={`overflow-y-auto ${className}`}
      style={{ maxHeight }}
      onScroll={(e) => setScrollTop((e.target as HTMLDivElement).scrollTop)}
      role="list"
      aria-label={ariaLabel}
      aria-rowcount={items.length}
    >
      <div style={{ height: totalHeight, position: "relative" }}>
        <div style={{ transform: `translateY(${offsetY}px)` }}>
          {visible.map((item, i) => {
            const absoluteIndex = startIndex + i;
            return (
              <div
                key={keyFor(item, absoluteIndex)}
                role="listitem"
                aria-rowindex={absoluteIndex + 1}
                style={{ height: itemHeight }}
              >
                {render(item, absoluteIndex)}
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}

export default VirtualList;
