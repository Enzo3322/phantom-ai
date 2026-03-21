import { ReactNode } from "react";

interface GlassContainerProps {
  children: ReactNode;
}

export function GlassContainer({ children }: GlassContainerProps) {
  return (
    <div
      style={{
        width: "100%",
        height: "100%",
        display: "flex",
        flexDirection: "column",
        borderRadius: "var(--radius-xl)",
        border: "1px solid var(--border-glass)",
        overflow: "hidden",
      }}
    >
      {children}
    </div>
  );
}
