const express = require('express');
const bodyParser = require('body-parser');
const http = require('http');
const WebSocket = require('ws');
const pty = require('node-pty');
const path = require('path');

const app = express();
app.use(bodyParser.json());
app.use(express.static(path.join(__dirname, 'static')));

const server = http.createServer(app);
const wss = new WebSocket.Server({ server });

wss.on('connection', function connection(ws) {
  // Try to spawn the compiled Aeonmi binary if present, otherwise fallback to a shell
  const repoRoot = process.cwd();
  // Prefer the user-requested 'Aeonmi Shard' executable name, fall back to common names.
  const candidates = [];
  if (process.platform === 'win32') {
    candidates.push('Aeonmi Shard.exe', 'Aeonmi.exe', 'aeonmi_project.exe');
  } else {
    candidates.push('Aeonmi Shard', 'Aeonmi', 'aeonmi_project');
  }
  let spawnCmd = null;
  for (const name of candidates) {
    const pth = path.join(repoRoot, 'target', 'debug', name);
    if (require('fs').existsSync(pth)) {
      spawnCmd = pth;
      break;
    }
  }
  if (!spawnCmd) {
    // fallback to a shell if no binary found
    spawnCmd = process.platform === 'win32' ? 'powershell.exe' : 'bash';
  }

  const p = pty.spawn(spawnCmd, spawnArgs, {
    name: 'xterm-color',
    cols: 80,
    rows: 24,
    cwd: process.cwd(),
    env: process.env
  });

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
  const repoRoot = process.cwd();
  const fs = require('fs');
  const candidates = [];
  if (process.platform === 'win32') {
    candidates.push('Aeonmi Shard.exe', 'Aeonmi.exe', 'aeonmi_project.exe');
  } else {
    candidates.push('Aeonmi Shard', 'Aeonmi', 'aeonmi_project');
  }
  let binPath = null;
  for (const name of candidates) {
    const pth = path.join(repoRoot, 'target', 'debug', name);
    if (fs.existsSync(pth)) {
      binPath = pth;
      break;
    }
  }

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

server.listen(4000, () => {
  console.log('GUI prototype server started at http://localhost:4000');
});
