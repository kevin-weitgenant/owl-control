<div align="center">
  
# 🦉 OWL Control

### **Help train the next generation of AI by sharing your gameplay!**

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

<p align="center">
  <strong>OWL Control</strong> is a desktop application that records gameplay footage and input data from video games<br/>
  to create open-source datasets for AI research. By using OWL Control,<br/>
  you're contributing to the development of AI agents and world models.
</p>

---

</div>

## 🎮 What is OWL Control?

OWL Control automatically records your gameplay sessions (video + keyboard/mouse inputs) from supported single-player games using OBS websocket. This data is uploaded to create a public dataset that researchers worldwide can use to train AI models.

### ✨ Key Features

| Feature | Description |
|---------|-------------|
| 🎬 **Automatic Recording** | Detects and records supported games automatically |
| 🎛️ **Full Control** | Start/stop recording anytime with hotkeys (F4/F5) |
| 🪶 **Lightweight** | Runs quietly in your system tray |

## 🚀 Getting Started

<table>
<tr>
<td width="50px" align="center">1️⃣</td>
<td><strong>Download</strong> the latest version from the <a href="https://github.com/Wayfarer-Labs/owl-control/releases">Releases</a> page</td>
</tr>
<tr>
<td align="center">2️⃣</td>
<td><strong>Install</strong> the application for your operating system</td>
</tr>
<tr>
<td align="center">3️⃣</td>
<td><strong>Create an account</strong> or enter your <a href="https://wayfarerlabs.ai/dashboard">API key</a></td>
</tr>
<tr>
<td align="center">4️⃣</td>
<td><strong>Review and accept</strong> the data collection terms</td>
</tr>
<tr>
<td align="center">5️⃣</td>
<td><strong>Install and setup OBS Studio</strong> (<a href="https://obsproject.com/">download</a>) and <a href="https://imgur.com/a/rN7C79y">enable websocket + disable authentication</a></td>
</tr>
<tr>
<td align="center">6️⃣</td>
<td><strong>Start gaming!</strong> OWL Control will automatically record supported games</td>
</tr>
</table>

## 🛡️ Privacy & Security

> **Your privacy is our top priority!**

<table>
<tr>
<td>✅</td>
<td><strong>No microphone recording</strong> - your voice stays private</td>
</tr>
<tr>
<td>✅</td>
<td><strong>No personal information</strong> captured</td>
</tr>
<tr>
<td>✅</td>
<td><strong>All data is anonymized</strong> before public release</td>
</tr>
<tr>
<td>✅</td>
<td><strong>You can delete recordings</strong> anytime</td>
</tr>
<tr>
<td>✅</td>
<td><strong>Completely voluntary</strong> - stop recording whenever you want</td>
</tr>
</table>

## 💻 System Requirements

- Any system capable of running your game and OBS
- 10-20GB of free storage space for recordings
- Low bandwidth internet connection (videos are compressed before upload)

## ⚙️ Upload Settings

OWL Control offers flexible upload scheduling:

- Automatic daily uploads at midnight
- Manual uploads when you open the application
- Configurable through the settings menu

## 📦 Data Collection

OWL Control only records gameplay footage and keyboard/mouse inputs. No microphone audio is recorded.

## ⌨️ Controls

<div align="center">

| Key/Action | Function |
|:----------:|:---------|
| **F4** | 🟢 Start recording manually |
| **F5** | 🔴 Stop recording |
| **System Tray Icon** | ⚙️ Access settings and controls |

</div>

## 🤝 Contributing to AI Research

<div align="center">
  <h3>By using OWL Control, you're helping to:</h3>
</div>

<table align="center">
<tr>
<td align="center">🤖</td>
<td><strong>Train AI agents</strong> to understand and play games</td>
</tr>
<tr>
<td align="center">🌍</td>
<td><strong>Develop better world models</strong> for AI systems</td>
</tr>
<tr>
<td align="center">📊</td>
<td><strong>Create open datasets</strong> for the research community</td>
</tr>
<tr>
<td align="center">🚀</td>
<td><strong>Advance the field</strong> of AI and machine learning</td>
</tr>
</table>

<div align="center">
  <em>✨ All collected data will be made publicly available for research purposes ✨</em>
</div>

## 💻 For Developers

> **OWL Control is open source!** If you're interested in the technical details or want to contribute:

### 🔨 Building from Source  

First make sure you've installed Node, UV, and Rust (Cargo)

```bash
# Clone the repository
git clone https://github.com/Wayfarer-Labs/owl-control.git
cd owl_control

# Install dependencies
npm install
uv sync

# Build the application
cargo build --release --bin owl-recorder
npm run build

# Run in development mode
npm run dev

# Package for distribution (includes Python bundling)
npm run package        # All platforms
npm run package:win    # Windows only
npm run package:mac    # macOS only
npm run package:linux  # Linux only
```

Currently only Windows is supported, although we'll be adding more platforms in the near future.

<div align="center">
  <em>📖 For detailed development instructions, see our <a href="docs/development.md">Development Guide</a></em>
</div>

## 📄 License

<div align="center">
  <em>This project is open source and available under the <a href="LICENSE">MIT License</a></em>
</div>

## 🙋 Support

<div align="center">

| Need Help? | Where to Go |
|:----------:|:------------|
| 🐛 **Issues or Bugs?** | Report them on our [GitHub Issues](https://github.com/Wayfarer-Labs/owl-control/issues) page |
| ❓ **Questions?** | Visit our [GitHub Issues](https://github.com/Wayfarer-Labs/owl-control/issues) page |

</div>

---

<div align="center">
  <br>
  <strong>🦉 OWL Control</strong> is a project by <a href="https://wayfarerlabs.ai">Wayfarer Labs</a>
  <br>
  <em>Building open datasets for AI research</em>
  <br><br>
  <a href="https://github.com/Wayfarer-Labs/owl-control">⭐ Star us on GitHub</a>
</div>
