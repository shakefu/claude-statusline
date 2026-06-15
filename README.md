# 🚀 claude-statusline ✨

> _Because your terminal deserves to be ✨ beautiful ✨ too!_

🎉 **Welcome to the most blazingly fast™ status line renderer for Claude Code!** 🎉

Are you tired of your boring old bash script chugging along, wasting precious milliseconds that could be spent _innovating_? Say hello to `claude-statusline` — a lovingly handcrafted, artisanally compiled **Rust binary** that transforms raw JSON into a gorgeous ANSI-colored status bar at the speed of light! ⚡️

## 🌟 Features

- 🧠 **Smart Token Display** — See your context window usage at a glance with intuitive `k` and `m` suffixes!
- 🚦 **Color-Coded Warnings** — Cyan when you're chillin' 😎, yellow when things are heating up 🔥, and RED when you're living on the edge! 🚨
- 📂 **Directory Awareness** — Shows your repo as `owner/name` or falls back to the directory basename. It just _gets_ you!
- 🌿 **Git Branch Detection** — Always know which branch you're on without lifting a finger! Works automagically! 🪄
- 🐍 **Virtual Environment Support** — Python devs, we see you! Your venv name, right there in the status bar! 💚
- 🤖 **Model Display** — Know exactly which Claude model is powering your session! Knowledge is power! 💪

## 📦 Installation

```bash
cargo install --path .
```

That's it! No, seriously. That's the whole thing. _We believe in simplicity._ 🧘

## 🎨 What Does It Look Like?

Imagine this, but with ✨ colors ✨:

```
50.0k/200.0k acme/widgets (main) [Claude Sonnet 4]
```

- `50.0k/200.0k` → 🔵 Cyan (you've got plenty of room!)
- `acme/widgets` → 🔷 Blue (so you know where you are!)
- `(main)` → 🟢 Green (branching out!)
- `[Claude Sonnet 4]` → 🔵 Cyan (powered by the best! 🧠)

## 🔧 Usage

Pipe Claude Code's JSON status payload into stdin:

```bash
echo '{"context_window":{"total_input_tokens":50000,"context_window_size":200000},"workspace":{"repo":{"owner":"acme","name":"widgets"}},"model":{"display_name":"Claude Sonnet 4"}}' | claude-statusline
```

### 🎯 Token Color Thresholds

| Usage Level | Color | Vibe |
|---|---|---|
| < 60,000 tokens | 🟦 Cyan | Cool as a cucumber 🥒 |
| 60,000 – 99,999 | 🟨 Yellow | Getting toasty! 🌡️ |
| ≥ 100,000 tokens | 🟥 Red | DANGER ZONE 🚨🔥 |

## 🏗️ Architecture

Built with love using **spec-driven TDD** because we're _professionals_ here! 💼

```
src/
├── main.rs           # 🎬 The star of the show — stdin → parse → render → stdout
├── format_tokens.rs  # 🔢 Makes big numbers smol and cute (1500000 → "1.5m")
├── input.rs          # 📥 JSON parsing with graceful degradation™
├── segments.rs       # 🎨 Where the ANSI magic happens ✨
└── util.rs           # 🧰 Shared utilities (because DRY is a lifestyle choice)
```

## 🧪 Testing

We don't just _think_ it works. We **know** it works! 💯

```bash
cargo test
```

**73 tests** covering every single spec obligation! That's right — _seventy-three_! Each one a tiny guardian angel 👼 protecting your status line from harm.

- ✅ 61 unit tests across all modules
- ✅ 12 integration tests that pipe real JSON into the actual binary
- ✅ Boundary testing at every color threshold
- ✅ Segment ordering verification
- ✅ Edge case coverage that would make your QA team weep tears of joy 😭

## 📋 Spec Compliance

Every behavior is driven by `statusline.yass.yaml` — the single source of truth! 📜

We don't just follow the spec. We _embody_ the spec. We _are_ the spec. 🧘‍♂️

| Spec | Status | Notes |
|---|---|---|
| FormatTokens | ✅ Implemented | Precision to one decimal place! |
| Input | ✅ Implemented | Gracefully handles empty/malformed JSON! |
| Input.ContextWindow | ✅ Implemented | Both-or-neither token counts! |
| Input.Workspace | ✅ Implemented | Smart owner/name fallback! |
| Input.Model | ✅ Implemented | Even filters empty strings! |
| TokenSegment | ✅ Implemented | Three-tier color system! |
| DirectorySegment | ✅ Implemented | Always beautifully blue! |
| GitBranchSegment | ✅ Implemented | With fsmonitor disabled for speed! |
| VenvSegment | ✅ Implemented | Python-friendly! |
| ModelSegment | ✅ Implemented | Know your model! |
| Output | ✅ Implemented | No trailing newline! Exit 0! |

## 🤝 Contributing

This project was built with the power of ☕ caffeine, 🎵 lo-fi beats, and an unwavering belief that status lines deserve better.

## 📄 License

MIT — because sharing is caring! 💕

---

_Built with 🦀 Rust and an mass of ❤️_

_"I replaced a bash script with a Rust binary and mass all I got was this mass README" — mass Developer, 2026_
