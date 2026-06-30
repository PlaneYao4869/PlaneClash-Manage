import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface ClashClient {
  name: string;
  config_path: string;
  install_root: string;
  looks_valid: boolean;
  source: string;
}

export function App() {
  const [clients, setClients] = useState<ClashClient[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function refresh() {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<ClashClient[]>("scan_clients");
      setClients(result);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    refresh();
  }, []);

  return (
    <div className="app">
      <header className="header">
        <h1>PlaneClash Manage</h1>
        <p className="subtitle">
          独立的 Clash 规则管理器 · 域名 / 进程 / IP-CIDR 一站式编辑
        </p>
      </header>

      <section className="card">
        <div className="card-header">
          <h2>已检测到的 Clash 客户端</h2>
          <button onClick={refresh} disabled={loading}>
            {loading ? "扫描中…" : "重新扫描"}
          </button>
        </div>
        {error && <div className="error">扫描失败：{error}</div>}
        {clients.length === 0 && !loading && !error && (
          <div className="empty">
            未检测到任何 Clash 客户端。请确认 FlClash / Clash Verge / mihomo 已安装。
          </div>
        )}
        <ul className="client-list">
          {clients.map((c) => (
            <li key={c.config_path} className="client-item">
              <div className="client-main">
                <span className="client-name">{c.name}</span>
                {!c.looks_valid && (
                  <span className="badge badge-warn">配置文件不可识别</span>
                )}
              </div>
              <div className="client-path" title={c.config_path}>
                {c.config_path}
              </div>
              <div className="client-source">来源：{c.source}</div>
            </li>
          ))}
        </ul>
      </section>

      <footer className="footer">
        <span>PlaneClash Manage v0.1.0 · MVP Step 1（扫描）</span>
      </footer>
    </div>
  );
}
