use anyhow::Result;
use std::path::PathBuf;
use tokio::fs;

#[derive(Debug, Clone)]
pub struct Workspace {
    pub root: PathBuf,
    pub scene_file: PathBuf,
    pub media_dir: PathBuf,
    pub output_dir: PathBuf,
}

pub async fn create_workspace(
    job_id: &str,
) -> Result<Workspace> {

    let root = std::env::temp_dir()
        .join("renders")
        .join(job_id);

    let media_dir = root.join("media");

    let output_dir = root.join("output");

    let scene_file = root.join("scene.py");

    // Create root workspace directory
    fs::create_dir_all(&root).await?;

    // Create media directory
    fs::create_dir_all(&media_dir).await?;

    // Create output directory
    fs::create_dir_all(&output_dir).await?;

    Ok(Workspace {
        root,
        scene_file,
        media_dir,
        output_dir,
    })
}

pub async fn write_scene_file(
    workspace: &Workspace,
    code: &str,
) -> Result<()> {

    fs::write(
        &workspace.scene_file,
        code,
    )
    .await?;

    Ok(())
}

pub async fn cleanup_workspace(
    workspace: &Workspace,
) -> Result<()> {

    if fs::try_exists(&workspace.root).await? {

        fs::remove_dir_all(
            &workspace.root,
        )
        .await?;
    }

    Ok(())
}