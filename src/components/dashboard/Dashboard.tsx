export default function Dashboard() {
  return (
    <div>
      <h1 className="font-mono" style={{ fontSize: "24px", fontWeight: 700, color: "var(--primary-fixed)", marginBottom: "24px" }}>
        Dashboard
      </h1>

      {/* HUD Section */}
      <div className="mc-bevel-in" style={{ padding: "16px", marginBottom: "16px" }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
          <div>
            <div style={{ display: "flex", gap: "4px", marginBottom: "8px" }}>
              {[...Array(10)].map((_, i) => (
                <span key={i} className="material-symbols-outlined" style={{ fontSize: "20px", color: i < 8 ? "var(--mc-red)" : "var(--mc-gray)", fontVariationSettings: i < 8 ? "'FILL' 1" : "'FILL' 0" }}>
                  favorite
                </span>
              ))}
            </div>
            <div className="xp-bar" style={{ width: "200px" }}>
              <div className="xp-bar-fill" style={{ width: "70%" }} />
            </div>
          </div>

          <div className="font-pixel" style={{ fontSize: "32px", color: "var(--mc-green)" }}>
            Level 42
          </div>

          <div>
            <div style={{ display: "flex", gap: "4px", marginBottom: "8px" }}>
              {[...Array(6)].map((_, i) => (
                <span key={i} className="material-symbols-outlined" style={{ fontSize: "20px", color: i < 5 ? "#AA5500" : "var(--mc-gray)", fontVariationSettings: i < 5 ? "'FILL' 1" : "'FILL' 0" }}>
                  nutrition
                </span>
              ))}
            </div>
          </div>
        </div>
      </div>

      {/* Bento Grid */}
      <div style={{ display: "grid", gridTemplateColumns: "2fr 1fr", gap: "16px" }}>
        {/* Player List */}
        <div className="mc-bevel-in" style={{ padding: "16px" }}>
          <div className="font-pixel" style={{ fontSize: "18px", color: "var(--mc-green)", marginBottom: "12px" }}>
            PLAYER LIST [0/50]
          </div>
          <div style={{ color: "var(--mc-gray)" }}>
            No players connected
          </div>
        </div>

        {/* Quick Actions */}
        <div className="mc-bevel-in" style={{ padding: "16px" }}>
          <div className="font-pixel" style={{ fontSize: "18px", color: "var(--mc-green)", marginBottom: "12px" }}>
            QUICK ACTIONS
          </div>
          <div className="inventory-grid" style={{ gridTemplateColumns: "repeat(3, 64px)" }}>
            {["bolt", "shield", "cleaning_services", "restart_alt", "save", "schedule", "history", "warning", "delete_forever"].map((icon, i) => (
              <div key={i} className={`inventory-slot ${i === 0 ? "mc-slot-active" : ""}`}>
                <span className="material-symbols-outlined">{icon}</span>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* Event Log */}
      <div className="mc-bevel-in" style={{ padding: "16px", marginTop: "16px" }}>
        <div style={{ display: "flex", alignItems: "center", gap: "8px", marginBottom: "12px" }}>
          <span className="font-pixel" style={{ fontSize: "18px", color: "var(--mc-green)" }}>
            EVENT_LOG_V2
          </span>
          <span className="font-pixel" style={{ fontSize: "12px", color: "var(--mc-green)", animation: "pulse 2s infinite" }}>
            LIVE STREAMING
          </span>
        </div>
        <div style={{ fontFamily: "monospace", fontSize: "12px", color: "var(--mc-gray)" }}>
          <div>[SYSTEM] Bot initialized</div>
          <div>[SYSTEM] Waiting for connection...</div>
        </div>
      </div>
    </div>
  );
}
