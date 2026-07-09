import { useEffect, useState } from "react";
import { useBotStatusPolling, useMemory, useBotEvents, Player, HistoryEntry } from "../../hooks/useTauri";

export default function Dashboard() {
  const status = useBotStatusPolling(5000);
  const { listPlayers, getHistory } = useMemory();
  const [players, setPlayers] = useState<Player[]>([]);
  const [events, setEvents] = useState<HistoryEntry[]>([]);

  useEffect(() => {
    const loadData = async () => {
      try {
        const p = await listPlayers();
        setPlayers(p);
        const h = await getHistory(10);
        setEvents(h);
      } catch (e) {
        console.error("Failed to load data:", e);
      }
    };
    loadData();
  }, [listPlayers, getHistory]);

  useBotEvents((event) => {
    if (event.type === "PlayerJoined" || event.type === "PlayerLeft") {
      listPlayers().then(setPlayers);
    }
    if (event.type === "ChatMessage" || event.type === "SystemEvent") {
      getHistory(10).then(setEvents);
    }
  });

  const hearts = Math.floor(status.health);
  const emptyHearts = 10 - hearts;
  const hunger = Math.floor(status.food / 2);
  const emptyHunger = 6 - hunger;

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
                <span key={i} className="material-symbols-outlined" style={{ fontSize: "20px", color: i < hearts ? "var(--mc-red)" : "var(--mc-gray)", fontVariationSettings: i < hearts ? "'FILL' 1" : "'FILL' 0" }}>
                  favorite
                </span>
              ))}
            </div>
            <div className="xp-bar" style={{ width: "200px" }}>
              <div className="xp-bar-fill" style={{ width: `${(status.health / 20) * 100}%` }} />
            </div>
          </div>

          <div className="font-pixel" style={{ fontSize: "32px", color: "var(--mc-green)" }}>
            {status.connected ? "Connected" : "Disconnected"}
          </div>

          <div>
            <div style={{ display: "flex", gap: "4px", marginBottom: "8px" }}>
              {[...Array(6)].map((_, i) => (
                <span key={i} className="material-symbols-outlined" style={{ fontSize: "20px", color: i < hunger ? "#AA5500" : "var(--mc-gray)", fontVariationSettings: i < hunger ? "'FILL' 1" : "'FILL' 0" }}>
                  nutrition
                </span>
              ))}
            </div>
          </div>
        </div>

        <div style={{ display: "flex", gap: "24px", marginTop: "16px" }}>
          <div className="mc-bevel-out" style={{ padding: "8px 12px" }}>
            <span className="font-mono" style={{ fontSize: "12px", color: "var(--mc-gray)" }}>X: </span>
            <span className="font-pixel" style={{ fontSize: "16px", color: "var(--mc-green)" }}>{status.x.toFixed(1)}</span>
          </div>
          <div className="mc-bevel-out" style={{ padding: "8px 12px" }}>
            <span className="font-mono" style={{ fontSize: "12px", color: "var(--mc-gray)" }}>Y: </span>
            <span className="font-pixel" style={{ fontSize: "16px", color: "var(--mc-green)" }}>{status.y.toFixed(1)}</span>
          </div>
          <div className="mc-bevel-out" style={{ padding: "8px 12px" }}>
            <span className="font-mono" style={{ fontSize: "12px", color: "var(--mc-gray)" }}>Z: </span>
            <span className="font-pixel" style={{ fontSize: "16px", color: "var(--mc-green)" }}>{status.z.toFixed(1)}</span>
          </div>
        </div>
      </div>

      {/* Bento Grid */}
      <div style={{ display: "grid", gridTemplateColumns: "2fr 1fr", gap: "16px" }}>
        {/* Player List */}
        <div className="mc-bevel-in" style={{ padding: "16px" }}>
          <div className="font-pixel" style={{ fontSize: "18px", color: "var(--mc-green)", marginBottom: "12px" }}>
            PLAYER LIST [{players.length}/50]
          </div>
          {players.length === 0 ? (
            <div style={{ color: "var(--mc-gray)" }}>
              No players connected
            </div>
          ) : (
            <div style={{ display: "grid", gridTemplateColumns: "repeat(2, 1fr)", gap: "8px" }}>
              {players.slice(0, 10).map((player) => (
                <div key={player.id} className="mc-slot" style={{ padding: "8px", display: "flex", alignItems: "center", gap: "8px" }}>
                  <div style={{ width: "32px", height: "32px", background: "var(--mc-stone)", display: "flex", alignItems: "center", justifyContent: "center" }}>
                    <span className="material-symbols-outlined">person</span>
                  </div>
                  <div>
                    <div className="font-mono" style={{ fontSize: "12px" }}>{player.name}</div>
                    <div className="font-pixel" style={{ fontSize: "10px", color: "var(--mc-gray)" }}>
                      Last seen: {new Date(player.last_seen).toLocaleDateString()}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Quick Actions */}
        <div className="mc-bevel-in" style={{ padding: "16px" }}>
          <div className="font-pixel" style={{ fontSize: "18px", color: "var(--mc-green)", marginBottom: "12px" }}>
            QUICK ACTIONS
          </div>
          <div style={{ display: "grid", gridTemplateColumns: "repeat(3, 64px)", gap: "4px" }}>
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
          {events.length === 0 ? (
            <div>[SYSTEM] Bot initialized - waiting for events...</div>
          ) : (
            events.map((event) => (
              <div key={event.id}>
                <span style={{ color: "var(--mc-gray)" }}>[{new Date(event.timestamp).toLocaleTimeString()}]</span>{' '}
                <span style={{ color: event.event_type === "error" ? "var(--mc-red)" : "var(--mc-green)" }}>
                  [{event.event_type}]
                </span>{' '}
                {event.player && <span style={{ color: "var(--mc-aqua)" }}>{event.player}: </span>}
                {event.details}
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}
