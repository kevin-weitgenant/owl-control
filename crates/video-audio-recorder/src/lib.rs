use std::{
    path::Path,
    process::Stdio,
    sync::Arc,
    time::Duration,
};

use color_eyre::{
    Result,
    eyre::{Context, OptionExt as _, eyre},
};
use futures_util::Future;
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::Child as TokioChild,
    sync::Mutex,
    time::timeout,
};

#[derive(Serialize, Deserialize, Debug)]
struct BridgeMessage {
    #[serde(rename = "type")]
    message_type: String,
    data: serde_json::Value,
    timestamp: f64,
}

#[derive(Serialize, Deserialize, Debug)]
struct BridgeCommand {
    #[serde(rename = "type")]
    command_type: String,
    data: serde_json::Value,
}

struct OBSBridgeProcess {
    child: Arc<Mutex<Option<TokioChild>>>,
    stdin: Arc<Mutex<Option<tokio::process::ChildStdin>>>,
}

pub struct WindowRecorder {
    bridge: OBSBridgeProcess,
    #[allow(dead_code)]
    recording_path: String,
}

impl WindowRecorder {
    pub async fn start_recording(path: &Path, _pid: u32, _hwnd: usize) -> Result<WindowRecorder> {
        let recording_dir = path.parent()
            .ok_or_eyre("Recording path must have a parent directory")?;
        
        // Convert to absolute path for OBS
        let absolute_recording_path = std::fs::canonicalize(recording_dir)
            .wrap_err("Failed to get absolute path for recording directory")?;
        
        let recording_path = absolute_recording_path.to_str()
            .ok_or_eyre("Path must be valid UTF-8")?;

        tracing::debug!("Starting OBS bridge process");
        
        // Get the appropriate uv path (bundled in release, system in debug)
        let uv_path = if cfg!(debug_assertions) {
            "uv".to_string() // Use system uv in debug mode
        } else {
            // Use bundled uv in release mode - look for uv.exe next to the executable
            std::env::current_exe()
                .ok()
                .and_then(|exe| exe.parent().map(|p| p.join("uv.exe")))
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "uv".to_string()) // Fallback to system uv
        };
        
        tracing::debug!("Using uv path: {}", uv_path);
        
        // Start the Python OBS bridge process
        let mut command = tokio::process::Command::new(uv_path)
            .arg("run")
            .arg("--no-project")
            .arg("-m")
            .arg("vg_control.video.obs_bridge")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .wrap_err("Failed to spawn OBS bridge process")?;

        let stdin = command.stdin.take()
            .ok_or_eyre("Failed to get stdin handle")?;
        let stdout = command.stdout.take()
            .ok_or_eyre("Failed to get stdout handle")?;
        let stderr = command.stderr.take()
            .ok_or_eyre("Failed to get stderr handle")?;
        
        let bridge = OBSBridgeProcess {
            child: Arc::new(Mutex::new(Some(command))),
            stdin: Arc::new(Mutex::new(Some(stdin))),
        };

        let recorder = WindowRecorder {
            bridge,
            recording_path: recording_path.to_string(),
        };

        // Set up stderr reader in background to capture errors
        let stderr_reader = BufReader::new(stderr);
        tokio::spawn(async move {
            let mut stderr_lines = stderr_reader.lines();
            while let Ok(Some(line)) = stderr_lines.next_line().await {
                tracing::error!("OBS bridge stderr: {}", line);
            }
        });

        // Wait for ready message
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        
        if let Some(line) = timeout(Duration::from_secs(10), lines.next_line()).await?? {
            let message: BridgeMessage = serde_json::from_str(&line)
                .wrap_err("Failed to parse ready message")?;
            
            if message.message_type != "ready" {
                return Err(eyre!("Expected ready message, got: {}", message.message_type));
            }
            tracing::debug!("OBS bridge is ready");
        } else {
            return Err(eyre!("No ready message received from OBS bridge"));
        }

        // Initialize OBS
        recorder.send_command("initialize", serde_json::json!({
            "recording_path": recording_path
        })).await?;

        // Read initialize response
        if let Some(line) = timeout(Duration::from_secs(5), lines.next_line()).await?? {
            tracing::debug!("Raw initialize response: {}", line);
            let message: BridgeMessage = serde_json::from_str(&line)
                .wrap_err_with(|| format!("Failed to parse initialize response: '{}'", line))?;
            
            if message.message_type == "error" {
                return Err(eyre!("OBS initialization failed: {}", message.data.get("message").unwrap_or(&serde_json::Value::String("Unknown error".to_string()))));
            } else if message.message_type != "initialized" {
                return Err(eyre!("Expected initialized message, got: {}", message.message_type));
            }
            tracing::debug!("OBS initialized successfully");
        } else {
            return Err(eyre!("No initialize response received from OBS bridge"));
        }

        // Start recording
        recorder.send_command("start", serde_json::json!({})).await?;

        // Read start response
        if let Some(line) = timeout(Duration::from_secs(5), lines.next_line()).await?? {
            let message: BridgeMessage = serde_json::from_str(&line)
                .wrap_err("Failed to parse start response")?;
            
            if message.message_type == "error" {
                return Err(eyre!("OBS start recording failed: {}", message.data.get("message").unwrap_or(&serde_json::Value::String("Unknown error".to_string()))));
            } else if message.message_type != "start_requested" {
                tracing::warn!("Expected start_requested message, got: {}", message.message_type);
            }
            tracing::debug!("OBS start recording requested");
        } else {
            return Err(eyre!("No start response received from OBS bridge"));
        }

        // Once we're done reading stdout, spin up a task to echo stdout
        tokio::spawn(async move {
            while let Ok(Some(line)) = lines.next_line().await {
                tracing::info!("OBS bridge stdout: {}", line);
            }
        });

        tracing::debug!("OBS recording started successfully");
        Ok(recorder)
    }

    async fn send_command(&self, command_type: &str, data: serde_json::Value) -> Result<()> {
        let command = BridgeCommand {
            command_type: command_type.to_string(),
            data,
        };
        
        let command_json = serde_json::to_string(&command)?
            .replace('\n', "") + "\n";

        let mut stdin_guard = self.bridge.stdin.lock().await;
        if let Some(stdin) = stdin_guard.as_mut() {
            stdin.write_all(command_json.as_bytes()).await
                .wrap_err("Failed to write command to OBS bridge")?;
            stdin.flush().await
                .wrap_err("Failed to flush stdin")?;
        } else {
            return Err(eyre!("OBS bridge stdin not available"));
        }

        Ok(())
    }

    pub fn listen_to_messages(&self) -> impl Future<Output = Result<()>> + use<> {
        async move {
            // For now, just wait - we could implement proper message handling later
            // The OBS bridge will handle recording state internally
            tokio::time::sleep(Duration::from_millis(100)).await;
            Ok(())
        }
    }

    pub fn stop_recording(&self) {
        tracing::debug!("Stopping OBS recording");
        let stdin = self.bridge.stdin.clone();
        
        tokio::spawn(async move {
            let command = BridgeCommand {
                command_type: "stop".to_string(),
                data: serde_json::json!({}),
            };
            
            let command_json = match serde_json::to_string(&command) {
                Ok(json) => json + "\n",
                Err(e) => {
                    tracing::error!("Failed to serialize stop command: {}", e);
                    return;
                }
            };

            let mut stdin_guard = stdin.lock().await;
            if let Some(stdin) = stdin_guard.as_mut() {
                if let Err(e) = stdin.write_all(command_json.as_bytes()).await {
                    tracing::error!("Failed to send stop command: {}", e);
                }
                if let Err(e) = stdin.flush().await {
                    tracing::error!("Failed to flush stop command: {}", e);
                }
            }
        });
    }
}

impl Drop for WindowRecorder {
    fn drop(&mut self) {
        tracing::debug!("Shutting down OBS bridge process");
        
        // Send shutdown command (fire and forget)
        tokio::spawn({
            let stdin = self.bridge.stdin.clone();
            let child = self.bridge.child.clone();
            
            async move {
                // Try to send shutdown command
                let shutdown_cmd = match serde_json::to_string(&BridgeCommand {
                    command_type: "shutdown".to_string(),
                    data: serde_json::json!({}),
                }) {
                    Ok(json) => json + "\n",
                    Err(_) => return,
                };
                
                if let Ok(mut stdin_guard) = stdin.try_lock() {
                    if let Some(stdin) = stdin_guard.as_mut() {
                        let _ = stdin.write_all(shutdown_cmd.as_bytes()).await;
                        let _ = stdin.flush().await;
                    }
                }
                
                // Wait a bit then kill if needed
                tokio::time::sleep(Duration::from_millis(500)).await;
                
                if let Ok(mut child_guard) = child.try_lock() {
                    if let Some(child) = child_guard.as_mut() {
                        let _ = child.kill().await;
                    }
                }
            }
        });
    }
}