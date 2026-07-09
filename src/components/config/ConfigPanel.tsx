import { useState, useEffect } from "react";
import { useConfig, AppConfig } from "../../hooks/useTauri";

interface ToggleSwitch {
  label: string;
  key: keyof AppConfig["automation"];
  enabled: boolean;
}

export default function ConfigPanel() {
  const { getConfig, saveConfig } = useConfig();
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    const loadConfig = async () => {
      try {
        const c = await getConfig();
        setConfig(c);
      } catch (e) {
        console.error("Failed to load config:", e);
      }
    };
    loadConfig();
  }, []);

  const toggleSwitch = (index: number) => {
    if (!config) return;
    const newConfig = { ...config };
    const keys: (keyof AppConfig["automation"])[] = [
      "auto_sleep", "auto_eat", "auto_reconnect", "welcome_messages", "starter_kit_on_respawn"
    ];
    const key = keys[index];
    newConfig.automation[key] = !newConfig.automation[key];
    setConfig(newConfig);
  };

  const handleSave = async () => {
    if (!config) return;
    setSaving(true);
    try {
      await saveConfig(config);
      // Emit event so TopNavBar reloads config
      window.dispatchEvent(new CustomEvent("config-saved"));
    } catch (e) {
      console.error("Failed to save config:", e);
    }
    setSaving(false);
  };

  if (!config) {
    return <div style={{ color: "var(--mc-gray)" }}>Loading configuration...</div>;
  }

  const toggles = [
    { label: "Auto Sleep", key: "auto_sleep" as keyof AppConfig["automation"], enabled: config.automation.auto_sleep },
    { label: "Auto Eat", key: "auto_eat" as keyof AppConfig["automation"], enabled: config.automation.auto_eat },
    { label: "Auto Reconnect", key: "auto_reconnect" as keyof AppConfig["automation"], enabled: config.automation.auto_reconnect },
    { label: "Welcome Messages", key: "welcome_messages" as keyof AppConfig["automation"], enabled: config.automation.welcome_messages },
    { label: "Starter Kit on Respawn", key: "starter_kit_on_respawn" as keyof AppConfig["automation"], enabled: config.automation.starter_kit_on_respawn },
  ];

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
              <input
                className="mc-input"
                style={{ width: "100%" }}
                value={config.server.address}
                onChange={(e) => setConfig({ ...config, server: { ...config.server, address: e.target.value } })}
              />
            </div>
            <div>
              <label className="font-mono" style={{ fontSize: "12px", color: "var(--mc-gray)", display: "block", marginBottom: "4px" }}>Port</label>
              <input
                className="mc-input"
                style={{ width: "100%" }}
                value={config.server.port}
                onChange={(e) => setConfig({ ...config, server: { ...config.server, port: parseInt(e.target.value) || 25565 } })}
              />
            </div>
            <div>
              <label className="font-mono" style={{ fontSize: "12px", color: "var(--mc-gray)", display: "block", marginBottom: "4px" }}>Bot Username</label>
              <input
                className="mc-input"
                style={{ width: "100%" }}
                value={config.bot.username}
                onChange={(e) => setConfig({ ...config, bot: { ...config.bot, username: e.target.value } })}
              />
            </div>
            <div>
              <label className="font-mono" style={{ fontSize: "12px", color: "var(--mc-gray)", display: "block", marginBottom: "4px" }}>Permission Mode</label>
              <select
                className="mc-input"
                style={{ width: "100%" }}
                value={config.bot.permission_mode}
                onChange={(e) => setConfig({ ...config, bot: { ...config.bot, permission_mode: e.target.value } })}
              >
                <option value="Player">Player</option>
                <option value="Operator">Operator</option>
              </select>
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
              <input
                className="mc-input"
                style={{ width: "100%" }}
                type="password"
                value={config.ai.api_key}
                onChange={(e) => setConfig({ ...config, ai: { ...config.ai, api_key: e.target.value } })}
                placeholder="nvapi-..."
              />
            </div>
            <div>
              <label className="font-mono" style={{ fontSize: "12px", color: "var(--mc-gray)", display: "block", marginBottom: "4px" }}>Model</label>
              <select
                className="mc-input"
                style={{ width: "100%" }}
                value={config.ai.model}
                onChange={(e) => setConfig({ ...config, ai: { ...config.ai, model: e.target.value } })}
              >
                <option value="meta/llama-3.3-70b-instruct">Llama 3.3 70B</option>
                <option value="qwen/qwen2.5-coder-32b-instruct">Qwen 2.5 Coder 32B</option>
                <option value="deepseek-ai/deepseek-r1">DeepSeek R1</option>
              </select>
            </div>
            <div>
              <label className="font-mono" style={{ fontSize: "12px", color: "var(--mc-gray)", display: "block", marginBottom: "4px" }}>Temperature: {config.ai.temperature}</label>
              <input
                type="range"
                min="0"
                max="1"
                step="0.1"
                value={config.ai.temperature}
                onChange={(e) => setConfig({ ...config, ai: { ...config.ai, temperature: parseFloat(e.target.value) } })}
                style={{ width: "100%" }}
              />
            </div>
            <div>
              <label className="font-mono" style={{ fontSize: "12px", color: "var(--mc-gray)", display: "block", marginBottom: "4px" }}>Max Tokens</label>
              <input
                className="mc-input"
                style={{ width: "100%" }}
                type="number"
                value={config.ai.max_tokens}
                onChange={(e) => setConfig({ ...config, ai: { ...config.ai, max_tokens: parseInt(e.target.value) || 1024 } })}
              />
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
              {config.starter_kit[i] && (
                <div style={{ position: "relative" }}>
                  <span className="material-symbols-outlined" style={{ color: "var(--mc-aqua)" }}>sports_esports</span>
                  <span className="font-pixel" style={{ position: "absolute", bottom: "-4px", right: "-4px", fontSize: "10px", color: "var(--mc-yellow)" }}>
                    {config.starter_kit[i].count}
                  </span>
                </div>
              )}
            </div>
          ))}
        </div>
      </div>

      {/* Save Button */}
      <div style={{ marginTop: "16px", display: "flex", gap: "8px" }}>
        <button className="mc-button" onClick={() => window.location.reload()}>
          RESET
        </button>
        <button className="mc-button mc-button-primary" onClick={handleSave} disabled={saving}>
          {saving ? "SAVING..." : "SAVE CONFIG"}
        </button>
      </div>
    </div>
  );
}
