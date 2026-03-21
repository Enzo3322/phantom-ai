export function LoadingSpinner() {
  return (
    <div className="loading-bubbles">
      <span className="bubble" />
      <span className="bubble" />
      <span className="bubble" />
      <style>{`
        .loading-bubbles {
          display: flex;
          align-items: center;
          justify-content: center;
          gap: 6px;
          padding: 16px 0;
        }
        .bubble {
          width: 8px;
          height: 8px;
          border-radius: 50%;
          background: var(--text-accent);
          opacity: 0.3;
          animation: pulse 1.2s ease-in-out infinite;
        }
        .bubble:nth-child(2) {
          animation-delay: 0.15s;
        }
        .bubble:nth-child(3) {
          animation-delay: 0.3s;
        }
        @keyframes pulse {
          0%, 60%, 100% { opacity: 0.3; transform: scale(1); }
          30% { opacity: 1; transform: scale(1.3); }
        }
      `}</style>
    </div>
  );
}
