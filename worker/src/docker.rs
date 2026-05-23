use anyhow::{anyhow, Result};
use tokio::process::Command;

use crate::filesystem::Workspace;

// ======================================================
// Run Manim Docker Container
// ======================================================

pub async fn run_manim_container(
    workspace: &Workspace,
) -> Result<()> {

    println!("\n=====================================");
    println!("STARTING DOCKER RENDER");
    println!("=====================================");

    println!(
        "\nWorkspace Root: {:?}",
        workspace.root
    );

    // ==============================================
    // Build Docker Mount Path
    // ==============================================

    let mount_path = format!(
        "{}:/workspace",
        workspace.root.display()
    );

    println!("\nMount Path:");
    println!("{}", mount_path);

    // ==============================================
    // Execute Docker
    // ==============================================

    let output = Command::new("docker")
        .arg("run")
        .arg("--rm")

        // Disable networking
        .arg("--network")
        .arg("none")

        // Limit memory
        .arg("--memory")
        .arg("512m")

        // Limit CPU
        .arg("--cpus")
        .arg("1.0")

        // Mount workspace
        .arg("-v")
        .arg(&mount_path)

        // Docker image
        .arg("manim-sandbox")

        // Command INSIDE container
        .arg("manim")
        .arg("/workspace/scene.py")
        .arg("GeneratedScene")
        .arg("-ql")

        .output()
        .await?;

    // ==============================================
    // Print Logs
    // ==============================================

    println!(
        "\nDOCKER STDOUT:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );

    println!(
        "\nDOCKER STDERR:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    // ==============================================
    // Handle Failure
    // ==============================================

    if !output.status.success() {

        return Err(anyhow!(
            "Docker render failed"
        ));
    }

    println!("\nDocker render completed successfully");

    Ok(())
}