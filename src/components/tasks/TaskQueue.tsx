import { useState, useEffect } from "react";

interface Task {
  id: string;
  name: string;
  type: string;
  progress: number;
  status: "running" | "completed" | "failed" | "queued";
}

export default function TaskQueue() {
  const [tasks, setTasks] = useState<Task[]>([
    { id: "1", name: "Automated Mining", type: "Mining", progress: 82, status: "running" },
    { id: "2", name: "Wheat Harvest", type: "Farming", progress: 45, status: "running" },
    { id: "3", name: "Server Defenses", type: "Combat", progress: 12, status: "failed" },
  ]);

  useEffect(() => {
    const interval = setInterval(() => {
      setTasks((prev) =>
        prev.map((t) => ({
          ...t,
          progress: t.status === "running" ? Math.min(t.progress + 0.1, 100) : t.progress,
        }))
      );
    }, 2000);
    return () => clearInterval(interval);
  }, []);

  const addTask = () => {
    const newTask: Task = {
      id: String(tasks.length + 1),
      name: `New Task ${tasks.length + 1}`,
      type: "Mining",
      progress: 0,
      status: "queued",
    };
    setTasks([...tasks, newTask]);
  };

  const removeTask = (id: string) => {
    setTasks(tasks.filter((t) => t.id !== id));
  };

  const totalTasks = tasks.length;
  const completedTasks = tasks.filter((t) => t.status === "completed").length;
  const failedTasks = tasks.filter((t) => t.status === "failed").length;
  const avgLatency = 12;

  return (
    <div>
      <h1 className="font-mono" style={{ fontSize: "24px", fontWeight: 700, color: "var(--primary-fixed)", marginBottom: "24px" }}>
        ACTIVE TASK QUEUE
      </h1>

      <p style={{ color: "var(--mc-gray)", marginBottom: "16px" }}>Manage bot operations and automation tasks</p>

      <button className="mc-button mc-button-primary" style={{ marginBottom: "16px" }} onClick={addTask}>
        <span className="material-symbols-outlined" style={{ marginRight: "8px" }}>add</span>
        ADD TASK
      </button>

      {/* Stats */}
      <div style={{ display: "grid", gridTemplateColumns: "repeat(4, 1fr)", gap: "12px", marginBottom: "16px" }}>
        {[
          { label: "TASKS_TOTAL", value: String(totalTasks), color: "var(--on-surface)" },
          { label: "COMPLETED", value: String(completedTasks), color: "var(--mc-green)" },
          { label: "FAILED", value: String(failedTasks), color: "var(--mc-red)" },
          { label: "AVG_LATENCY", value: `${avgLatency}ms`, color: "var(--on-surface)" },
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
          <div key={task.id} className="mc-slot" style={{ padding: "16px", position: "relative" }}>
            <button
              className="mc-button"
              style={{ position: "absolute", top: "8px", right: "8px", padding: "2px 6px", fontSize: "10px" }}
              onClick={() => removeTask(task.id)}
            >
              ×
            </button>

            <div style={{ display: "flex", alignItems: "center", gap: "8px", marginBottom: "8px" }}>
              <span className="material-symbols-outlined" style={{ color: task.status === "failed" ? "var(--mc-red)" : task.status === "completed" ? "var(--mc-green)" : "var(--mc-yellow)" }}>
                {task.type === "Mining" ? "terrain" : task.type === "Farming" ? "agriculture" : task.type === "Combat" ? "warning" : "pending"}
              </span>
              <span className="font-mono" style={{ fontWeight: 700 }}>{task.name}</span>
            </div>

            <div className="font-pixel" style={{ fontSize: "12px", color: "var(--mc-gray)", marginBottom: "8px" }}>
              Job #{task.id.padStart(4, "0")} | {task.type} | {task.status.toUpperCase()}
            </div>

            <div className="xp-bar" style={{ marginBottom: "4px" }}>
              <div
                className="xp-bar-fill"
                style={{
                  width: `${task.progress}%`,
                  background: task.status === "failed" ? "var(--mc-red)" : task.status === "completed" ? "var(--mc-green)" : task.progress > 60 ? "var(--mc-green)" : "var(--mc-yellow)",
                }}
              />
            </div>

            <div className="font-pixel" style={{ fontSize: "12px", color: task.status === "failed" ? "var(--mc-red)" : "var(--mc-green)" }}>
              {task.progress.toFixed(1)}%
            </div>
          </div>
        ))}

        {/* Empty slots */}
        {[...Array(Math.max(0, 3 - tasks.length))].map((_, i) => (
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
          <div><span style={{ color: "var(--mc-gray)" }}>[{new Date().toLocaleTimeString()}]</span> <span style={{ color: "var(--mc-green)" }}>Task queue initialized</span></div>
          <div><span style={{ color: "var(--mc-gray)" }}>[{new Date().toLocaleTimeString()}]</span> <span style={{ color: "var(--on-surface)" }}>{totalTasks} tasks loaded</span></div>
          {failedTasks > 0 && (
            <div><span style={{ color: "var(--mc-gray)" }}>[{new Date().toLocaleTimeString()}]</span> <span style={{ color: "var(--mc-red)" }}>Warning: {failedTasks} failed tasks</span></div>
          )}
        </div>
        <div className="font-mono" style={{ color: "var(--mc-green)", marginTop: "8px" }}>
          &gt;_<span style={{ animation: "blink 1s infinite" }}>|</span>
        </div>
      </div>
    </div>
  );
}
