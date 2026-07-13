import { useLayoutEffect, useRef, type ComponentProps, type Ref } from "react";
import { OverlayScrollbars, type OverflowBehavior } from "overlayscrollbars";

import { cn } from "@/lib/utils";

interface OverlayScrollAreaProps extends Omit<ComponentProps<"div">, "ref"> {
  overflowX?: OverflowBehavior;
  overflowY?: OverflowBehavior;
  viewportClassName?: string;
  viewportRef?: Ref<HTMLDivElement>;
}

function OverlayScrollArea({
  children,
  className,
  overflowX = "scroll",
  overflowY = "scroll",
  viewportClassName,
  viewportRef,
  ...props
}: OverlayScrollAreaProps) {
  const host = useRef<HTMLDivElement>(null);
  const viewport = useRef<HTMLDivElement>(null);

  useLayoutEffect(() => {
    if (!host.current || !viewport.current) return;

    const viewportElement = viewport.current;
    const instance = OverlayScrollbars(
      {
        target: host.current,
        elements: { viewport: viewportElement },
      },
      {
        overflow: { x: overflowX, y: overflowY },
        scrollbars: {
          autoHide: "leave",
          autoHideDelay: 250,
          theme: "os-theme-kickhatsnare",
        },
      },
    );
    setRef(viewportRef, viewportElement);

    return () => {
      setRef(viewportRef, null);
      instance.destroy();
    };
  }, [overflowX, overflowY, viewportRef]);

  return (
    <div
      ref={host}
      data-overlayscrollbars-initialize=""
      className={cn("min-h-0 min-w-0", className)}
      {...props}
    >
      <div ref={viewport} className={cn("h-full w-full", viewportClassName)}>
        {children}
      </div>
    </div>
  );
}

function setRef<T>(ref: Ref<T> | undefined, value: T | null) {
  if (typeof ref === "function") ref(value);
  else if (ref) ref.current = value;
}

export { OverlayScrollArea };
