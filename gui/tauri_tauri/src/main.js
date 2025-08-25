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
const diagnosticsPanel = document.getElementById('diagnostics');
const filepathInput = document.getElementById('filepath');
// AI chat elements
const aiProviderSelect = document.getElementById('ai_provider');
const aiPrompt = document.getElementById('ai_prompt');
const aiSendBtn = document.getElementById('ai_send');
const aiOutput = document.getElementById('ai_output');
const aiListBtn = document.getElementById('ai_list');
const aiStreamBtn = document.getElementById('ai_stream');
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
