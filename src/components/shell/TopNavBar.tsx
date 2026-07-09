import { useState, useEffect } from "react";

export default function TopNavBar() {
  const [connected, setConnected] = useState(false);
  const [server, setServer] = useState("localhost:25565");

  useEffect(() => {
    const interval = setInterval(() => {
      // TODO: Get real status from Tauri backend
    }, 5000);
    return () => clearInterval(interval);
  }, []);

  return (
    <header className="top-nav">
      <div className="font-mono" style={{ fontSize: "20px", fontWeight: 700 }}>
        <span className="pixel-text-shadow" style={{ color: "var(--mc-green)" }}>
          BotControl v1.0
        </span>
      </div>

      <div style={{ flex: 1 }} />

      <div style={{ display: "flex", alignItems: "center", gap: "16px" }}>
        <div className="mc-bevel-out" style={{ padding: "4px 12px", display: "flex", alignItems: "center", gap: "8px" }}>
          <div className={`status-dot ${connected ? "" : "disconnected"}`} />
          <span className="font-mono" style={{ fontSize: "14px", color: connected ? "var(--mc-green)" : "var(--mc-red)" }}>
            {connected ? "Online" : "Offline"}
          </span>
        </div>

        <div className="mc-bevel-in" style={{ padding: "4px 12px" }}>
          <span className="font-mono" style={{ fontSize: "14px", color: "var(--mc-gray)" }}>
            {server}
          </span>
        </div>

        <button className="mc-button" style={{ padding: "4px 8px" }}>
          <span className="material-symbols-outlined">settings</span>
        </button>

        <button className="mc-button" style={{ padding: "4px 8px" }}>
          <span className="material-symbols-outlined">power_settings_new</span>
        </button>
      </div>
    </header>
  );
}
