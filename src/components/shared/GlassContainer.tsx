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
          backdropFilter: "blur(40px) saturate(180%)",
          WebkitBackdropFilter: "blur(40px) saturate(180%)",
          border: "1px solid rgba(255, 255, 255, 0.08)",
          borderRadius: "14px",
        }}
      >
        {children}
      </div>
    );
  }
);
