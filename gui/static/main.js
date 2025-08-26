// State & elements
let monacoEditor = null;
let currentSymbols = [];
let outlineTimer = null;
let activePtyId = localStorage.getItem('aeonmi.lastPty') || null;
const ptySelect = document.getElementById('ptyList');
const ptyMode = document.getElementById('ptyMode');
const ptyTitleInput = document.getElementById('ptyTitle');
const statusEl = document.getElementById('status');
const termDiv = document.getElementById('xterm');
const exportBtn = document.getElementById('exportPty');
const copyBtn = document.getElementById('copyPty');
const saveBtn = document.getElementById('savePty');
const btnQuantumSim = document.getElementById('btnQuantumSim');
const btnAIStream = document.getElementById('btnAIStream');
const aiPrompt = document.getElementById('aiPrompt');
const aiStreamOut = document.getElementById('aiStreamOut');
const aiHistory = document.getElementById('aiHistory');
const autoQuantumSim = document.getElementById('autoQuantumSim');
const clearAIHistBtn = document.getElementById('clearAIHist');
const autoAICheck = document.getElementById('autoAI');
const aiHistLimitInput = document.getElementById('aiHistLimit');
const themeToggle = document.getElementById('themeToggle');
const openSettings = document.getElementById('openSettings');
const settingsPanel = document.getElementById('settingsPanel');
const closeSettings = document.getElementById('closeSettings');
const prefTheme = document.getElementById('prefTheme');
const prefHistLimit = document.getElementById('prefHistLimit');
const prefAutoAI = document.getElementById('prefAutoAI');
const prefAutoQuantum = document.getElementById('prefAutoQuantum');
const savePrefsBtn = document.getElementById('savePrefs');
const codeActionsBtn = document.getElementById('codeActionsBtn');
const typeCheckBtn = document.getElementById('typeCheckBtn');
const aiProviderSelect = document.getElementById('aiProvider');
const circuitCanvas = document.getElementById('circuitCanvas');
const quantumCircuitPanel = document.getElementById('quantumCircuitPanel');
const circuitText = document.getElementById('circuitText');
const hideCircuitBtn = document.getElementById('hideCircuit');
const cacheLogToggle = document.getElementById('cacheLogToggle');
const exportCircuitBtn = document.getElementById('exportCircuit');
const showMetricsBtn = document.getElementById('showMetrics');
const metricsPanel = document.getElementById('metricsPanel');
const metricsContent = document.getElementById('metricsContent');
const closeMetrics = document.getElementById('closeMetrics');
const resetMetricsBtn = document.getElementById('resetMetrics');
const resetMetricsFullBtn = document.getElementById('resetMetricsFull');
const deepPropToggle = document.getElementById('deepPropToggle');
let cachedActions = [];
let autoSimEnabled = false;
let quantumSimTimer = null;
let autoAIEnabled = false;
let aiDebounceTimer = null;
let currentAIStreamId = 0;
let currentAIUnlisten = null;
if (hideCircuitBtn) hideCircuitBtn.addEventListener('click', () => { if (quantumCircuitPanel) quantumCircuitPanel.style.display='none'; });
// Cache logging toggle persistence
if (cacheLogToggle && window.__TAURI__?.invoke) {
  const stored = localStorage.getItem('aeonmi.cacheLogging') === '1';
  cacheLogToggle.checked = stored;
  // initialize backend
  window.__TAURI__.invoke('cache_logging', { enable: stored }).catch(()=>{});
  cacheLogToggle.addEventListener('change', () => {
    const en = cacheLogToggle.checked; localStorage.setItem('aeonmi.cacheLogging', en?'1':'0');
    window.__TAURI__.invoke('cache_logging', { enable: en }).catch(()=>{});
  });
}

// Circuit export with custom filename prompt
if (exportCircuitBtn && window.__TAURI__?.invoke) {
  exportCircuitBtn.addEventListener('click', async () => {
    if (!monacoEditor) return; const src = monacoEditor.getValue();
    try {
      const respStr = await window.__TAURI__.invoke('aeonmi_quantum_circuit_export', { source: src });
      let data = {}; try { data = JSON.parse(respStr); } catch {}
      const ts = new Date().toISOString().replace(/[:.]/g,'-');
      const base = prompt('Circuit base filename (no extension):', 'circuit_'+ts) || ('circuit_'+ts);
      const jsonName = base + '.json'; const qasmName = base + '.qasm.txt';
      if (window.__TAURI__?.dialog?.save) {
        // Offer to save both; if user picks first path, derive directory for second
        const dirPath = await window.__TAURI__.dialog.save({ title:'Save circuit JSON', defaultPath: jsonName });
        if (dirPath) {
          const pathMod = dirPath.endsWith('.json') ? dirPath : dirPath + '.json';
          await window.__TAURI__.invoke('save_file', { path: pathMod, contents: data.json||'' });
          const qasmPath = pathMod.replace(/\.json$/i, '.qasm.txt');
          await window.__TAURI__.invoke('save_file', { path: qasmPath, contents: data.pseudo_qasm||'' });
          term.writeln('[Circuit exported: '+pathMod+' & '+qasmPath+']');
        } else { term.writeln('[Circuit export canceled]'); }
      } else {
        // Fallback download via browser
        const blob1 = new Blob([data.json||''], { type:'application/json' });
        const a1 = document.createElement('a'); a1.href = URL.createObjectURL(blob1); a1.download = jsonName; a1.click(); setTimeout(()=>URL.revokeObjectURL(a1.href), 1500);
        const blob2 = new Blob([data.pseudo_qasm||''], { type:'text/plain' });
        const a2 = document.createElement('a'); a2.href = URL.createObjectURL(blob2); a2.download = qasmName; a2.click(); setTimeout(()=>URL.revokeObjectURL(a2.href), 1500);
        term.writeln('[Circuit exported (download): '+jsonName+' & '+qasmName+']');
      }
    } catch(e) { term.writeln('[Circuit export failed: '+e+']'); }
  });
}

// Metrics panel
async function refreshMetrics() {
  if (!window.__TAURI__?.invoke || !metricsContent) return;
  try { const raw = await window.__TAURI__.invoke('aeonmi_metrics'); let obj={}; try{obj=JSON.parse(raw);}catch{}; metricsContent.textContent = JSON.stringify(obj, null, 2); } catch(e){ metricsContent.textContent='(error '+e+')'; }
}
showMetricsBtn?.addEventListener('click', () => { if (!metricsPanel) return; metricsPanel.style.display='block'; refreshMetrics(); });
closeMetrics?.addEventListener('click', ()=> { if (metricsPanel) metricsPanel.style.display='none'; });
resetMetricsBtn?.addEventListener('click', async () => { if (window.__TAURI__?.invoke) { try { await window.__TAURI__.invoke('metrics_reset'); refreshMetrics(); } catch(e) { metricsContent.textContent = '(reset failed '+e+')'; } } });
resetMetricsFullBtn?.addEventListener('click', async () => { if (window.__TAURI__?.invoke && confirm('Full reset will clear all metrics, dependencies, and timings. Continue?')) { try { await window.__TAURI__.invoke('metrics_reset_full'); refreshMetrics(); } catch(e) { metricsContent.textContent = '(full reset failed '+e+')'; } } });
if (deepPropToggle && window.__TAURI__?.invoke) {
  (async () => { try { const v = await window.__TAURI__.invoke('metrics_get_deep'); deepPropToggle.checked = !!v; } catch {} })();
  deepPropToggle.addEventListener('change', () => { window.__TAURI__.invoke('metrics_set_deep', { enable: deepPropToggle.checked }).catch(()=>{}); });
}
setInterval(()=> { if (metricsPanel && metricsPanel.style.display!=='none') refreshMetrics(); }, 6000);
// Populate AI providers
async function loadProviders() {
  if (!window.__TAURI__?.invoke || !aiProviderSelect) return;
  try { const list = await window.__TAURI__.invoke('ai_list_providers'); if (Array.isArray(list)) { aiProviderSelect.innerHTML = '<option value="">(prov)</option>' + list.map(p=>`<option value="${p}">${p}</option>`).join(''); } } catch {}
}
aiProviderSelect?.addEventListener('change', async () => { const v = aiProviderSelect.value; if (!v) return; try { await window.__TAURI__.invoke('ai_set_provider', { name: v }); term.writeln('[AI provider set: '+v+']'); } catch(e){ term.writeln('[Set provider failed]'); } });
loadProviders();
let lastCircuitHash = null;

function setStatus(msg) { if (!statusEl) return; if (msg) { statusEl.textContent = msg; statusEl.style.display='block'; } else { statusEl.style.display='none'; } }

// Periodic cache stats (if enabled by toggle) appended to status
async function pollCacheStats() {
  if (!window.__TAURI__?.invoke || !statusEl) return;
  try {
    const raw = await window.__TAURI__.invoke('cache_stats_get');
    let js = {}; try { js = JSON.parse(raw); } catch {}
    if (cacheLogToggle?.checked) {
      statusEl.style.display='block';
      statusEl.textContent = `Cache: ${js.entries||0} entries / ${(js.bytes||0)} bytes`;
    }
  } catch {}
}
setInterval(pollCacheStats, 4000);

// Monaco loader
require.config({ paths: { 'vs': 'https://cdn.jsdelivr.net/npm/monaco-editor@0.38.0/min/vs' } });
require(['vs/editor/editor.main'], function () {
  const scr = document.createElement('script');
  scr.src = 'monaco_ai_language.js';
  scr.onload = function() {
    monacoEditor = monaco.editor.create(document.getElementById('editorMonaco'), {
      value: "// Aeonmi editor (aeonmi language)\n",
      language: 'aeonmi',
      automaticLayout: true,
      fontFamily: 'monospace',
      minimap: { enabled: false }
    });
    let diagTimer = null;
    monacoEditor.onDidChangeModelContent(() => {
      if (diagTimer) clearTimeout(diagTimer);
      diagTimer = setTimeout(() => { runDiagnostics(); scheduleSymbols(); }, 350);
    });
    runDiagnostics();
    scheduleSymbols();
    registerHover();
  };
  document.head.appendChild(scr);
});

// Terminal / PTY
const term = new Terminal({ cursorBlink: true, fontFamily: 'monospace' });
term.open(termDiv);
term.focus();

async function refreshPtyList(selectionChanged) {
  if (!window.__TAURI__?.invoke) return;
  try {
    const list = await window.__TAURI__.invoke('pty_list_detailed');
    if (Array.isArray(list)) {
      ptySelect.innerHTML = '';
      list.forEach(item => {
        const id = item.id; const title = item.title || id;
        const opt = document.createElement('option'); opt.value=id; opt.textContent=`${title} (${id})`; if (id===activePtyId) opt.selected=true; ptySelect.appendChild(opt);
      });
      const ids = list.map(i=>i.id);
      if (selectionChanged && ids.length && !ids.includes(activePtyId)) { activePtyId = ids[0]; }
    }
  } catch {}
}

async function replayBuffer() {
  if (!activePtyId) return;
  try {
    const buf = await window.__TAURI__.invoke('pty_buffer', { id: activePtyId });
    if (typeof buf === 'string') { term.clear(); term.write(buf); }
  } catch {}
}

ptySelect?.addEventListener('change', () => { activePtyId = ptySelect.value; localStorage.setItem('aeonmi.lastPty', activePtyId); replayBuffer(); term.write(`\r\n[Switched active PTY to ${activePtyId}]\r\n`); });
document.getElementById('renamePty').addEventListener('click', async () => {
exportBtn.addEventListener('click', async () => {
  if (!activePtyId) return;
  try {
    const data = await window.__TAURI__.invoke('pty_export', { id: activePtyId });
    // download as file
    const blob = new Blob([data], { type: 'text/plain' });
    const a = document.createElement('a'); a.href = URL.createObjectURL(blob); a.download = `${activePtyId}.log`; a.click();
    setTimeout(()=>URL.revokeObjectURL(a.href), 2000);
  } catch (e) { term.writeln(`[Export failed: ${e}]`); }
});
copyBtn.addEventListener('click', async () => {
  if (!activePtyId) return;
  try { const data = await window.__TAURI__.invoke('pty_export', { id: activePtyId }); await navigator.clipboard.writeText(data); term.writeln('[Buffer copied]'); } catch(e){ term.writeln(`[Copy failed: ${e}]`);} });
saveBtn.addEventListener('click', async () => {
  if (!activePtyId) return;
  try {
    const data = await window.__TAURI__.invoke('pty_export', { id: activePtyId });
    if (window.__TAURI__?.dialog?.save) {
      const path = await window.__TAURI__.dialog.save({ title: 'Save PTY Buffer', defaultPath: `${activePtyId}.log` });
      if (path) {
        await window.__TAURI__.fs.writeTextFile(path, data);
        term.writeln(`[Saved buffer to ${path}]`);
      } else { term.writeln('[Save canceled]'); }
    } else {
      term.writeln('[Save dialog not available – using download fallback]');
      exportBtn.click();
    }
  } catch(e){ term.writeln(`[Save failed: ${e}]`); }
});
  if (!activePtyId) return;
  const t = ptyTitleInput.value.trim(); if (!t) return;
  await window.__TAURI__.invoke('pty_rename', { id: activePtyId, title: t }).catch(()=>{});
  refreshPtyList(false);
});

async function connectPty() {
  if (!window.__TAURI__?.invoke) { alert('Tauri API not available'); return; }
  if (activePtyId) {
    await window.__TAURI__.invoke('pty_close', { id: activePtyId }).catch(()=>{});
    term.writeln('\r\n[PTY closed]\r\n');
    activePtyId = null;
    refreshPtyList(true);
    return;
  }
  try {
    setStatus('Spawning...');
    const repl = ptyMode.value === 'repl';
    const title = ptyTitleInput.value.trim() || undefined;
    const id = await window.__TAURI__.invoke('pty_create', { repl, title });
    activePtyId = id; localStorage.setItem('aeonmi.lastPty', activePtyId);
    term.writeln(`[PTY created ${id}]`);
    refreshPtyList(true);
  } catch (e) { term.writeln(`[PTY create failed: ${e}]`); }
  setStatus(null);
}

if (window.__TAURI__?.event) {
  window.__TAURI__.event.listen('pty-data', ev => {
    const p = ev.payload || {}; if (p.id === activePtyId && p.data) term.write(p.data);
  });
  window.__TAURI__.event.listen('pty-exit', ev => {
    const p = ev.payload || {}; if (p.id === activePtyId) { term.writeln('\r\n[process exited]\r\n'); activePtyId = null; refreshPtyList(true); }
  });
}

term.onData(d => { if (activePtyId && window.__TAURI__?.invoke) window.__TAURI__.invoke('pty_write', { id: activePtyId, data: d }).catch(()=>{}); });

function scheduleResize() {
  if (activePtyId && window.__TAURI__?.invoke) {
    window.__TAURI__.invoke('pty_resize', { id: activePtyId, cols: term.cols, rows: term.rows }).catch(()=>{});
  }
}
window.addEventListener('resize', scheduleResize);
setTimeout(scheduleResize, 500);
refreshPtyList(false).then(()=> { if (activePtyId) replayBuffer(); });

document.getElementById('startTui').addEventListener('click', connectPty);
document.getElementById('openNative').addEventListener('click', async () => {
  const res = await fetch('/api/detach', { method: 'POST' });
  const j = await res.json(); if (j.ok) term.writeln('[Launched native terminal]'); else term.writeln('[Detach failed]');
});

// File open/save logic (unchanged apart from placement)
function pickLanguageForFile(path) {
  if (!path) return 'aeonmi';
  if (path.endsWith('.ai')) return 'aeonmi';
  if (path.endsWith('.js')) return 'javascript';
  if (path.endsWith('.ts')) return 'typescript';
  if (path.endsWith('.rs')) return 'rust';
  if (path.endsWith('.py')) return 'python';
  return 'plaintext';
}

document.getElementById('openBtn').addEventListener('click', async () => {
  const file = document.getElementById('filePath').value; if (!file) return alert('enter file path');
  const res = await fetch(`/api/open?file=${encodeURIComponent(file)}`); const j = await res.json();
  if (j.ok) { if (monacoEditor) { monacoEditor.setValue(j.content); const lang = pickLanguageForFile(file.toLowerCase()); monaco.editor.setModelLanguage(monacoEditor.getModel(), lang); } }
});
document.getElementById('saveBtn').addEventListener('click', async () => {
  const file = document.getElementById('filePath').value; if (!file) return alert('enter file path');
  const content = monacoEditor ? monacoEditor.getValue() : '';
  const res = await fetch('/api/save', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ file, content }) }); const j = await res.json(); if (!j.ok) alert('save failed');
});

// Diagnostics & symbols
async function runDiagnostics() {
  if (!monacoEditor) return;
  const src = monacoEditor.getValue();
  try {
    let diagPayload = null;
    if (window.__TAURI__?.invoke) {
      try { const val = await window.__TAURI__.invoke('aeonmi_diagnostics', { source: src }); diagPayload = typeof val === 'string' ? JSON.parse(val) : val; } catch (e) { console.warn('tauri diagnostics invoke failed', e); }
    }
    if (!diagPayload) {
      const res = await fetch('/diagnostics', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ source: src, path: '<memory>' }) });
      const j = await res.json(); if (j.ok) diagPayload = j; else diagPayload = { diagnostics: [] };
    }
    let baseMarkers = [];
    if (diagPayload) {
      baseMarkers = (diagPayload.diagnostics || []).map(d => ({
        message: d.message,
        severity: d.severity === 'warning' ? monaco.MarkerSeverity.Warning : monaco.MarkerSeverity.Error,
        startLineNumber: d.line || 1,
        startColumn: d.column || 1,
        endLineNumber: d.endLine || d.line || 1,
        endColumn: d.endColumn || (d.column || 1) + 1
      }));
    }
    // Fetch type diagnostics (non-blocking best effort)
    let typeMarkers = [];
    if (window.__TAURI__?.invoke) {
      try {
        const raw = await window.__TAURI__.invoke('aeonmi_types', { source: src });
        const tds = typeof raw === 'string' ? JSON.parse(raw) : raw;
        if (Array.isArray(tds)) {
          typeMarkers = tds.map(td => ({
            message: td.message,
            severity: monaco.MarkerSeverity.Info,
            startLineNumber: td.line || 1,
            startColumn: td.column || 1,
            endLineNumber: td.line || 1,
            endColumn: (td.column || 1) + 1
          }));
        }
      } catch(e) { /* ignore type errors for now */ }
    }
    monaco.editor.setModelMarkers(monacoEditor.getModel(), 'aeonmi', [...baseMarkers, ...typeMarkers]);
    // Hook for quantum circuit visualization refresh (placeholder)
    if (typeof updateQuantumCircuit === 'function') { try { updateQuantumCircuit(src); } catch(e) { console.warn('circuit update failed', e); } }
  } catch (e) { console.warn('diagnostics failed', e); }
}

function scheduleSymbols() { if (outlineTimer) clearTimeout(outlineTimer); outlineTimer = setTimeout(fetchSymbols, 400); }
async function fetchSymbols() {
  if (!monacoEditor) return; const src = monacoEditor.getValue(); let syms = [];
  if (window.__TAURI__?.invoke) {
    try { const raw = await window.__TAURI__.invoke('aeonmi_symbols', { source: src }); const parsed = typeof raw === 'string' ? JSON.parse(raw) : raw; if (Array.isArray(parsed)) syms = parsed; } catch (e) { console.warn('symbols invoke failed', e); }
  }
  currentSymbols = syms; renderOutline();
}

function renderOutline() {
  const list = document.getElementById('outlineList'); if (!list) return; list.innerHTML='';
  if (!currentSymbols.length) { const li = document.createElement('li'); li.textContent='(empty)'; li.className='dim'; list.appendChild(li); return; }
  const kindEmoji = { function:'ƒ', variable:'𝓋', parameter:'π' };
  currentSymbols.forEach(sym => {
    const li = document.createElement('li');
    li.innerHTML = `<span class="kind">${kindEmoji[sym.kind]||sym.kind}</span>${sym.name} <span class="dim">(${sym.line}:${sym.column})</span>`;
    li.addEventListener('click', () => { if (!monacoEditor) return; monacoEditor.revealPositionInCenter({ lineNumber: sym.line, column: sym.column }); monacoEditor.setSelection({ startLineNumber: sym.line, startColumn: sym.column, endLineNumber: sym.end_line||sym.line, endColumn: sym.end_column||sym.column+1 }); monacoEditor.focus(); });
    li.addEventListener('contextmenu', e => {
      e.preventDefault();
      showOutlineContextMenu(e.pageX, e.pageY, sym);
    });
    list.appendChild(li);
  });
}

function showOutlineContextMenu(x, y, sym) {
  let menu = document.getElementById('outlineCtx');
  if (!menu) { menu = document.createElement('div'); menu.id='outlineCtx'; document.body.appendChild(menu); }
  menu.innerHTML = '';
  Object.assign(menu.style, { position:'absolute', top: y+'px', left: x+'px', background:'#fff', border:'1px solid #ccc', fontSize:'12px', padding:'4px 0', zIndex:1000, boxShadow:'0 2px 6px rgba(0,0,0,0.15)' });
  const items = [
  { label: 'Go to Definition', action: () => { if (!monacoEditor) return; monacoEditor.revealPositionInCenter({ lineNumber: sym.line, column: sym.column }); monacoEditor.focus(); } },
  { label: 'Copy Name', action: () => navigator.clipboard.writeText(sym.name).catch(()=>{}) },
  { label: 'Rename Symbol', action: () => renameSymbol(sym) },
  { label: 'Extract Function (stub)', action: () => extractFunctionStub(sym) }
  ];
  items.forEach(it => { const el = document.createElement('div'); el.textContent = it.label; Object.assign(el.style, { padding:'3px 16px', cursor:'pointer' }); el.addEventListener('mouseenter', ()=>el.style.background='#eee'); el.addEventListener('mouseleave', ()=>el.style.background='transparent'); el.addEventListener('click', () => { it.action(); hideOutlineContextMenu(); }); menu.appendChild(el); });
  document.addEventListener('click', hideOutlineContextMenu, { once: true });
}
function hideOutlineContextMenu() { const menu = document.getElementById('outlineCtx'); if (menu) menu.remove(); }

function renameSymbol(sym) {
  if (!monacoEditor) return; const oldName = sym.name;
  const newName = prompt(`Rename symbol '${oldName}' to:`, oldName);
  if (!newName || newName === oldName) return;
  const model = monacoEditor.getModel(); if (!model) return;
  const full = model.getValue();
  // word boundary replace
  const re = new RegExp(`\\b${oldName.replace(/[-/\\^$*+?.()|[\]{}]/g,'\\$&')}\\b`, 'g');
  const updated = full.replace(re, newName);
  model.setValue(updated);
  runDiagnostics(); // refresh markers
  fetchSymbols(); // refresh outline
}

function extractFunctionStub(sym) {
  if (!monacoEditor) return;
  const model = monacoEditor.getModel(); if (!model) return;
  const sel = monacoEditor.getSelection();
  if (!sel || sel.isEmpty()) { alert('Select code to extract first.'); return; }
  const extractName = prompt('New function name:', 'extracted'); if (!extractName) return;
  const startLine = sel.startLineNumber; const endLine = sel.endLineNumber;
  const lines = [];
  for (let l = startLine; l <= endLine; l++) { lines.push(model.getLineContent(l)); }
  const body = lines.join('\n');
  const funcDef = `fn ${extractName}() {\n${body}\n}\n\n`;
  model.applyEdits([
    { range: new monaco.Range(startLine,1,endLine, model.getLineMaxColumn(endLine)), text: `${extractName}();` },
    { range: new monaco.Range(1,1,1,1), text: funcDef }
  ]);
  runDiagnostics(); fetchSymbols();
}

// Quantum simulate integration
btnQuantumSim?.addEventListener('click', async () => {
  if (!monacoEditor) return;
  const src = monacoEditor.getValue();
  try {
    const res = await window.__TAURI__.invoke('aeonmi_quantum_simulate', { source: src });
    const obj = JSON.parse(res);
    const summary = `qubits: ${obj.qubits}\nops: ${JSON.stringify(obj.ops)}\nstate: ${obj.statevector.slice(0,16).map(v=>v.toFixed(3)).join(', ')}${obj.statevector.length>16?' ...':''}`;
    term.writeln('\x1b[36m[Quantum Sim]\x1b[0m');
    term.writeln(summary);
    if (obj.histogram && Object.keys(obj.histogram).length) {
      term.writeln('histogram:');
      for (const k of Object.keys(obj.histogram)) { term.writeln(`  ${k}: ${obj.histogram[k]}`); }
    }
  } catch(e){ term.writeln(`[Quantum sim error: ${e}]`); }
});

// AI streaming integration
btnAIStream?.addEventListener('click', async () => {
  const prompt = aiPrompt?.value || ''; if (!prompt.trim()) { alert('Enter prompt'); return; }
  addPromptHistory(prompt);
  startAIStream(prompt);
});

// --- AI Prompt History ---
function loadPromptHistory() {
  try { const raw = localStorage.getItem('aeonmi.aiHistory'); if (!raw) return []; return JSON.parse(raw); } catch { return []; }
}
function savePromptHistory(list) { try { localStorage.setItem('aeonmi.aiHistory', JSON.stringify(list.slice(0,25))); } catch {} }
function getHistoryLimit() { const v = parseInt(localStorage.getItem('aeonmi.aiHistoryLimit')||'25', 10); return isNaN(v)?25:Math.min(Math.max(v,1),200); }
function savePromptHistoryLimited(list) { const lim = getHistoryLimit(); savePromptHistory(list.slice(0, lim)); }
function rebuildHistorySelect() {
  if (!aiHistory) return; const list = loadPromptHistory(); aiHistory.innerHTML = '<option value="">(history)</option>' + list.map(h=>`<option value="${h.replace(/"/g,'&quot;')}">${h.length>40?h.slice(0,37)+'...':h}</option>`).join('');
}
function addPromptHistory(p) {
  let list = loadPromptHistory();
  list = [p, ...list.filter(x=>x!==p)];
  savePromptHistoryLimited(list); rebuildHistorySelect();
}
clearAIHistBtn?.addEventListener('click', () => { localStorage.removeItem('aeonmi.aiHistory'); rebuildHistorySelect(); });
// history limit input
if (aiHistLimitInput) {
  aiHistLimitInput.value = getHistoryLimit();
  aiHistLimitInput.addEventListener('change', () => {
    const val = parseInt(aiHistLimitInput.value,10); if (!isNaN(val)) { localStorage.setItem('aeonmi.aiHistoryLimit', String(Math.min(Math.max(val,1),200))); }
    // trim existing history
    let list = loadPromptHistory(); savePromptHistoryLimited(list); rebuildHistorySelect();
  });
}
aiHistory?.addEventListener('change', () => { if (aiHistory.value) { aiPrompt.value = aiHistory.value; } });
rebuildHistorySelect();

// --- Auto Quantum Simulation ---
autoSimEnabled = localStorage.getItem('aeonmi.autoQuantumSim') === '1';
if (autoQuantumSim) { autoQuantumSim.checked = autoSimEnabled; autoQuantumSim.addEventListener('change', () => { autoSimEnabled = autoQuantumSim.checked; localStorage.setItem('aeonmi.autoQuantumSim', autoSimEnabled ? '1':'0'); if (autoSimEnabled) scheduleQuantumSim(); }); }

function scheduleQuantumSim() {
  if (!autoSimEnabled) return; if (quantumSimTimer) clearTimeout(quantumSimTimer);
  quantumSimTimer = setTimeout(runAutoQuantumSim, 900);
}
async function runAutoQuantumSim() {
  if (!autoSimEnabled) return;
  if (!monacoEditor) return; const src = monacoEditor.getValue();
  try {
    const res = await window.__TAURI__.invoke('aeonmi_quantum_simulate', { source: src });
    const obj = JSON.parse(res);
    term.writeln('\x1b[35m[Auto Quantum]\x1b[0m qubits='+obj.qubits+' ops='+JSON.stringify(obj.ops));
  } catch(e) { term.writeln('[Auto quantum error: '+e+']'); }
}

// Hook into editor changes (after monacoEditor created) - we retry until available
const waitForEditor = setInterval(() => { if (window.monacoEditor) { clearInterval(waitForEditor); window.monacoEditor.onDidChangeModelContent(()=>{ scheduleQuantumSim(); }); } }, 200);

// --- Auto AI Streaming ---
autoAIEnabled = localStorage.getItem('aeonmi.autoAI') === '1';
if (autoAICheck) { autoAICheck.checked = autoAIEnabled; autoAICheck.addEventListener('change', () => { autoAIEnabled = autoAICheck.checked; localStorage.setItem('aeonmi.autoAI', autoAIEnabled?'1':'0'); if (autoAIEnabled) scheduleAIStream(); }); }
aiPrompt?.addEventListener('input', () => { if (autoAIEnabled) scheduleAIStream(); });

function scheduleAIStream() { if (aiDebounceTimer) clearTimeout(aiDebounceTimer); aiDebounceTimer = setTimeout(()=> { const text = aiPrompt?.value || ''; if (text.trim()) { startAIStream(text); } }, 1100); }

async function startAIStream(prompt) {
  if (!prompt.trim()) return;
  addPromptHistory(prompt);
  currentAIStreamId += 1; const streamId = currentAIStreamId;
  if (currentAIUnlisten) { try { currentAIUnlisten(); } catch {} currentAIUnlisten = null; }
  aiStreamOut.style.display='block'; aiStreamOut.textContent='';
  currentAIUnlisten = await window.__TAURI__.event.listen('ai-stream', e => {
    if (streamId !== currentAIStreamId) return; // stale
    const payload = e.payload;
    try {
      if (payload.done) { aiStreamOut.textContent += '\n[done]'; if (currentAIUnlisten) { currentAIUnlisten(); currentAIUnlisten=null; } return; }
      if (payload.chunk) { aiStreamOut.textContent += payload.chunk; aiStreamOut.scrollTop = aiStreamOut.scrollHeight; }
      if (payload.error) { aiStreamOut.textContent += `\n[error: ${payload.error}]`; }
    } catch(err) { console.error(err); }
  });
  try { await window.__TAURI__.invoke('ai_chat_stream', { provider: null, prompt }); } catch(e){ aiStreamOut.textContent += `\nInvoke error: ${e}`; }
}

// --- Theme toggle ---
function applyTheme(theme) {
  const body = document.body; body.classList.remove('theme-dark','theme-light');
  const t = (theme==='light')?'theme-light':'theme-dark'; body.classList.add(t);
  localStorage.setItem('aeonmi.theme', t);
  if (window.monaco) {
    if (t==='theme-light') { monaco.editor.setTheme('vs'); }
    else { monaco.editor.setTheme('vs-dark'); }
  }
}
const storedTheme = localStorage.getItem('aeonmi.theme') || 'theme-dark';
applyTheme(storedTheme.includes('light')?'light':'dark');
themeToggle?.addEventListener('click', () => {
  const curr = localStorage.getItem('aeonmi.theme') || 'theme-dark';
  const next = curr==='theme-dark' ? 'light' : 'dark';
  applyTheme(next);
});

// Minimal inline theming adjustments (body classes) – rely on existing dark defaults
const styleEl = document.getElementById('dynamicThemeStyle') || (()=>{ const s=document.createElement('style'); s.id='dynamicThemeStyle'; document.head.appendChild(s); return s; })();
styleEl.textContent = `body.theme-light { background:#f7f7f7; color:#222; } body.theme-light #outlinePanel { background:#fafafa; border-color:#ccc; } body.theme-light #aiStreamOut { background:#eef; color:#123; } body.theme-light #xterm { filter: invert(90%) hue-rotate(180deg); }`;

// --- Settings Panel / Pref Sync ---
async function loadPrefsBackend() {
  try { const raw = await window.__TAURI__.invoke('prefs_get_all'); return raw; } catch(e){ return {}; }
}
async function ensurePrefsInitialized() {
  const prefs = await loadPrefsBackend();
  // migrate local storage values if backend empty for key
  const migrations = [
    ['theme', localStorage.getItem('aeonmi.theme')||'theme-dark'],
    ['aiHistoryLimit', localStorage.getItem('aeonmi.aiHistoryLimit')||'25'],
    ['autoAI', localStorage.getItem('aeonmi.autoAI')=='1' ? '1':'0'],
    ['autoQuantumSim', localStorage.getItem('aeonmi.autoQuantumSim')=='1' ? '1':'0']
  ];
  for (const [k,v] of migrations) { if (!(k in prefs)) { try { await window.__TAURI__.invoke('prefs_set', { key: k, value: v }); } catch {} } }
}
function applyPrefsToUI(prefs) {
  if (prefTheme) prefTheme.value = (prefs.theme||'theme-dark').includes('light') ? 'light':'dark';
  if (prefHistLimit) prefHistLimit.value = prefs.aiHistoryLimit || getHistoryLimit();
  if (prefAutoAI) prefAutoAI.checked = prefs.autoAI === '1';
  if (prefAutoQuantum) prefAutoQuantum.checked = prefs.autoQuantumSim === '1';
}
async function refreshPrefsPanel() { const prefs = await loadPrefsBackend(); applyPrefsToUI(prefs); }
openSettings?.addEventListener('click', async () => { await refreshPrefsPanel(); settingsPanel.style.display='block'; });
closeSettings?.addEventListener('click', ()=> settingsPanel.style.display='none');
savePrefsBtn?.addEventListener('click', async () => {
  const updates = [
    ['theme', prefTheme.value==='light'?'theme-light':'theme-dark'],
    ['aiHistoryLimit', String(prefHistLimit.value||'25')],
    ['autoAI', prefAutoAI.checked?'1':'0'],
    ['autoQuantumSim', prefAutoQuantum.checked?'1':'0']
  ];
  for (const [k,v] of updates) { try { await window.__TAURI__.invoke('prefs_set', { key: k, value: v }); } catch(e){ console.error('pref set fail', k, e); } }
  // apply locally
  localStorage.setItem('aeonmi.theme', updates[0][1]);
  localStorage.setItem('aeonmi.aiHistoryLimit', updates[1][1]);
  localStorage.setItem('aeonmi.autoAI', updates[2][1]);
  localStorage.setItem('aeonmi.autoQuantumSim', updates[3][1]);
  applyTheme(prefTheme.value);
  aiHistLimitInput.value = updates[1][1];
  autoAICheck.checked = prefAutoAI.checked; autoAIEnabled = prefAutoAI.checked;
  if (prefAutoAI.checked) scheduleAIStream();
  if (prefAutoQuantum.checked) { autoSimEnabled = true; scheduleQuantumSim(); } else { autoSimEnabled = false; }
  settingsPanel.style.display='none';
});

// Periodic background sync (detect external window changes)
setInterval(async () => {
  if (settingsPanel.style.display !== 'none') return; // don't override while open
  const prefs = await loadPrefsBackend();
  // theme sync
  if (prefs.theme && !document.body.classList.contains(prefs.theme)) {
    applyTheme(prefs.theme.includes('light')?'light':'dark');
  }
}, 5000);

ensurePrefsInitialized();
// --- Type Checking ---
typeCheckBtn?.addEventListener('click', async () => {
  if (!monacoEditor) return; const src = monacoEditor.getValue();
  try { const raw = await window.__TAURI__.invoke('aeonmi_types', { source: src }); const diags = JSON.parse(raw); applyTypeMarkers(diags); term.writeln(`[Type check: ${diags.length} issues]`); } catch(e){ term.writeln('[Type check failed: '+e+']'); }
});

function applyTypeMarkers(diags) {
  if (!window.monaco || !monacoEditor) return;
  const model = monacoEditor.getModel(); if (!model) return;
  const existing = monaco.editor.getModelMarkers({ resource: model.uri }) || [];
  const others = existing.filter(m => m.owner !== 'type');
  const typeMarkers = diags.map(d => ({ startLineNumber: d.line||1, startColumn: (d.column||1), endLineNumber: d.line||1, endColumn: (d.column||1)+1, message: d.message, severity: monaco.MarkerSeverity.Info, owner: 'type' }));
  monaco.editor.setModelMarkers(model, 'type', typeMarkers);
}

// Placeholder quantum circuit visualizer hook; future implementation will parse ops and render.
async function updateQuantumCircuit(src) {
  if (!window.__TAURI__?.invoke || !quantumCircuitPanel) return;
  // Throttle identical content to avoid repeated backend calls
  const h = src.length + ':' + (src.charCodeAt(0)||0) + ':' + (src.charCodeAt(src.length-1)||0);
  if (h === lastCircuitHash) return; lastCircuitHash = h;
  try {
    const raw = await window.__TAURI__.invoke('aeonmi_quantum_circuit', { source: src });
    const data = typeof raw === 'string' ? JSON.parse(raw) : raw;
    if (!data || typeof data !== 'object') { quantumCircuitPanel.style.display='none'; return; }
    const gates = Array.isArray(data.gates) ? data.gates : [];
    if (!gates.length) { quantumCircuitPanel.style.display='none'; return; }
    const qcount = data.qubits || Math.max(0, ...gates.map(g=> (Array.isArray(g.qubits)?Math.max(...g.qubits):-1))) + 1;
    const lines = Array.from({ length: qcount }, (_,i)=>({ label:`q${i}: `, cells:[] }));
    // Assign each gate to a timeline column sequentially
    gates.forEach((g, idx) => {
      const name = (g.name||'?').toUpperCase();
      const qs = Array.isArray(g.qubits)? g.qubits : [];
      for (let qi=0; qi<qcount; qi++) {
        if (qs.includes(qi)) lines[qi].cells[idx] = name.padEnd(4,' ');
        else if (lines[qi].cells[idx] === undefined) lines[qi].cells[idx] = '│   ';
      }
    });
    // Add multi-qubit connectors: for any gate spanning >1 qubit, replace cells with brackets and draw '=' between qubits vertically aligned
    gates.forEach((g, idx) => {
      const qs = Array.isArray(g.qubits)? [...g.qubits].sort((a,b)=>a-b) : [];
      if (qs.length > 1) {
        const top = qs[0]; const bottom = qs[qs.length-1];
        for (let qi = top; qi <= bottom; qi++) {
          if (!lines[qi].cells[idx] || lines[qi].cells[idx].startsWith('│')) lines[qi].cells[idx] = '║   ';
        }
        // Mark endpoints
        lines[top].cells[idx] = '[' + (lines[top].cells[idx].trim()[0]||'M') + ']  '.slice(0,4);
        lines[bottom].cells[idx] = '[' + (lines[bottom].cells[idx].trim()[0]||'M') + ']  '.slice(0,4);
      }
    });
    const header = '     ' + gates.map((_,i)=> String(i).padEnd(4,' ')).join('');
    const body = lines.map(l => l.label + l.cells.map(c=> c||'    ').join('')).join('\n');
    const legend = '\nLegend: [X] gate, ║ multi-qubit span';
    circuitText.textContent = header + '\n' + body + legend;
    drawCircuitCanvas(gates, qcount);
    quantumCircuitPanel.style.display='block';
  } catch(e) {
    if (quantumCircuitPanel) quantumCircuitPanel.style.display='none';
  }
}
function drawCircuitCanvas(gates, qcount){ if(!circuitCanvas) return; const ctx = circuitCanvas.getContext('2d'); if(!ctx) return; circuitCanvas.style.display='block'; const W = circuitCanvas.width = circuitCanvas.clientWidth; const H = circuitCanvas.height = circuitCanvas.clientHeight; ctx.clearRect(0,0,W,H); const margin=20; const rowH=(H-2*margin)/(qcount||1); const colW = Math.max(40, (W-2*margin)/Math.max(1,gates.length)); ctx.strokeStyle='#377'; ctx.lineWidth=1; ctx.font='11px monospace'; ctx.fillStyle='#9cf'; for(let q=0;q<qcount;q++){ const y=margin+q*rowH; ctx.beginPath(); ctx.moveTo(margin,y); ctx.lineTo(W-margin,y); ctx.stroke(); ctx.fillText('q'+q,4,y+4); }
  gates.forEach((g,i)=>{ const x = margin + i*colW + colW/2; const qs = Array.isArray(g.qubits)? g.qubits.slice().sort((a,b)=>a-b):[]; if(!qs.length) return; if(qs.length===1){ const y = margin + qs[0]*rowH; ctx.fillStyle='#2a8'; ctx.beginPath(); ctx.arc(x,y,10,0,Math.PI*2); ctx.fill(); ctx.fillStyle='#fff'; ctx.fillText((g.name||'?').substr(0,3).toUpperCase(), x-12, y+4); } else { const top = margin + qs[0]*rowH; const bottom = margin + qs[qs.length-1]*rowH; ctx.strokeStyle='#a83'; ctx.lineWidth=3; ctx.beginPath(); ctx.moveTo(x, top); ctx.lineTo(x, bottom); ctx.stroke(); ctx.fillStyle='#a83'; ctx.beginPath(); ctx.arc(x, top, 9,0,Math.PI*2); ctx.arc(x, bottom, 9,0,Math.PI*2); ctx.fill(); ctx.fillStyle='#000'; ctx.fillText((g.name||'?').substr(0,2).toUpperCase(), x-10, top+4); ctx.fillText((g.name||'?').substr(0,2).toUpperCase(), x-10, bottom+4); }
  }); }

// --- Code Actions ---
async function refreshCodeActionsCache() {
  if (!monacoEditor) return; const src = monacoEditor.getValue();
  try { const raw = await window.__TAURI__.invoke('aeonmi_code_actions', { source: src }); cachedActions = JSON.parse(raw); } catch(e){ cachedActions = []; }
}
codeActionsBtn?.addEventListener('click', async (ev) => {
  await refreshCodeActionsCache();
  if (!monacoEditor) return;
  const pos = monacoEditor.getPosition(); if (!pos) return;
  const line = pos.lineNumber; const column = pos.column;
  const near = cachedActions.filter(a => Math.abs(a.line - line) < 3);
  if (!near.length) { term.writeln('[No actions]'); return; }
  showActionsMenu(ev.pageX, ev.pageY, near, line, column);
});

function showActionsMenu(x,y, actions, line, column) {
  let menu = document.getElementById('actionsMenu'); if (menu) menu.remove();
  menu = document.createElement('div'); menu.id='actionsMenu'; document.body.appendChild(menu);
  Object.assign(menu.style, { position:'absolute', top:y+'px', left:x+'px', background:'#222', color:'#eee', border:'1px solid #555', padding:'4px 0', fontSize:'12px', zIndex:2000, minWidth:'200px' });
  actions.forEach(a=> { const el = document.createElement('div'); el.textContent = a.title; Object.assign(el.style,{padding:'4px 10px', cursor:'pointer'}); el.addEventListener('mouseenter',()=>el.style.background='#444'); el.addEventListener('mouseleave',()=>el.style.background='transparent'); el.addEventListener('click',()=>{ applyAction(a, line, column); menu.remove(); }); menu.appendChild(el); });
  document.addEventListener('click', ()=> menu.remove(), { once: true });
}
function applyAction(action, line, column) {
  if (!monacoEditor) return;
  switch(action.kind) {
    case 'rename':
      renameAt(line, column); break;
    case 'extractFunction':
      extractFunctionAt(line, column); break;
    case 'introduceVariable':
      introduceVariableAt(line, column); break;
    default:
      term.writeln(`[Unhandled action ${action.kind}]`);
  }
}
async function renameAt(line, column) {
  const model = monacoEditor.getModel(); if (!model) return;
  const word = model.getWordAtPosition({ lineNumber: line, column: column }) || model.getWordAtPosition({ lineNumber: line, column: column-1 });
  if (!word) { term.writeln('[No symbol to rename]'); return; }
  const newName = prompt('Rename to:', word.word); if (!newName || newName===word.word) return;
  const src = model.getValue();
  if (window.__TAURI__?.invoke) {
    try {
      const updated = await window.__TAURI__.invoke('aeonmi_rename_symbol', { source: src, line, column, newName });
      if (typeof updated === 'string') { model.setValue(updated); runDiagnostics(); fetchSymbols(); return; }
    } catch(e) { /* fallback below */ }
  }
  // Fallback client-side naive rename
  const re = new RegExp(`\\b${word.word.replace(/[-/\\^$*+?.()|[\]{}]/g,'\\$&')}\\b`, 'g');
  model.setValue(src.replace(re, newName));
  runDiagnostics(); fetchSymbols();
}
function extractFunctionAt(line, column) {
  // naive: extract current line into function
  const model = monacoEditor.getModel(); if (!model) return;
  const text = model.getLineContent(line).trim(); if (!text) { term.writeln('[Empty line]'); return; }
  const name = prompt('Function name:', 'extracted_part'); if (!name) return;
  model.applyEdits([
    { range: new monaco.Range(line,1,line, model.getLineMaxColumn(line)), text: `${name}();` },
    { range: new monaco.Range(1,1,1,1), text: `fn ${name}() {\n  ${text}\n}\n\n` }
  ]);
  runDiagnostics(); fetchSymbols();
}
function introduceVariableAt(line, column) {
  const model = monacoEditor.getModel(); if (!model) return;
  const sel = monacoEditor.getSelection();
  if (sel && !sel.isEmpty()) {
    const text = model.getValueInRange(sel);
    const varName = prompt('Variable name:', 'temp'); if (!varName) return;
    model.pushEditOperations([], [ { range: sel, text: varName }, { range: new monaco.Range(line,1,line,1), text: `let ${varName} = ${text};\n` } ], ()=>null);
  } else { term.writeln('[Select expression to introduce variable]'); }
  runDiagnostics(); fetchSymbols();
}

function registerHover() {
  if (!window.monaco) return;
  monaco.languages.registerHoverProvider('aeonmi', { provideHover(model, position) { const sym = currentSymbols.find(s => position.lineNumber === s.line && position.column >= s.column && position.column <= (s.end_column || s.column+1)); if (!sym) return null; const kindLabel = sym.kind.charAt(0).toUpperCase()+sym.kind.slice(1); return { range: new monaco.Range(sym.line, sym.column, sym.end_line||sym.line, sym.end_column||sym.column+1), contents: [{ value: `**${kindLabel}** ${sym.name}` }] }; } });
  monaco.languages.registerDefinitionProvider('aeonmi', { provideDefinition(model, position) { const word = model.getWordAtPosition(position); if (!word) return null; const target = currentSymbols.find(s => s.name === word.word); if (!target) return null; return [{ uri: model.uri, range: new monaco.Range(target.line, target.column, target.end_line||target.line, target.end_column||target.column+1) }]; } });
  window.addEventListener('keydown', e => { if (e.key === 'F12' && monacoEditor && document.activeElement.closest('#editorMonaco')) { const pos = monacoEditor.getPosition(); const model = monacoEditor.getModel(); const word = model.getWordAtPosition(pos); if (word) { const target = currentSymbols.find(s => s.name === word.word); if (target) { monacoEditor.revealPositionInCenter({ lineNumber: target.line, column: target.column }); monacoEditor.setSelection({ startLineNumber: target.line, startColumn: target.column, endLineNumber: target.end_line||target.line, endColumn: target.end_column||target.column+1 }); e.preventDefault(); } } } });
}

