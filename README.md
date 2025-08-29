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

## 🚀 Getting Started (User Installation)

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
<td><strong>Strongly Recommended:</strong> In OBS, go to File -> Settings -> Output -> Streaming and set encoder to NVENC with p7 (highest quality) preset</td>
</tr>
<tr>
<td align="center">7️⃣</td>
<td><strong>Start gaming!</strong> OWL Control will automatically record supported games</td>
</tr>
</table>

## 🛡️ Risks And Additional Information

Audio: OWL control does not record microphone inputs. It does record all system audio. Be aware of this if on voice chat/watching videos in the background during your play session.  
Accidental Recording: We have observed a bug where sometimes OWL control responds to F4 (the default record button, which can be accidentally activated if one alt-f4s a game to close it) right after you close a game. In cases where this happens, it can be a good idea to quickly double check OBS after you close a game. Black recordings that result from this will be filtered out of the uploaded dataset but might still upload.  
Processing: All data will undergo an automated vetting process to ensure we aren't using any empty recordings. That being said, OWL Control specifically sets OBS to record full screen applications, so there is no risk of accidental desktop capture.  
Data Verification: You can press "file -> show recordings" in OBS if you want to verify your data is recording properly before upload.  

## 💻 System Requirements

An NVIDIA GPU is reccomended. You should set your preset in OBS to NVENC with P7 otherwise you may get lag. AMD GPUs should have an equivalent.  
Video upload can be bandwidth intensive, though we downsample videos to 360p and set an appropriate bitrate to lighten the load as much as possible.   

## ⚙️ Uploading  

Uploads are manual. You should open the tray icon and select "upload" in settings. That being said, this feature is a WIP and it may be better to instead zip up your files and upload elsewhere for persistent cloud storage. Note that you can always press show recordings to find all the files, as mentioned earlier.

## ⌨️ Default Hotkeys

<div align="center">

| Key/Action | Function |
|:----------:|:---------|
| **F4** | 🟢 Start recording manually |
| **F5** | 🔴 Stop recording |
| **System Tray Icon** | ⚙️ Access settings and controls |

Hotkeys for record/stop record can be changed in the applications settings.
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

First make sure you've installed Node, UV, and Rust (Cargo). It is reccomended to use powershell, and windows developer tools for this.  
Winget can sometimes make a lot of this easier for you, but online installers should work. If you have any issues with setup, ask your local LLM!

```bash
# Clone the repository
git clone https://github.com/Wayfarer-Labs/owl-control.git
cd owl_control

# Install dependencies
npm install
uv sync
.venv\Scripts\activate.ps1

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
