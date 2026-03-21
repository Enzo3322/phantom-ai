import { forwardRef, ReactNode } from "react";

interface GlassContainerProps {
  children: ReactNode;
}

export const GlassContainer = forwardRef<HTMLDivElement, GlassContainerProps>(
  ({ children }, ref) => {
    return (
      <div
        ref={ref}
        style={{
          width: "100%",
          display: "flex",
          flexDirection: "column",
          overflow: "hidden",
          background: "var(--bg-panel)",
          borderRadius: "14px",
        }}
      >
        {children}
      </div>
    );
  }
);
