const wsUrl = `ws://${location.host}`;
let ws;
const termDiv = document.getElementById('xterm');
let monacoEditor = null;
// Monaco initialize
require.config({ paths: { 'vs': 'https://cdn.jsdelivr.net/npm/monaco-editor@0.38.0/min/vs' } });
require(['vs/editor/editor.main'], function () {
  monacoEditor = monaco.editor.create(document.getElementById('editorMonaco'), {
    value: "// Aeonmi editor\n",
    language: 'javascript',
    automaticLayout: true,
    fontFamily: 'monospace',
    minimap: { enabled: false }
  });
});

const term = new Terminal({ cursorBlink: true });
term.open(termDiv);
term.focus();

function connectPty() {
  if (ws) { ws.close(); ws = null; term.writeln('[connection closed]'); return; }
  ws = new WebSocket(wsUrl);
  ws.binaryType = 'arraybuffer';
  ws.addEventListener('open', () => term.writeln('[connected to TUI]'));
  ws.addEventListener('message', (ev) => {
    try {
      const m = JSON.parse(ev.data);
      if (m.type === 'output') term.write(m.data);
    } catch (e) {
      term.write(ev.data);
    }
  });
  ws.addEventListener('close', () => term.writeln('\r\n[disconnected]\r\n'));

  term.onData(data => {
    if (ws && ws.readyState === WebSocket.OPEN) ws.send(JSON.stringify({ type: 'input', data }));
  });

  // handle resize
  function sendResize() {
    const cols = term.cols;
    const rows = term.rows;
    if (ws && ws.readyState === WebSocket.OPEN) ws.send(JSON.stringify({ type: 'resize', cols, rows }));
  }
  window.addEventListener('resize', sendResize);
}

document.getElementById('startTui').addEventListener('click', connectPty);
document.getElementById('openNative').addEventListener('click', async () => {
  const res = await fetch('/api/detach', { method: 'POST' });
  const j = await res.json();
  if (j.ok) alert('Launched native terminal'); else alert('Failed: ' + j.error);
});

// simple open/save
document.getElementById('openBtn').addEventListener('click', async () => {
  const file = document.getElementById('filePath').value;
  if (!file) return alert('enter file path');
  const res = await fetch(`/api/open?file=${encodeURIComponent(file)}`);
  const j = await res.json();
  if (j.ok) {
    if (monacoEditor) monacoEditor.setValue(j.content); else alert('editor not ready');
  } else alert(j.error);
});

document.getElementById('saveBtn').addEventListener('click', async () => {
  const file = document.getElementById('filePath').value;
  if (!file) return alert('enter file path');
  const content = monacoEditor ? monacoEditor.getValue() : '';
  const res = await fetch('/api/save', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ file, content }) });
  const j = await res.json();
  if (j.ok) alert('saved'); else alert(j.error);
});
