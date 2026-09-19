---
name: opencode-gemini-auth
source: npm
description: Gemini authentication for OpenCode
---

# OpenCode Gemini Auth Plugin

Provides Gemini API authentication for OpenCode.

## Installation

```bash
npm install -g opencode-gemini-auth
```

## Configuration

Add to `opencode.json`:

```json
{
  "plugin": ["opencode-gemini-auth"],
  "provider": {
    "gemini": { "options": { "apiKey": "{env:GEMINI_API_KEY}" } }
  }
}
```

## Environment

Set `GEMINI_API_KEY` in your shell or `.env` file.

## Usage

Once configured, select `gemini/` models in OpenCode model picker.