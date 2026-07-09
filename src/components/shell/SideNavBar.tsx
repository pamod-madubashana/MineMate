import { useLocation, useNavigate } from "react-router-dom";
import { useBot, useConfig } from "../../hooks/useTauri";

const navItems = [
  { path: "/", label: "Dashboard", icon: "dashboard" },
  { path: "/chat", label: "Logs", icon: "list_alt" },
  { path: "/config", label: "Config", icon: "settings" },
  { path: "/tasks", label: "Tasks", icon: "assignment" },
];

export default function SideNavBar() {
  const location = useLocation();
  const navigate = useNavigate();
  const { startBot, stopBot } = useBot();
  const { getConfig } = useConfig();

  const handleStartBot = async () => {
    try {
      const config = await getConfig();
      const server = `${config.server.address}:${config.server.port}`;
      await startBot(server, config.bot.username);
      console.log("Bot started on", server);
    } catch (e) {
      console.error("Failed to start bot:", e);
    }
  };

  const handleStopBot = async () => {
    try {
      await stopBot();
      console.log("Bot stopped");
    } catch (e) {
      console.error("Failed to stop bot:", e);
    }
  };

  return (
    <aside className="sidebar">
      <div style={{ marginBottom: "24px" }}>
        <div className="font-mono" style={{ fontSize: "16px", fontWeight: 700, color: "var(--primary-fixed)" }}>
          Admin GUI
        </div>
        <div className="font-mono" style={{ fontSize: "12px", color: "var(--mc-gray)" }}>
          Voxel Engine v2
        </div>
      </div>

      <nav style={{ flex: 1 }}>
        {navItems.map((item) => (
          <button
            key={item.path}
            className={`nav-item ${location.pathname === item.path ? "active" : ""}`}
            onClick={() => navigate(item.path)}
          >
            <span className="material-symbols-outlined">{item.icon}</span>
            <span>{item.label}</span>
          </button>
        ))}
      </nav>

      <div style={{ marginTop: "auto" }}>
        <button
          className="mc-button mc-button-primary"
          style={{ width: "100%", marginBottom: "8px" }}
          onClick={handleStartBot}
        >
          <span className="material-symbols-outlined" style={{ marginRight: "8px" }}>play_arrow</span>
          Start Bot
        </button>
        <button
          className="mc-button"
          style={{ width: "100%", marginBottom: "8px" }}
          onClick={handleStopBot}
        >
          <span className="material-symbols-outlined" style={{ marginRight: "8px" }}>stop</span>
          Stop Bot
        </button>

        <div style={{ textAlign: "center", padding: "8px" }}>
          <span className="material-symbols-outlined" style={{ color: "var(--mc-gray)" }}>help</span>
          <span className="font-mono" style={{ fontSize: "12px", color: "var(--mc-gray)", marginLeft: "4px" }}>
            Support
          </span>
        </div>
      </div>
    </aside>
  );
}
