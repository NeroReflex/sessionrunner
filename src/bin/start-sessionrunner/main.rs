#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Try to connect to the D-Bus session bus to check if it's available
    let dbus_ready = match sessionrunner::zbus::Connection::session().await {
        Ok(_) => true,
        Err(err) => {
            eprintln!("Failed to connect to D-Bus session bus: {err}");
            false
        }
    };

    let mut command = match dbus_ready {
        true => tokio::process::Command::new("sessionrunner"),
        false => {
            let mut cmd = tokio::process::Command::new("dbus-launch");
            cmd.arg("sessionrunner");

            cmd
        }
    };

    let mut process = command
        .spawn()
        .inspect_err(|err| eprintln!("Error starting sessionrunner: {err}"))?;

    let exit_status = process
        .wait()
        .await
        .inspect_err(|err| eprintln!("Error waiting for sessionrunner: {err}"))?;

    if !exit_status.success() {
        eprintln!("sessionrunner exited with status: {exit_status}");
    }

    Ok(())
}
