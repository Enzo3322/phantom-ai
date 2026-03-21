export function LoadingSpinner() {
  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        gap: "16px",
        padding: "48px 24px",
      }}
    >
      <div
        style={{
          width: "32px",
          height: "32px",
          border: "2px solid rgba(255, 255, 255, 0.1)",
          borderTopColor: "var(--text-accent)",
          borderRadius: "50%",
          animation: "spin 0.8s linear infinite",
        }}
      />
      <span style={{ color: "var(--text-secondary)", fontSize: "13px" }}>
        Analyzing screenshot...
      </span>
      <style>{`
        @keyframes spin {
          to { transform: rotate(360deg); }
        }
      `}</style>
    </div>
  );
}
