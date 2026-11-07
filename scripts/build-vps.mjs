#!/usr/bin/env node
/**
 * Cross-platform runner for `build:vps` that sets NITRO_PRESET=node-server
 * before invoking `with-app-env.mjs vite build`.
 * Compatible with Windows (cmd/PowerShell) and POSIX (Linux/macOS).
 */
import { spawn } from "node:child_process";

process.env.NITRO_PRESET = "node-server";

const isWin = process.platform === "win32";
const child = spawn("node", ["scripts/with-app-env.mjs", "vite", "build"], {
  stdio: "inherit",
  shell: isWin,
  env: process.env,
});

child.on("exit", (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal);
    return;
  }
  process.exit(code ?? 0);
});
