import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

// === Type mirrors of src-tauri structs ===

interface ClashClient {
  name: string;
  config_path: string;
  install_root: string;
  looks_valid: boolean;
  source: string;
}

type RuleType =
  | "DOMAIN"
  | "DOMAIN-SUFFIX"
  | "DOMAIN-KEYWORD"
  | "DOMAIN-REGEX"
  | "GEOIP"
  | "PROCESS-NAME"
  | "PROCESS-PATH"
  | "SRC-PROCESS-NAME"
  | "SRC-PROCESS-PATH"
  | "IP-CIDR"
  | "IP-CIDR6"
  | "SRC-IP-CIDR"
  | "SRC-IP-CIDR6"
  | "RULE-SET"
  | "AND"
  | "OR"
  | "NOT"
  | "MATCH";

type RuleGroup = "domain" | "process" | "ip_cidr" | "rule_set" | "match" | "logical";

interface Rule {
  id: number;
  rule_type: RuleType;
  payload: string;
  target: string;
  params: string[];
  disabled_in_source: boolean;
}

interface LoadedConfig {
  config_path: string;
  raw_yaml: string;
  rules: Rule[];
}

// === Group metadata for sidebar ===

const GROUP_ORDER: { key: RuleGroup; label: string; icon: string }[] = [
  { key: "domain", label: "域名规则", icon: "🌐" },
  { key: "process", label: "进程规则", icon: "⚙️" },
  { key: "ip_cidr", label: "IP-CIDR", icon: "🔢" },
  { key: "rule_set", label: "RULE-SET", icon: "📦" },
  { key: "match", label: "兜底 MATCH", icon: "🎯" },
  { key: "logical", label: "逻辑运算", icon: "🧠" },
];

const groupOf = (t: RuleType): RuleGroup => {
  if (t.startsWith("DOMAIN") || t === "GEOIP") return "domain";
  if (t.includes("PROCESS")) return "process";
  if (t.includes("IP-CIDR")) return "ip_cidr";
  if (t === "RULE-SET") return "rule_set";
  if (t === "MATCH") return "match";
  return "logical";
};

// Common targets suggested when adding a new rule
const COMMON_TARGETS = ["DIRECT", "REJECT", "PROXY", "GLOBAL"];

// Common domain rules one-click import (similar to FlClash "common websites")
const COMMON_DOMAINS = [
  "baidu.com",
  "bilibili.com",
  "taobao.com",
  "jd.com",
  "weibo.com",
  "zhihu.com",
  "douyin.com",
  "qq.com",
  "weixin.qq.com",
  "163.com",
  "126.com",
  "csdn.net",
  "gitee.com",
  "aliyun.com",
  "tencent.com",
  "huawei.com",
  "xiaomi.com",
];

// === App ===

export function App() {
  const [clients, setClients] = useState<ClashClient[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const [loaded, setLoaded] = useState<LoadedConfig | null>(null);
  const [dirty, setDirty] = useState(false);

  const [search, setSearch] = useState("");
  const [activeGroup, setActiveGroup] = useState<RuleGroup | "all">("all");
  const [selected, setSelected] = useState<Set<number>>(new Set());
  const [showAddDialog, setShowAddDialog] = useState(false);

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

  async function loadPath(path: string) {
    setError(null);
    setSelected(new Set());
    setSearch("");
    setActiveGroup("all");
    try {
      const cfg = await invoke<LoadedConfig>("load_config", { path });
      setLoaded(cfg);
      setDirty(false);
    } catch (e) {
      setError(`加载失败：${e}`);
    }
  }

  async function pickConfig() {
    const selected = await open({
      multiple: false,
      filters: [{ name: "Clash config", extensions: ["yaml", "yml"] }],
    });
    if (typeof selected === "string") {
      loadPath(selected);
    }
  }

  function mutateRules(next: Rule[]) {
    if (!loaded) return;
    setLoaded({ ...loaded, rules: next });
    setDirty(true);
  }

  function deleteRule(id: number) {
    if (!loaded) return;
    if (!confirm("删除该规则？")) return;
    mutateRules(loaded.rules.filter((r) => r.id !== id));
  }

  function toggleSelected(id: number) {
    const s = new Set(selected);
    if (s.has(id)) s.delete(id);
    else s.add(id);
    setSelected(s);
  }

  function clearSelection() {
    setSelected(new Set());
  }

  function bulkDelete() {
    if (!loaded || selected.size === 0) return;
    if (!confirm(`删除选中的 ${selected.size} 条规则？`)) return;
    mutateRules(loaded.rules.filter((r) => !selected.has(r.id)));
    clearSelection();
  }

  function bulkSetEnabled(_enabled: boolean) {
    // For MVP, "enable/disable" is expressed via disabling in source (commenting).
    // We just clear selection for now since enabled/disabled is a source-level
    // concern handled per-line via disabled_in_source.
    clearSelection();
  }

  async function save() {
    if (!loaded) return;
    if (!confirm(`保存到 ${loaded.config_path}？将覆盖现有文件并创建 .bak 备份。`)) return;
    try {
      const count = await invoke<number>("save_rules", {
        path: loaded.config_path,
        rules: loaded.rules,
      });
      setDirty(false);
      alert(`已保存 ${count} 条规则到 ${loaded.config_path}\n.bak 已创建在同目录`);
    } catch (e) {
      setError(`保存失败：${e}`);
    }
  }

  async function reload() {
    if (!loaded) return;
    if (dirty && !confirm("放弃未保存的修改并重新加载？")) return;
    await loadPath(loaded.config_path);
  }

  function importCommon() {
    if (!loaded) return;
    const newRules: Rule[] = COMMON_DOMAINS.map((d) => ({
      id: 0, // backend will reassign on save
      rule_type: "DOMAIN-SUFFIX",
      payload: d,
      target: "DIRECT",
      params: [],
      disabled_in_source: false,
    }));
    mutateRules([...loaded.rules, ...newRules]);
  }

  // === Filtered rules ===
  const filteredRules = useMemo(() => {
    if (!loaded) return [];
    const q = search.trim().toLowerCase();
    return loaded.rules.filter((r) => {
      if (activeGroup !== "all" && groupOf(r.rule_type) !== activeGroup) return false;
      if (q && !`${r.payload} ${r.target}`.toLowerCase().includes(q)) return false;
      return true;
    });
  }, [loaded, search, activeGroup]);

  const groupCounts = useMemo(() => {
    const m: Record<RuleGroup, number> = {
      domain: 0,
      process: 0,
      ip_cidr: 0,
      rule_set: 0,
      match: 0,
      logical: 0,
    };
    if (!loaded) return m;
    for (const r of loaded.rules) m[groupOf(r.rule_type)]++;
    return m;
  }, [loaded]);

  // === Render ===

  return (
    <div className="app">
      <header className="header">
        <div className="header-left">
          <h1>PlaneClash Manage</h1>
          <p className="subtitle">独立的 Clash 规则管理器</p>
        </div>
        <div className="header-right">
          {loaded && (
            <>
              <button onClick={reload} className="btn-secondary">
                ↻ 重新加载
              </button>
              <button onClick={save} className={dirty ? "btn-primary btn-dirty" : "btn-primary"}>
                {dirty ? "保存（已修改）" : "保存"}
              </button>
            </>
          )}
        </div>
      </header>

      {error && <div className="error">⚠ {error}</div>}

      {!loaded && (
        <ClientPicker
          clients={clients}
          loading={loading}
          onRescan={refresh}
          onPickClient={(c) => loadPath(c.config_path)}
          onPickManual={pickConfig}
        />
      )}

      {loaded && (
        <RuleEditor
          loaded={loaded}
          dirty={dirty}
          search={search}
          setSearch={setSearch}
          activeGroup={activeGroup}
          setActiveGroup={setActiveGroup}
          selected={selected}
          toggleSelected={toggleSelected}
          clearSelection={clearSelection}
          bulkDelete={bulkDelete}
          bulkSetEnabled={bulkSetEnabled}
          deleteRule={deleteRule}
          filteredRules={filteredRules}
          groupCounts={groupCounts}
          onAdd={() => setShowAddDialog(true)}
          onImportCommon={importCommon}
          onClose={() => {
            if (dirty && !confirm("放弃未保存的修改并关闭？")) return;
            setLoaded(null);
            setSelected(new Set());
            setDirty(false);
          }}
        />
      )}

      {showAddDialog && loaded && (
        <AddRuleDialog
          existing={loaded.rules}
          onCancel={() => setShowAddDialog(false)}
          onAdd={(r) => {
            mutateRules([...loaded.rules, r]);
            setShowAddDialog(false);
          }}
        />
      )}

      <footer className="footer">
        <span>PlaneClash Manage v0.1.0 · MVP Step 1-6</span>
      </footer>
    </div>
  );
}

// === ClientPicker ===

function ClientPicker(props: {
  clients: ClashClient[];
  loading: boolean;
  onRescan: () => void;
  onPickClient: (c: ClashClient) => void;
  onPickManual: () => void;
}) {
  return (
    <section className="card">
      <div className="card-header">
        <h2>选择要管理的 Clash 客户端</h2>
        <div style={{ display: "flex", gap: 8 }}>
          <button onClick={props.onPickManual} className="btn-secondary">
            手动选择 config.yaml
          </button>
          <button onClick={props.onRescan} disabled={props.loading}>
            {props.loading ? "扫描中…" : "重新扫描"}
          </button>
        </div>
      </div>

      {props.clients.length === 0 && !props.loading && (
        <div className="empty">
          未检测到任何 Clash 客户端。请确认 FlClash / Clash Verge / mihomo 已安装，或点上方"手动选择"。
        </div>
      )}

      <ul className="client-list">
        {props.clients.map((c) => (
          <li key={c.config_path} className="client-item" onClick={() => props.onPickClient(c)}>
            <div className="client-main">
              <span className="client-name">{c.name}</span>
              {!c.looks_valid && <span className="badge badge-warn">配置文件不可识别</span>}
            </div>
            <div className="client-path">{c.config_path}</div>
            <div className="client-source">来源：{c.source}</div>
            <div className="client-cta">点击打开 →</div>
          </li>
        ))}
      </ul>
    </section>
  );
}

// === RuleEditor ===

function RuleEditor(props: {
  loaded: LoadedConfig;
  dirty: boolean;
  search: string;
  setSearch: (s: string) => void;
  activeGroup: RuleGroup | "all";
  setActiveGroup: (g: RuleGroup | "all") => void;
  selected: Set<number>;
  toggleSelected: (id: number) => void;
  clearSelection: () => void;
  bulkDelete: () => void;
  bulkSetEnabled: (enabled: boolean) => void;
  deleteRule: (id: number) => void;
  filteredRules: Rule[];
  groupCounts: Record<RuleGroup, number>;
  onAdd: () => void;
  onImportCommon: () => void;
  onClose: () => void;
}) {
  const total = props.loaded.rules.length;
  const enabledCount = props.loaded.rules.filter((r) => !r.disabled_in_source).length;
  const selectedCount = props.selected.size;

  return (
    <section className="card editor">
      <div className="card-header">
        <div>
          <h2>{props.loaded.config_path}</h2>
          <p className="hint">
            共 {total} 条规则，已启用 {enabledCount} 条
            {props.dirty && <span className="badge badge-warn">未保存</span>}
          </p>
        </div>
        <div style={{ display: "flex", gap: 8 }}>
          <button onClick={props.onImportCommon} className="btn-secondary">
            导入常见国内域名
          </button>
          <button onClick={props.onAdd} className="btn-primary">
            + 添加规则
          </button>
          <button onClick={props.onClose} className="btn-secondary">
            ✕ 关闭
          </button>
        </div>
      </div>

      <div className="toolbar">
        <input
          className="search"
          type="text"
          placeholder="搜索 payload 或 target…"
          value={props.search}
          onChange={(e) => props.setSearch(e.target.value)}
        />
        <div className="group-tabs">
          <button
            className={props.activeGroup === "all" ? "tab active" : "tab"}
            onClick={() => props.setActiveGroup("all")}
          >
            全部 ({total})
          </button>
          {GROUP_ORDER.map((g) => (
            <button
              key={g.key}
              className={props.activeGroup === g.key ? "tab active" : "tab"}
              onClick={() => props.setActiveGroup(g.key)}
            >
              {g.icon} {g.label} ({props.groupCounts[g.key]})
            </button>
          ))}
        </div>
      </div>

      <RuleTable
        rules={props.filteredRules}
        selected={props.selected}
        toggleSelected={props.toggleSelected}
        onDelete={props.deleteRule}
      />

      {selectedCount > 0 && (
        <div className="bulk-bar">
          <span>已选 {selectedCount} 项</span>
          <button onClick={props.bulkDelete} className="btn-danger">
            删除选中
          </button>
          <button onClick={() => props.bulkSetEnabled(true)} className="btn-secondary">
            全部启用
          </button>
          <button onClick={() => props.bulkSetEnabled(false)} className="btn-secondary">
            全部禁用（注释）
          </button>
          <button onClick={props.clearSelection} className="btn-secondary">
            取消选择
          </button>
        </div>
      )}
    </section>
  );
}

function RuleTable(props: {
  rules: Rule[];
  selected: Set<number>;
  toggleSelected: (id: number) => void;
  onDelete: (id: number) => void;
}) {
  if (props.rules.length === 0) {
    return <div className="empty">没有匹配的规则</div>;
  }
  return (
    <table className="rule-table">
      <thead>
        <tr>
          <th style={{ width: 32 }}></th>
          <th style={{ width: 140 }}>类型</th>
          <th>Payload</th>
          <th style={{ width: 120 }}>Target</th>
          <th style={{ width: 120 }}>参数</th>
          <th style={{ width: 60 }}></th>
        </tr>
      </thead>
      <tbody>
        {props.rules.map((r) => (
          <tr
            key={r.id}
            className={
              (r.disabled_in_source ? "row-disabled " : "") +
              (props.selected.has(r.id) ? "row-selected" : "")
            }
          >
            <td>
              <input
                type="checkbox"
                checked={props.selected.has(r.id)}
                onChange={() => props.toggleSelected(r.id)}
              />
            </td>
            <td>
              <span className={`type-tag type-${groupOf(r.rule_type)}`}>{r.rule_type}</span>
            </td>
            <td className="mono">{r.payload || <em className="muted">（无）</em>}</td>
            <td className="mono">{r.target}</td>
            <td className="mono">{r.params.join(", ") || <em className="muted">—</em>}</td>
            <td>
              <button onClick={() => props.onDelete(r.id)} className="btn-icon" title="删除">
                🗑
              </button>
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

// === AddRuleDialog ===

function AddRuleDialog(props: {
  existing: Rule[];
  onCancel: () => void;
  onAdd: (r: Rule) => void;
}) {
  const CREATABLE: RuleType[] = [
    "DOMAIN",
    "DOMAIN-SUFFIX",
    "DOMAIN-KEYWORD",
    "DOMAIN-REGEX",
    "GEOIP",
    "PROCESS-NAME",
    "PROCESS-PATH",
    "IP-CIDR",
    "IP-CIDR6",
    "SRC-IP-CIDR",
    "RULE-SET",
    "MATCH",
  ];
  const [ruleType, setRuleType] = useState<RuleType>("DOMAIN-SUFFIX");
  const [payload, setPayload] = useState("");
  const [target, setTarget] = useState("DIRECT");
  const [params, setParams] = useState("");

  function submit() {
    if (!payload && ruleType !== "MATCH") {
      alert("请填写 payload（规则要匹配的内容）");
      return;
    }
    const nextId = props.existing.reduce((m, r) => Math.max(m, r.id), 0) + 1;
    props.onAdd({
      id: nextId,
      rule_type: ruleType,
      payload: ruleType === "MATCH" ? "" : payload.trim(),
      target: target.trim() || "DIRECT",
      params: params
        .split(",")
        .map((s) => s.trim())
        .filter(Boolean),
      disabled_in_source: false,
    });
  }

  return (
    <div className="modal-backdrop" onClick={props.onCancel}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <h3>添加规则</h3>
        <div className="form-row">
          <label>类型</label>
          <select value={ruleType} onChange={(e) => setRuleType(e.target.value as RuleType)}>
            {CREATABLE.map((t) => (
              <option key={t} value={t}>
                {t}
              </option>
            ))}
          </select>
        </div>
        {ruleType !== "MATCH" && (
          <div className="form-row">
            <label>Payload</label>
            <input
              type="text"
              value={payload}
              onChange={(e) => setPayload(e.target.value)}
              placeholder={
                ruleType.startsWith("DOMAIN")
                  ? "例如 baidu.com"
                  : ruleType.startsWith("PROCESS")
                    ? "例如 WeChatApp.exe"
                    : ruleType.startsWith("IP-CIDR")
                      ? "例如 192.168.0.0/16"
                      : ruleType === "RULE-SET"
                        ? "ruleset 名称（须在 rule-providers 定义）"
                        : ""
              }
              autoFocus
            />
          </div>
        )}
        <div className="form-row">
          <label>Target</label>
          <input
            type="text"
            value={target}
            onChange={(e) => setTarget(e.target.value)}
            list="common-targets"
            placeholder="DIRECT / REJECT / 代理组名"
          />
          <datalist id="common-targets">
            {COMMON_TARGETS.map((t) => (
              <option key={t} value={t} />
            ))}
          </datalist>
        </div>
        <div className="form-row">
          <label>参数（可选，逗号分隔）</label>
          <input
            type="text"
            value={params}
            onChange={(e) => setParams(e.target.value)}
            placeholder="例如 no-resolve"
          />
        </div>
        <div className="form-actions">
          <button onClick={props.onCancel} className="btn-secondary">
            取消
          </button>
          <button onClick={submit} className="btn-primary">
            添加
          </button>
        </div>
      </div>
    </div>
  );
}