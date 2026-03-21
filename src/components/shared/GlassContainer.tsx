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
        overflow: "hidden",
      }}
    >
      {children}
    </div>
  );
}
