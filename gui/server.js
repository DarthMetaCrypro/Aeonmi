const express = require('express');
const bodyParser = require('body-parser');
const http = require('http');
const WebSocket = require('ws');
let pty = null;
try {
  pty = require('node-pty');
} catch (e) {
  console.warn('[gui] node-pty not available, falling back to simple echo shell emulation');
}
const path = require('path');

const app = express();
app.use(bodyParser.json());
app.use(express.static(path.join(__dirname, 'static')));

const server = http.createServer(app);
const wss = new WebSocket.Server({ server });

function detectBinary() {
  const repoRoot = process.cwd();
  const fs = require('fs');
  const candidates = [];
  if (process.platform === 'win32') {
    // Include canonical lowercase names; previous list missed some
    candidates.push('aeonmi.exe', 'Aeonmi.exe', 'aeonmi_project.exe');
  } else {
    candidates.push('aeonmi', 'Aeonmi', 'aeonmi_project');
  }
  for (const name of candidates) {
    const pth = path.join(repoRoot, 'target', 'debug', name);
    if (fs.existsSync(pth)) return pth;
  }
  return null;
}

wss.on('connection', function connection(ws) {
  let spawnCmd = detectBinary();
  if (!pty) {
    ws.send(JSON.stringify({ type: 'output', data: '[pseudo-terminal]\r\nnode-pty not installed; limited echo mode.\r\n' }));
    ws.on('message', msg => {
      try { const m = JSON.parse(msg); if (m.type === 'input') { ws.send(JSON.stringify({ type: 'output', data: m.data })); } } catch {}
    });
    return;
  }
  if (!spawnCmd) spawnCmd = process.platform === 'win32' ? 'powershell.exe' : 'bash';
  const spawnArgs = [];
  let p;
  try {
    p = pty.spawn(spawnCmd, spawnArgs, { name: 'xterm-color', cols: 80, rows: 24, cwd: process.cwd(), env: process.env });
  } catch (e) {
    ws.send(JSON.stringify({ type: 'output', data: `Failed to spawn PTY: ${e}\r\n` }));
    return;
  }

  p.on('data', function(data) {
    ws.send(JSON.stringify({ type: 'output', data }));
  });

  ws.on('message', function incoming(message) {
    try {
      const m = JSON.parse(message);
      if (m.type === 'input') {
        p.write(m.data);
      } else if (m.type === 'resize') {
        p.resize(m.cols, m.rows);
      }
    } catch (e) {
      console.error('ws parse error', e);
    }
  });

  ws.on('close', () => p.kill());
});

app.get('/api/open', (req, res) => {
  const f = req.query.file;
  if (!f) return res.status(400).send('missing file');
  try {
    const content = require('fs').readFileSync(path.resolve(f), 'utf8');
    res.json({ ok: true, content });
  } catch (err) {
    res.status(500).json({ ok: false, error: String(err) });
  }
});

app.post('/api/save', (req, res) => {
  const { file, content } = req.body;
  if (!file) return res.status(400).send('missing file');
  try {
    require('fs').writeFileSync(path.resolve(file), content, 'utf8');
    res.json({ ok: true });
  } catch (err) {
    res.status(500).json({ ok: false, error: String(err) });
  }
});

// Spawn a detached native terminal running the Aeonmi Shard (or fallback shell)
app.post('/api/detach', (req, res) => {
  const binPath = detectBinary();

  // Command to run in terminal
  let cmd = null;
  let args = [];
  if (binPath) {
    cmd = binPath;
  } else {
    cmd = process.platform === 'win32' ? 'powershell.exe' : '/bin/bash';
  }

  try {
    const child_process = require('child_process');
    if (process.platform === 'win32') {
      // Use 'start' to spawn a new window running cmd /k <command>
      const finalCmd = binPath ? `\"${binPath}\"` : cmd;
      child_process.exec(`start cmd /k ${finalCmd}`);
    } else {
      // Try common terminal emulators
      const terminals = ['x-terminal-emulator', 'gnome-terminal', 'konsole', 'alacritty', 'kitty', 'xterm'];
      let launched = false;
      for (const termName of terminals) {
        try {
          // -e for many emulators, alacritty/kitty accept -e too
          child_process.spawn(termName, ['-e', cmd], { detached: true, stdio: 'ignore' }).unref();
          launched = true;
          break;
        } catch (e) {
          // try next
        }
      }
      if (!launched) {
        // fallback: spawn command detached (may not show a window)
        child_process.spawn(cmd, args, { detached: true, stdio: 'ignore' }).unref();
      }
    }
    res.json({ ok: true });
  } catch (err) {
    res.status(500).json({ ok: false, error: String(err) });
  }
});

app.get('/health', (_req, res) => res.json({ ok: true, bin: !!detectBinary() }));

server.listen(4000, () => {
  console.log('[gui] server at http://localhost:4000');
  const bin = detectBinary();
  if (bin) console.log(`[gui] found binary: ${bin}`); else console.log('[gui] no binary found yet; will fall back to shell');
});
