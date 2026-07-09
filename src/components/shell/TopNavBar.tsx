import { useState, useEffect, useCallback } from "react";
import { useBot, useConfig } from "../../hooks/useTauri";

export default function TopNavBar() {
  const [connected, setConnected] = useState(false);
  const [server, setServer] = useState("localhost:25565");
  const { startBot, stopBot, getConnectionStatus } = useBot();
  const { getConfig } = useConfig();

  const loadConfig = useCallback(async () => {
    try {
      const config = await getConfig();
      setServer(`${config.server.address}:${config.server.port}`);
    } catch (e) {
      console.error("Failed to load config:", e);
    }
  }, [getConfig]);

  const checkStatus = useCallback(async () => {
    try {
      const status = await getConnectionStatus();
      setConnected(status);
    } catch (e) {
      console.error("Failed to get status:", e);
    }
  }, [getConnectionStatus]);

  useEffect(() => {
    loadConfig();
    checkStatus();
    const interval = setInterval(checkStatus, 5000);

    // Listen for config changes
    const handleConfigSaved = () => loadConfig();
    window.addEventListener("config-saved", handleConfigSaved);

    return () => {
      clearInterval(interval);
      window.removeEventListener("config-saved", handleConfigSaved);
    };
  }, [loadConfig, checkStatus]);

  const handlePowerToggle = async () => {
    if (connected) {
      try {
        await stopBot();
        setConnected(false);
      } catch (e) {
        console.error("Failed to stop bot:", e);
      }
    } else {
      try {
        const config = await getConfig();
        const serverAddr = `${config.server.address}:${config.server.port}`;
        await startBot(serverAddr, config.bot.username);
        setConnected(true);
      } catch (e) {
        console.error("Failed to start bot:", e);
      }
    }
  };

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

        <button
          className="mc-button"
          style={{ padding: "4px 8px" }}
          onClick={handlePowerToggle}
        >
          <span className="material-symbols-outlined" style={{ color: connected ? "var(--mc-red)" : "var(--mc-green)" }}>
            power_settings_new
          </span>
        </button>
      </div>
    </header>
  );
}
