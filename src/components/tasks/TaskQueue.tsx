import { useState, useEffect } from "react";

interface Task {
  id: string;
  name: string;
  type: string;
  progress: number;
  status: "running" | "completed" | "failed";
}

export default function TaskQueue() {
  const [tasks, setTasks] = useState<Task[]>([
    { id: "1", name: "Automated Mining", type: "Mining", progress: 82, status: "running" },
    { id: "2", name: "Wheat Harvest", type: "Farming", progress: 45, status: "running" },
    { id: "3", name: "Server Defenses", type: "Combat", progress: 12, status: "failed" },
  ]);

  useEffect(() => {
    const interval = setInterval(() => {
      setTasks(prev => prev.map(t => ({
        ...t,
        progress: t.status === "running" ? Math.min(t.progress + 0.1, 100) : t.progress
      })));
    }, 2000);
    return () => clearInterval(interval);
  }, []);

  return (
    <div>
      <h1 className="font-mono" style={{ fontSize: "24px", fontWeight: 700, color: "var(--primary-fixed)", marginBottom: "24px" }}>
        ACTIVE TASK QUEUE
      </h1>

      <p style={{ color: "var(--mc-gray)", marginBottom: "16px" }}>Manage bot operations and automation tasks</p>

      <button className="mc-button mc-button-primary" style={{ marginBottom: "16px" }}>
        <span className="material-symbols-outlined" style={{ marginRight: "8px" }}>add</span>
        ADD TASK
      </button>

      {/* Stats */}
      <div style={{ display: "grid", gridTemplateColumns: "repeat(4, 1fr)", gap: "12px", marginBottom: "16px" }}>
        {[
          { label: "TASKS_TOTAL", value: "12", color: "var(--on-surface)" },
          { label: "COMPLETED", value: "48", color: "var(--mc-green)" },
          { label: "FAILED", value: "03", color: "var(--mc-red)" },
          { label: "AVG_LATENCY", value: "12ms", color: "var(--on-surface)" },
        ].map((stat, i) => (
          <div key={i} className="mc-bevel-in" style={{ padding: "12px", borderLeft: `4px solid ${stat.color}` }}>
            <div className="font-pixel" style={{ fontSize: "12px", color: "var(--mc-gray)" }}>{stat.label}</div>
            <div className="font-pixel" style={{ fontSize: "24px", color: stat.color }}>{stat.value}</div>
          </div>
        ))}
      </div>

      {/* Task Cards */}
      <div style={{ display: "grid", gridTemplateColumns: "repeat(3, 1fr)", gap: "12px", marginBottom: "16px" }}>
        {tasks.map((task) => (
          <div key={task.id} className="mc-slot" style={{ padding: "16px" }}>
            <div style={{ display: "flex", alignItems: "center", gap: "8px", marginBottom: "8px" }}>
              <span className="material-symbols-outlined" style={{ color: task.status === "failed" ? "var(--mc-red)" : "var(--mc-green)" }}>
                {task.type === "Mining" ? "terrain" : task.type === "Farming" ? "agriculture" : "warning"}
              </span>
              <span className="font-mono" style={{ fontWeight: 700 }}>{task.name}</span>
            </div>

            <div className="font-pixel" style={{ fontSize: "12px", color: "var(--mc-gray)", marginBottom: "8px" }}>
              Job #{task.id.padStart(4, "0")}
            </div>

            <div className="xp-bar" style={{ marginBottom: "4px" }}>
              <div
                className="xp-bar-fill"
                style={{
                  width: `${task.progress}%`,
                  background: task.status === "failed" ? "var(--mc-red)" : task.progress > 60 ? "var(--mc-green)" : "var(--mc-yellow)"
                }}
              />
            </div>

            <div className="font-pixel" style={{ fontSize: "12px", color: task.status === "failed" ? "var(--mc-red)" : "var(--mc-green)" }}>
              {task.progress.toFixed(1)}%
            </div>
          </div>
        ))}

        {/* Empty slots */}
        {[...Array(3)].map((_, i) => (
          <div key={`empty-${i}`} className="mc-slot" style={{ padding: "16px", display: "flex", alignItems: "center", justifyContent: "center", borderStyle: "dashed" }}>
            <span className="material-symbols-outlined" style={{ color: "var(--mc-gray)", fontSize: "32px" }}>add</span>
          </div>
        ))}
      </div>

      {/* Log Terminal */}
      <div className="mc-bevel-in" style={{ padding: "16px", backgroundColor: "#000" }}>
        <div className="font-pixel" style={{ fontSize: "14px", color: "var(--mc-green)", marginBottom: "8px" }}>
          TASK_LOG
        </div>
        <div style={{ fontFamily: "monospace", fontSize: "12px" }}>
          <div><span style={{ color: "var(--mc-gray)" }}>[14:02]</span> <span style={{ color: "var(--mc-green)" }}>System initialized</span></div>
          <div><span style={{ color: "var(--mc-gray)" }}>[14:03]</span> <span style={{ color: "var(--on-surface)" }}>Mining task started</span></div>
          <div><span style={{ color: "var(--mc-gray)" }}>[14:05]</span> <span style={{ color: "var(--mc-yellow)" }}>Warning: Low durability</span></div>
          <div><span style={{ color: "var(--mc-gray)" }}>[14:06]</span> <span style={{ color: "var(--mc-red)" }}>Defense task failed</span></div>
        </div>
        <div className="font-mono" style={{ color: "var(--mc-green)", marginTop: "8px" }}>
          &gt;_<span style={{ animation: "blink 1s infinite" }}>|</span>
        </div>
      </div>
    </div>
  );
}
