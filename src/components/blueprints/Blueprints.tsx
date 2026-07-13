import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Blueprint {
  name: string;
  width: number;
  height: number;
  length: number;
  block_count: number;
}

export default function Blueprints() {
  const [importUrl, setImportUrl] = useState("");
  const [importName, setImportName] = useState("");
  const [blueprints, setBlueprints] = useState<Blueprint[]>([]);
  const [loading, setLoading] = useState(false);
  const [message, setMessage] = useState("");

  const loadBlueprints = async () => {
    try {
      const list = await invoke<string[]>("list_blueprints");
      const bps: Blueprint[] = list.map((name) => ({
        name,
        width: 0,
        height: 0,
        length: 0,
        block_count: 0,
      }));
      setBlueprints(bps);
    } catch (e) {
      console.error("Failed to load blueprints:", e);
    }
  };

  const handleImport = async () => {
    if (!importUrl.trim()) return;
    setLoading(true);
    setMessage("Importing...");
    try {
      const result = await invoke<string>("import_blueprint", {
        url: importUrl,
        name: importName || undefined,
      });
      setMessage(result);
      setImportUrl("");
      setImportName("");
      loadBlueprints();
    } catch (e) {
      setMessage(`Import failed: ${e}`);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="blueprints-page">
      <h1 className="page-title">Blueprints</h1>

      <div className="import-section">
        <h2>Import Blueprint</h2>
        <div className="import-form">
          <input
            type="text"
            placeholder="GrabCraft URL"
            value={importUrl}
            onChange={(e) => setImportUrl(e.target.value)}
            className="input-field"
          />
          <input
            type="text"
            placeholder="Name (optional)"
            value={importName}
            onChange={(e) => setImportName(e.target.value)}
            className="input-field"
          />
          <button
            onClick={handleImport}
            disabled={loading || !importUrl.trim()}
            className="btn-primary"
          >
            {loading ? "Importing..." : "Import"}
          </button>
        </div>
        {message && <div className="message">{message}</div>}
      </div>

      <div className="blueprints-list">
        <h2>Imported Blueprints</h2>
        <button onClick={loadBlueprints} className="btn-secondary">
          Refresh
        </button>
        {blueprints.length === 0 ? (
          <p className="empty-state">No blueprints imported yet.</p>
        ) : (
          <div className="blueprints-grid">
            {blueprints.map((bp, index) => (
              <div key={bp.name} className="blueprint-card">
                <div className="blueprint-index">{index + 1}</div>
                <div className="blueprint-info">
                  <h3>{bp.name}</h3>
                  {bp.width > 0 && (
                    <p>
                      {bp.width}x{bp.height}x{bp.length}
                    </p>
                  )}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
