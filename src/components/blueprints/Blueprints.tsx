import { useState, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Blueprint {
  name: string;
  width?: number;
  height?: number;
  length?: number;
}

export default function Blueprints() {
  const [importUrl, setImportUrl] = useState("");
  const [importName, setImportName] = useState("");
  const [blueprints, setBlueprints] = useState<Blueprint[]>([]);
  const [loading, setLoading] = useState(false);
  const [message, setMessage] = useState("");
  const fileInputRef = useRef<HTMLInputElement>(null);

  const loadBlueprints = async () => {
    try {
      const list = await invoke<string[]>("list_cached_blueprints");
      const bps: Blueprint[] = list.map((name) => ({ name }));
      setBlueprints(bps);
    } catch (e) {
      console.error("Failed to load blueprints:", e);
    }
  };

  const handleImportUrl = async () => {
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

  const handleFileUpload = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    setLoading(true);
    setMessage(`Uploading ${file.name}...`);

    try {
      const text = await file.text();
      const name = file.name.replace(/\.(json|blueprint)$/, "");

      await invoke<string>("save_blueprint", {
        name,
        data: text,
        author: "local",
      });

      setMessage(`Imported ${name}`);
      loadBlueprints();
    } catch (e) {
      setMessage(`Upload failed: ${e}`);
    } finally {
      setLoading(false);
      if (fileInputRef.current) {
        fileInputRef.current.value = "";
      }
    }
  };

  return (
    <div className="blueprints-page">
      <h1 className="page-title mc-shadow">Blueprints</h1>

      <div className="import-section">
        <div className="section-header">
          <span className="material-symbols-outlined">download</span>
          <h2>Import Blueprint</h2>
        </div>

        <div className="import-form">
          <input
            type="text"
            placeholder="GrabCraft URL"
            value={importUrl}
            onChange={(e) => setImportUrl(e.target.value)}
            className="input-field mc-bevel-in"
          />
          <input
            type="text"
            placeholder="Name (optional)"
            value={importName}
            onChange={(e) => setImportName(e.target.value)}
            className="input-field mc-bevel-in"
          />
          <button
            onClick={handleImportUrl}
            disabled={loading || !importUrl.trim()}
            className="mc-button"
          >
            {loading ? "Importing..." : "Import URL"}
          </button>
        </div>

        <div className="divider">
          <span>OR</span>
        </div>

        <div className="upload-section">
          <input
            type="file"
            ref={fileInputRef}
            onChange={handleFileUpload}
            accept=".json,.blueprint,.mcfunction,.litematic,.schem,.schematic,.nbt,.mcstructure"
            className="file-input"
            id="file-upload"
          />
          <label htmlFor="file-upload" className="mc-button upload-label">
            <span className="material-symbols-outlined">upload_file</span>
            Upload File
          </label>
          <span className="file-hint">.json, .blueprint, .mcfunction, .litematic, .schem, .schematic, .nbt</span>
        </div>

        {message && (
          <div className={`message ${message.includes("failed") || message.includes("error") ? "error" : "success"}`}>
            {message}
          </div>
        )}
      </div>

      <div className="blueprints-list">
        <div className="section-header">
          <span className="material-symbols-outlined">inventory_2</span>
          <h2>Imported Blueprints</h2>
          <button onClick={loadBlueprints} className="mc-button refresh-btn">
            <span className="material-symbols-outlined">refresh</span>
          </button>
        </div>

        {blueprints.length === 0 ? (
          <div className="empty-state">
            <span className="material-symbols-outlined">folder_open</span>
            <p>No blueprints imported yet.</p>
            <p className="hint">Import from GrabCraft or upload a .json file</p>
          </div>
        ) : (
          <div className="blueprints-grid">
            {blueprints.map((bp, index) => (
              <div key={bp.name} className="blueprint-card mc-bevel-out">
                <div className="blueprint-index">{index + 1}</div>
                <div className="blueprint-info">
                  <h3 className="mc-shadow">{bp.name}</h3>
                  {bp.width && bp.height && bp.length && (
                    <p className="dimensions">
                      {bp.width}x{bp.height}x{bp.length}
                    </p>
                  )}
                </div>
                <span className="material-symbols-outlined blueprint-icon">architecture</span>
              </div>
            ))}
          </div>
        )}
      </div>

      <style>{`
        .blueprints-page {
          padding: var(--block-lg);
          height: 100%;
          display: flex;
          flex-direction: column;
          gap: var(--block-lg);
        }

        .page-title {
          font-family: 'VT323', monospace;
          font-size: 32px;
          color: var(--mc-green);
          margin: 0;
        }

        .section-header {
          display: flex;
          align-items: center;
          gap: var(--block-sm);
          margin-bottom: var(--block-md);
        }

        .section-header h2 {
          font-family: 'Space Mono', monospace;
          font-size: 16px;
          color: var(--on-surface);
          margin: 0;
          flex: 1;
        }

        .section-header .material-symbols-outlined {
          color: var(--mc-green);
        }

        .import-section {
          background: var(--surface-container);
          padding: var(--block-md);
          border: 2px solid var(--outline-variant);
        }

        .import-form {
          display: flex;
          gap: var(--block-sm);
          flex-wrap: wrap;
        }

        .input-field {
          flex: 1;
          min-width: 150px;
          padding: var(--block-sm);
          background: var(--surface);
          color: var(--on-surface);
          font-family: 'Space Mono', monospace;
          font-size: 12px;
          border: none;
          outline: none;
        }

        .input-field:focus {
          border-color: var(--mc-green);
        }

        .input-field::placeholder {
          color: var(--mc-gray);
        }

        .divider {
          display: flex;
          align-items: center;
          margin: var(--block-md) 0;
          color: var(--mc-gray);
          font-family: 'Space Mono', monospace;
          font-size: 12px;
        }

        .divider::before,
        .divider::after {
          content: '';
          flex: 1;
          height: 1px;
          background: var(--outline-variant);
        }

        .divider span {
          padding: 0 var(--block-md);
        }

        .upload-section {
          display: flex;
          align-items: center;
          gap: var(--block-sm);
        }

        .file-input {
          display: none;
        }

        .upload-label {
          display: flex;
          align-items: center;
          gap: var(--block-xs);
          cursor: pointer;
        }

        .upload-label .material-symbols-outlined {
          font-size: 18px;
        }

        .file-hint {
          color: var(--mc-gray);
          font-size: 12px;
        }

        .message {
          margin-top: var(--block-md);
          padding: var(--block-sm);
          font-family: 'Space Mono', monospace;
          font-size: 12px;
        }

        .message.success {
          background: rgba(85, 255, 85, 0.1);
          border: 1px solid var(--mc-green);
          color: var(--mc-green);
        }

        .message.error {
          background: rgba(255, 85, 85, 0.1);
          border: 1px solid var(--mc-red);
          color: var(--mc-red);
        }

        .blueprints-list {
          flex: 1;
          background: var(--surface-container);
          padding: var(--block-md);
          border: 2px solid var(--outline-variant);
        }

        .refresh-btn {
          padding: var(--block-xs) !important;
          min-width: auto !important;
        }

        .refresh-btn .material-symbols-outlined {
          font-size: 18px;
        }

        .empty-state {
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          padding: var(--block-lg);
          color: var(--mc-gray);
          text-align: center;
        }

        .empty-state .material-symbols-outlined {
          font-size: 48px;
          margin-bottom: var(--block-md);
          opacity: 0.5;
        }

        .empty-state p {
          font-family: 'Space Mono', monospace;
          font-size: 14px;
          margin: 0;
        }

        .empty-state .hint {
          font-size: 12px;
          margin-top: var(--block-sm);
          color: var(--outline);
        }

        .blueprints-grid {
          display: grid;
          grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
          gap: var(--block-md);
        }

        .blueprint-card {
          display: flex;
          align-items: center;
          gap: var(--block-md);
          padding: var(--block-md);
          background: var(--surface);
          cursor: pointer;
          transition: background 0.2s;
        }

        .blueprint-card:hover {
          background: var(--surface-container-high);
        }

        .blueprint-index {
          font-family: 'VT323', monospace;
          font-size: 24px;
          color: var(--mc-green);
          min-width: 30px;
          text-align: center;
        }

        .blueprint-info {
          flex: 1;
          min-width: 0;
        }

        .blueprint-info h3 {
          font-family: 'Space Mono', monospace;
          font-size: 14px;
          color: var(--on-surface);
          margin: 0;
          white-space: nowrap;
          overflow: hidden;
          text-overflow: ellipsis;
        }

        .blueprint-info .dimensions {
          font-size: 11px;
          color: var(--mc-gray);
          margin: 4px 0 0;
        }

        .blueprint-icon {
          color: var(--outline);
          font-size: 20px;
        }

        .mc-button {
          padding: var(--block-sm) var(--block-md);
          background: var(--mc-stone);
          border: 4px solid;
          border-color: var(--mc-light-stone) var(--mc-dark-stone) var(--mc-dark-stone) var(--mc-light-stone);
          color: white;
          font-family: 'Space Mono', monospace;
          font-size: 12px;
          cursor: pointer;
          white-space: nowrap;
        }

        .mc-button:hover {
          background: var(--surface-bright);
        }

        .mc-button:active {
          border-color: var(--mc-dark-stone) var(--mc-light-stone) var(--mc-light-stone) var(--mc-dark-stone);
        }

        .mc-button:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }

        .mc-bevel-in {
          border: 4px solid;
          border-color: var(--mc-deep-charcoal) var(--mc-stone) var(--mc-stone) var(--mc-deep-charcoal);
        }

        .mc-bevel-out {
          border: 4px solid;
          border-color: var(--mc-light-stone) var(--mc-dark-stone) var(--mc-dark-stone) var(--mc-light-stone);
        }

        .mc-shadow {
          text-shadow: 1px 1px 0 #000;
        }
      `}</style>
    </div>
  );
}
