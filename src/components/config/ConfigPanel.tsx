import { useState } from "react";

interface ToggleSwitch {
  label: string;
  enabled: boolean;
}

export default function ConfigPanel() {
  const [toggles, setToggles] = useState<ToggleSwitch[]>([
    { label: "Auto Sleep", enabled: true },
    { label: "Auto Eat", enabled: true },
    { label: "Auto Reconnect", enabled: true },
    { label: "Welcome Messages", enabled: true },
    { label: "Starter Kit on Respawn", enabled: true },
  ]);

  const [apiKey, setApiKey] = useState("");
  const [server, setServer] = useState("localhost");
  const [port, setPort] = useState("25565");
  const [username, setUsername] = useState("MineMate");

  const toggleSwitch = (index: number) => {
    const newToggles = [...toggles];
    newToggles[index].enabled = !newToggles[index].enabled;
    setToggles(newToggles);
  };

  return (
    <div>
      <h1 className="font-mono" style={{ fontSize: "24px", fontWeight: 700, color: "var(--primary-fixed)", marginBottom: "24px" }}>
        Configuration Panel
      </h1>

      {/* XP Bar */}
      <div className="xp-bar" style={{ marginBottom: "24px" }}>
        <div className="xp-bar-fill" style={{ width: "75%" }} />
      </div>

      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "16px" }}>
        {/* General Settings */}
        <div className="mc-card">
          <div style={{ display: "flex", alignItems: "center", gap: "8px", marginBottom: "16px", borderBottom: "2px solid var(--outline-variant)", paddingBottom: "8px" }}>
            <span className="material-symbols-outlined">tune</span>
            <span className="font-mono" style={{ fontWeight: 700 }}>GENERAL SETTINGS</span>
          </div>

          <div style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
            <div>
              <label className="font-mono" style={{ fontSize: "12px", color: "var(--mc-gray)", display: "block", marginBottom: "4px" }}>Server Address</label>
              <input className="mc-input" style={{ width: "100%" }} value={server} onChange={(e) => setServer(e.target.value)} />
            </div>
            <div>
              <label className="font-mono" style={{ fontSize: "12px", color: "var(--mc-gray)", display: "block", marginBottom: "4px" }}>Port</label>
              <input className="mc-input" style={{ width: "100%" }} value={port} onChange={(e) => setPort(e.target.value)} />
            </div>
            <div>
              <label className="font-mono" style={{ fontSize: "12px", color: "var(--mc-gray)", display: "block", marginBottom: "4px" }}>Bot Username</label>
              <input className="mc-input" style={{ width: "100%" }} value={username} onChange={(e) => setUsername(e.target.value)} />
            </div>
          </div>
        </div>

        {/* AI Settings */}
        <div className="mc-card">
          <div style={{ display: "flex", alignItems: "center", gap: "8px", marginBottom: "16px", borderBottom: "2px solid var(--outline-variant)", paddingBottom: "8px" }}>
            <span className="material-symbols-outlined">smart_toy</span>
            <span className="font-mono" style={{ fontWeight: 700 }}>AI CONFIGURATION</span>
          </div>

          <div style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
            <div>
              <label className="font-mono" style={{ fontSize: "12px", color: "var(--mc-gray)", display: "block", marginBottom: "4px" }}>NVIDIA NIM API Key</label>
              <input className="mc-input" style={{ width: "100%" }} type="password" value={apiKey} onChange={(e) => setApiKey(e.target.value)} placeholder="nvapi-..." />
            </div>
            <div>
              <label className="font-mono" style={{ fontSize: "12px", color: "var(--mc-gray)", display: "block", marginBottom: "4px" }}>Model</label>
              <select className="mc-input" style={{ width: "100%" }}>
                <option>meta/llama-3.3-70b-instruct</option>
                <option>qwen/qwen2.5-coder-32b-instruct</option>
                <option>deepseek-ai/deepseek-r1</option>
              </select>
            </div>
          </div>
        </div>
      </div>

      {/* Automations */}
      <div className="mc-card" style={{ marginTop: "16px" }}>
        <div style={{ display: "flex", alignItems: "center", gap: "8px", marginBottom: "16px", borderBottom: "2px solid var(--outline-variant)", paddingBottom: "8px" }}>
          <span className="material-symbols-outlined">automation</span>
          <span className="font-mono" style={{ fontWeight: 700 }}>AUTOMATIONS</span>
        </div>

        <div style={{ display: "grid", gridTemplateColumns: "repeat(3, 1fr)", gap: "12px" }}>
          {toggles.map((toggle, i) => (
            <button
              key={i}
              className="mc-button"
              style={{ textAlign: "left", display: "flex", justifyContent: "space-between", alignItems: "center" }}
              onClick={() => toggleSwitch(i)}
            >
              <span>{toggle.label}</span>
              <span style={{ color: toggle.enabled ? "var(--mc-green)" : "var(--mc-red)" }}>
                {toggle.enabled ? "ON" : "OFF"}
              </span>
            </button>
          ))}
        </div>
      </div>

      {/* Starter Kit */}
      <div className="mc-card" style={{ marginTop: "16px" }}>
        <div style={{ display: "flex", alignItems: "center", gap: "8px", marginBottom: "16px", borderBottom: "2px solid var(--outline-variant)", paddingBottom: "8px" }}>
          <span className="material-symbols-outlined">inventory_2</span>
          <span className="font-mono" style={{ fontWeight: 700 }}>STARTER KIT</span>
        </div>

        <div className="inventory-grid">
          {[...Array(36)].map((_, i) => (
            <div key={i} className="inventory-slot">
              {i === 0 && <span className="material-symbols-outlined" style={{ color: "var(--mc-aqua)" }}>sports_esports</span>}
              {i === 1 && <span className="material-symbols-outlined" style={{ color: "var(--mc-aqua)" }}>agriculture</span>}
              {i === 8 && (
                <div style={{ position: "relative" }}>
                  <span className="material-symbols-outlined" style={{ color: "var(--mc-aqua)" }}>nutrition</span>
                  <span className="font-pixel" style={{ position: "absolute", bottom: "-4px", right: "-4px", fontSize: "10px", color: "var(--mc-yellow)" }}>64</span>
                </div>
              )}
            </div>
          ))}
        </div>

        <div style={{ display: "flex", gap: "8px", marginTop: "16px" }}>
          <button className="mc-button">RESET KIT</button>
          <button className="mc-button mc-button-primary">SAVE CONFIG</button>
        </div>
      </div>
    </div>
  );
}
