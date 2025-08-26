const { invoke } = window.__TAURI__.tauri;
const { Terminal } = window;
const { FitAddon } = window;

const startBtn = document.getElementById('start');
const detachBtn = document.getElementById('detach');
const status = document.getElementById('status');
const loadBtn = document.getElementById('load');
const saveBtn = document.getElementById('save');
const compileAiBtn = document.getElementById('compile_ai');
const compileJsBtn = document.getElementById('compile_js');
const runJsBtn = document.getElementById('run_js');
const circuitShowBtn = document.getElementById('circuit_show');
const circuitExportBtn = document.getElementById('circuit_export');
const codeActionsBtn = document.getElementById('code_actions_btn');
const circuitDownloadBtn = document.getElementById('circuit_download');
const circuitPanel = document.getElementById('circuit_panel');
const diagnosticsPanel = document.getElementById('diagnostics');
const filepathInput = document.getElementById('filepath');
// AI chat elements
const aiProviderSelect = document.getElementById('ai_provider');
const aiPrompt = document.getElementById('ai_prompt');
const aiSendBtn = document.getElementById('ai_send');
const aiOutput = document.getElementById('ai_output');
const aiListBtn = document.getElementById('ai_list');
const aiStreamBtn = document.getElementById('ai_stream');
const aiSetKeyBtn = document.getElementById('ai_set_key');
const aiShowKeyBtn = document.getElementById('ai_show_key');
const aiDeleteKeyBtn = document.getElementById('ai_delete_key');
let monacoEditor = null;
let monacoInstance = null;

const termEl = document.getElementById('terminal');
const term = new Terminal({ cursorBlink: true });
const fit = new FitAddon.FitAddon();
term.loadAddon(fit);
term.open(termEl);
fit.fit();

let ws = null;
let autosaveTimer = null;
const AUTOSAVE_DELAY = 800; // ms debounce

async function connectToBridge() {
  status.textContent = 'starting...';
  const url = await invoke('launch_bridge');
  status.textContent = `bridge: ${url}`;
  const wsUrl = url.replace('http', 'ws') + '/pty';
  ws = new WebSocket(wsUrl);
  ws.binaryType = 'arraybuffer';
  ws.onopen = () => {
    status.textContent = 'connected';
  };
  ws.onmessage = (ev) => {
    const data = typeof ev.data === 'string' ? ev.data : new TextDecoder().decode(ev.data);
    term.write(data);
  };
  ws.onclose = () => {
    status.textContent = 'closed';
  };
  ws.onerror = (e) => {
    status.textContent = 'error';
    console.error(e);
  };

  term.onData((d) => {
    if (ws && ws.readyState === WebSocket.OPEN) {
      ws.send(d);
    }
  });

  // Resize events
  window.addEventListener('resize', () => {
    fit.fit();
    if (ws && ws.readyState === WebSocket.OPEN) {
      const cols = term.cols;
      const rows = term.rows;
      ws.send(JSON.stringify({ type: 'resize', cols, rows }));
    }
  });
}

startBtn.addEventListener('click', async () => {
  try {
    await connectToBridge();
  } catch (e) {
    alert('Failed to connect: ' + e);
  }
});

detachBtn.addEventListener('click', async () => {
  try {
    await invoke('launch_bridge');
    // The backend's /detach behavior is part of the bridge; here we just notify user
    status.textContent = 'detached';
  } catch (e) {
    alert('Detach failed: ' + e);
  }
});

function getSource() { return monacoEditor ? monacoEditor.getValue() : ''; }
function setSource(src) { if (monacoEditor) monacoEditor.setValue(src); }

// Monaco bootstrap
if (window.require) {
  window.require(['vs/editor/editor.main'], () => {
    monacoInstance = window.monaco;
    // Define simple Aeonmi language
    monacoInstance.languages.register({ id: 'aeonmi' });
    monacoInstance.languages.setMonarchTokensProvider('aeonmi', {
      tokenizer: {
        root: [
          [/\b(function|let|if|else|while|for|return|log|superpose|entangle|measure|qubit)\b/, 'keyword'],
          [/"([^"\\]|\\.)*"/, 'string'],
          [/\b[0-9]+(\.[0-9]+)?\b/, 'number'],
          [/𓀀|𓀁|𓀂|𓀃|𓀄|𓁀|𓁁|𓁉|𓁍|𓁄/, 'keyword'],
          [/\b[a-zA-Z_][a-zA-Z0-9_]*\b/, 'identifier'],
        ]
      }
    });
    monacoInstance.editor.defineTheme('aeonmi-theme', {
      base: 'vs-dark', inherit: true,
      rules: [
        { token: 'keyword', foreground: 'C586C0', fontStyle: 'bold' },
        { token: 'string', foreground: 'CE9178' },
        { token: 'number', foreground: 'B5CEA8' },
        { token: 'identifier', foreground: 'D4D4D4' }
      ],
      colors: { 'editor.background': '#1E1E1E' }
    });
    monacoEditor = monacoInstance.editor.create(document.getElementById('code_editor'), {
      value: '// Load or create an Aeonmi file...\n',
      language: 'aeonmi',
      theme: 'aeonmi-theme',
      automaticLayout: true,
      minimap: { enabled: false },
    });
    monacoEditor.onDidChangeModelContent(() => {
      if (autosaveTimer) clearTimeout(autosaveTimer);
      autosaveTimer = setTimeout(async () => {
        const path = filepathInput.value.trim();
        if (path) {
          try { await invoke('save_file', { path, contents: getSource() }); status.textContent = 'autosaved'; } catch {}
        }
      }, AUTOSAVE_DELAY);
    });
  });
}

loadBtn?.addEventListener('click', async () => {
  const path = filepathInput.value.trim() || 'examples/hello.ai';
  try {
    const data = await invoke('load_file', { path });
    setSource(data);
    status.textContent = 'loaded';
  } catch (e) { alert('Load failed: ' + e); }
});

saveBtn?.addEventListener('click', async () => {
  const path = filepathInput.value.trim() || 'untitled.ai';
  try {
    await invoke('save_file', { path, contents: getSource() });
    status.textContent = 'saved';
  } catch (e) { alert('Save failed: ' + e); }
});

async function doCompile(ai) {
  const path = filepathInput.value.trim() || 'untitled.ai';
  // auto-save before compile
  try { await invoke('save_file', { path, contents: getSource() }); } catch {}
  try {
    const respStr = await invoke('tauri_compile', { path, emit: ai ? 'ai' : 'js' });
    let resp = {};
    try { resp = JSON.parse(respStr); } catch {}
    status.textContent = resp.success ? 'compiled ✓' : 'compile errors';
    if (resp.diagnostics) {
      diagnosticsPanel.innerHTML = '<b>Diagnostics</b><br/>' + resp.diagnostics.map(d => {
        const msg = d.message || d.msg || JSON.stringify(d);
        return `<div style="color:${d.severity==='error'?'#ff5555':'#ffaa00'};">${(d.file? d.file+':':'')}${d.line?d.line:''}${d.col?':'+d.col:''} - ${msg}</div>`;
      }).join('');
      // Apply Monaco markers
      if (monacoInstance && monacoEditor) {
        const model = monacoEditor.getModel();
        const markers = resp.diagnostics.filter(d => d.line && d.col).map(d => ({
          startLineNumber: d.line,
          startColumn: d.col,
          endLineNumber: d.line,
          endColumn: d.col + (d.len || 1),
          message: d.message || d.msg || 'error',
          severity: monacoInstance.MarkerSeverity.Error
        }));
        monacoInstance.editor.setModelMarkers(model, 'aeonmi', markers);
      }
    } else {
      diagnosticsPanel.innerHTML = '<b>Diagnostics</b><br/>None';
      if (monacoInstance && monacoEditor) {
        monacoInstance.editor.setModelMarkers(monacoEditor.getModel(), 'aeonmi', []);
      }
    }
  } catch (e) { alert('Compile failed: ' + e); }
}

compileAiBtn?.addEventListener('click', () => doCompile(true));
compileJsBtn?.addEventListener('click', () => doCompile(false));
runJsBtn?.addEventListener('click', async () => {
  const path = filepathInput.value.trim() || 'untitled.ai';
  // Ensure fresh compile to JS
  try {
    await invoke('save_file', { path, contents: getSource() });
    await doCompile(false); // JS compile
    const runRespStr = await invoke('run_js', { path: 'gui_output.js' });
    let runResp = {}; try { runResp = JSON.parse(runRespStr); } catch {}
    const out = (runResp.stdout||'').replace(/</g,'&lt;');
    const err = (runResp.stderr||'').replace(/</g,'&lt;');
    term.write(`\r\n[ run exit=${runResp.exitCode} ]\r\n`);
    if (out) term.write(out + '\r\n');
    if (err) term.write('[stderr]\r\n' + err + '\r\n');
    status.textContent = 'run complete';
  } catch (e) { alert('Run failed: ' + e); }
});

// Populate provider list by shelling through existing CLI list (invokes cargo run).
async function refreshProviders() {
  aiProviderSelect.innerHTML = '';
  try {
    const list = await invoke('ai_list_providers');
    (list || []).forEach(p => { const opt = document.createElement('option'); opt.value = p; opt.textContent = p; aiProviderSelect.appendChild(opt); });
  // Load saved provider preference
  try { const prefs = await invoke('prefs_get_all'); if (prefs && prefs.ai_provider) { aiProviderSelect.value = prefs.ai_provider; } } catch {}
  } catch (e) {
    ['openai','perplexity','deepseek','copilot'].forEach(p => { const opt = document.createElement('option'); opt.value = p; opt.textContent = p; aiProviderSelect.appendChild(opt); });
  }
}
refreshProviders();

aiListBtn?.addEventListener('click', () => {
  refreshProviders();
  aiOutput.textContent = 'Providers refreshed.';
});

aiSendBtn?.addEventListener('click', async () => {
  const prov = aiProviderSelect.value;
  const prompt = aiPrompt.value.trim();
  if (!prompt) { return; }
  aiOutput.textContent = '[sending...]\n';
  try {
    const respStr = await invoke('ai_chat', { provider: prov, prompt, stream: false });
    let resp = {}; try { resp = JSON.parse(respStr); } catch {}
    const text = resp.output || respStr;
    aiOutput.textContent += text;
    term.write(`\r\n[ai:${prov}] ${text.replace(/\n/g,'\r\n')}\r\n`);
  } catch (e) {
    aiOutput.textContent += 'Error: ' + e;
    term.write(`\r\n[ai-error:${prov}] ${String(e)}\r\n`);
  }
});

aiStreamBtn?.addEventListener('click', async () => {
  const prov = aiProviderSelect.value;
  const prompt = aiPrompt.value.trim();
  if (!prompt) { return; }
  aiOutput.textContent = '[streaming...]\n';
  window.__TAURI__.event.listen('ai-stream', (event) => {
    const payload = event.payload || {};
    if (payload.chunk) { aiOutput.textContent += payload.chunk; term.write(payload.chunk.replace(/\n/g,'\r\n')); }
    if (payload.error) { aiOutput.textContent += '\n[error] ' + payload.error; }
    if (payload.done) { aiOutput.textContent += '\n[done]'; }
  });
  try { await invoke('ai_chat_stream', { provider: prov, prompt }); } catch (e) { aiOutput.textContent += 'Error: ' + e; }
});

aiProviderSelect?.addEventListener('change', async () => {
  const prov = aiProviderSelect.value;
  try { await invoke('ai_set_provider', { name: prov }); await invoke('prefs_set', { key: 'ai_provider', value: prov }); } catch(e){ console.warn('provider set failed', e); }
});

aiSetKeyBtn?.addEventListener('click', async () => {
  const prov = aiProviderSelect.value; if (!prov) return;
  const key = prompt(`Enter API key for ${prov}`); if (!key) return;
  try { await invoke('api_key_set', { provider: prov, key }); status.textContent = 'key saved'; } catch(e){ alert('Key save failed: '+e); }
});

aiShowKeyBtn?.addEventListener('click', async () => {
  const prov = aiProviderSelect.value; if (!prov) return;
  try { const keyOpt = await invoke('api_key_get', { provider: prov }); if (keyOpt) { alert(`Key(${prov}): ${'*'.repeat(Math.max(0,keyOpt.length-4))}${keyOpt.slice(-4)}`); } else { alert('No key stored'); } } catch(e){ alert('Show failed: '+e); }
});

aiDeleteKeyBtn?.addEventListener('click', async () => {
  const prov = aiProviderSelect.value; if (!prov) return;
  if (!confirm(`Delete key for ${prov}?`)) return;
  try { await invoke('api_key_delete', { provider: prov }); status.textContent='key deleted'; } catch(e){ alert('Delete failed: '+e); }
});

circuitShowBtn?.addEventListener('click', async () => {
  try {
    const circStr = await invoke('aeonmi_quantum_circuit', { source: getSource() });
    let circ = {}; try { circ = JSON.parse(circStr); } catch {}
    circuitPanel.innerHTML = '<b>Circuit</b><br/>' + (circ.gates||[]).map(g=>`${g.gate}(${(g.qubits||[]).join(',')})`).join('<br/>');
  } catch(e){ circuitPanel.innerHTML = 'Circuit error: '+e; }
});

circuitExportBtn?.addEventListener('click', async () => {
  try {
    const respStr = await invoke('aeonmi_quantum_circuit_export', { source: getSource() });
    let resp={}; try { resp=JSON.parse(respStr);} catch {}
    const jsonEsc = (resp.json||'').replace(/</g,'&lt;');
    const qasmEsc = (resp.pseudo_qasm||'').replace(/</g,'&lt;');
    circuitPanel.innerHTML = `<b>Circuit Export</b><br/><details open><summary>JSON</summary><pre style="white-space:pre-wrap;font-size:11px;">${jsonEsc}</pre></details><details><summary>Pseudo-QASM</summary><pre style="white-space:pre-wrap;font-size:11px;">${qasmEsc}</pre></details>`;
  } catch(e){ circuitPanel.innerHTML = 'Export error: '+e; }
});

codeActionsBtn?.addEventListener('click', async () => {
  try { const actsStr = await invoke('aeonmi_code_actions', { source: getSource() }); let acts=[]; try{acts=JSON.parse(actsStr);}catch{} circuitPanel.innerHTML = '<b>Code Actions</b><br/>' + acts.map(a=>`${a.kind}: ${a.title}`).join('<br/>'); } catch(e){ circuitPanel.innerHTML='Actions error: '+e; }
});

circuitDownloadBtn?.addEventListener('click', async () => {
  try { const respStr = await invoke('aeonmi_quantum_circuit_export', { source: getSource() }); let resp={}; try{resp=JSON.parse(respStr);}catch{}; const ts = Date.now(); const base = `circuit_${ts}`; const jsonPath = base+`.json`; const qasmPath = base+`.qasm.txt`; await invoke('save_file', { path: jsonPath, contents: resp.json||'' }); await invoke('save_file', { path: qasmPath, contents: resp.pseudo_qasm||'' }); status.textContent = `saved ${jsonPath} & ${qasmPath}`; } catch(e){ alert('Download failed: '+e); }
});
