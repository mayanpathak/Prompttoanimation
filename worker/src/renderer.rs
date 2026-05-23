use anyhow::{anyhow, Result};
use std::path::PathBuf;
use walkdir::WalkDir;
use anyhow::{Context, };
use tokio::fs;
use std::path::{Path, };
use crate::filesystem::Workspace;

// ======================================================
// Find Final Rendered Video
// ======================================================

pub fn find_rendered_video(
    workspace: &Workspace,
) -> Result<PathBuf> {

    for entry in WalkDir::new(&workspace.root) {

        let entry = entry?;

        let path = entry.path();

        // ==========================================
        // Skip directories
        // ==========================================

        if path.is_dir() {
            continue;
        }

        // ==========================================
        // Skip partial movie files
        // ==========================================

        let path_string = path.to_string_lossy();

        if path_string.contains("partial_movie_files") {
            continue;
        }

        // ==========================================
        // Find final mp4
        // ==========================================

        if path.extension()
            == Some(std::ffi::OsStr::new("mp4"))
        {
            println!("\nFOUND FINAL VIDEO:");
            println!("{:?}", path);

            return Ok(path.to_path_buf());
        }
    }

    Err(anyhow!(
        "No final rendered video found"
    ))
}




pub async fn move_rendered_video(
    rendered_video_path: &Path,
    job_id: &str,
) -> Result<String> {

    println!("\n=====================================");
    println!("MOVING RENDERED VIDEO");
    println!("=====================================");

    println!(
        "\nSource Video Path:\n{:?}",
        rendered_video_path
    );

    // ==================================================
    // Create Static Render Directory
    // ==================================================

    let static_render_dir = PathBuf::from(
        "./static/renders"
    );

    fs::create_dir_all(&static_render_dir)
        .await
        .context("Failed to create static/renders directory")?;

    println!(
        "\nStatic Render Directory Ready:\n{:?}",
        static_render_dir
    );

    // ==================================================
    // Final Destination Path
    // ==================================================

    let final_video_filename = format!(
        "{}.mp4",
        job_id
    );

    let final_video_path = static_render_dir.join(
        &final_video_filename
    );

    println!(
        "\nFinal Video Destination:\n{:?}",
        final_video_path
    );

    // ==================================================
    // Move File
    // ==================================================

    fs::rename(
        rendered_video_path,
        &final_video_path,
    )
    .await
    .context("Failed to move rendered video")?;

    println!("\nVideo moved successfully");

    // ==================================================
    // Public URL Path
    // ==================================================

    let public_video_path = format!(
        "/renders/{}",
        final_video_filename
    );

    println!(
        "\nPublic Video Path:\n{}",
        public_video_path
    );

    Ok(public_video_path)
}