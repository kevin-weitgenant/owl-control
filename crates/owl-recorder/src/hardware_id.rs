use color_eyre::{Result, eyre::OptionExt as _};
use tokio::process::Command;

pub(crate) async fn get() -> Result<String> {
    let output = Command::new("wmic")
        .args(["csproduct", "get", "uuid"])
        .output()
        .await?;
    let hardware_id = output
        .stdout
        .split(|&c| c == b'\n')
        .nth(1)
        .ok_or_eyre("Invalid output from wmic")?;
    let hardware_id = String::from_utf8(hardware_id.to_owned())?.trim().to_owned();
    Ok(hardware_id)
}
