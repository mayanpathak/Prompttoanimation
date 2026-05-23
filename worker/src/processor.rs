use std::time::Duration;

use anyhow::Result;
use tokio::time::sleep;
use crate::docker::run_manim_container;
use crate::renderer::{
    find_rendered_video,
    move_rendered_video,
};
use crate::{
    generator::generate_manim_code,
    models::RenderResult,
    filesystem ::{
        create_workspace,
        write_scene_file,
        cleanup_workspace,
        Workspace,
    },
    services::job_service::{
        get_render_job_by_id,
        mark_job_completed,
        mark_job_failed,
        mark_job_generating,
    },
    AppState,
};

// ======================================================
// Process Render Job
// ======================================================

pub async fn process_render_job(
    state: &AppState,
    job_id: &str,
) -> Result<()> {



    println!("PROCESS_RENDER_JOB VERSION 777");
    println!("\n=================================================");
    println!("PROCESSING JOB");
    println!("JOB ID: {}", job_id);
    println!("=================================================");

    // ==================================================
    // Fetch Job
    // ==================================================

    println!("\nFetching render job...");

    let job = match get_render_job_by_id(
        state,
        job_id,
    )
    .await
    {
        Ok(job) => job,

        Err(err) => {

            eprintln!("\nFAILED TO FETCH JOB");
            eprintln!("{:#?}", err);

           return Err(err.into());
        }
    };

    let Some(job) = job else {

        eprintln!("\nJOB NOT FOUND");
        eprintln!("Job ID: {}", job_id);

        return Ok(());
    };

    println!("\nJOB FOUND");
    println!("Prompt: {}", job.prompt);

    // ==================================================
    // Mark Generating
    // ==================================================

    println!("\nMarking job as GeneratingCode...");

    if let Err(err) = mark_job_generating(
        state,
        job_id,
    )
    .await
    {
        eprintln!("\nFAILED TO MARK JOB AS GENERATING");
        eprintln!("{:#?}", err);

        return Err(err.into());
    }

    println!("Job marked as GeneratingCode");

    // ==================================================
    // Generate Manim Code
    // ==================================================

    println!("\nStarting Gemini generation...");

    let generated_code = match generate_manim_code(
        &job.prompt,
    )
    .await
    {
        Ok(code) => {

            println!("\nGEMINI GENERATION SUCCESS");

            code
        }

        Err(err) => {

            eprintln!("\n========================================");
            eprintln!("GEMINI GENERATION FAILED");
            eprintln!("========================================");

            eprintln!("{:#?}", err);

            // ==========================================
            // Mark Failed
            // ==========================================

            if let Err(db_err) = mark_job_failed(
                state,
                job_id,
                &format!("Gemini generation failed: {}", err),
            )
            .await
            {
                eprintln!("\nFAILED TO MARK JOB FAILED");
                eprintln!("{:#?}", db_err);
            }

            return Err(err);
        }
    };

    // ==================================================
    // Print Generated Code
    // ==================================================

    println!("\n=================================================");
    println!("GENERATED MANIM CODE");
    println!("=================================================\n");

    println!("{}", generated_code);





    // ==================================================
// Create Workspace
// ==================================================

println!("\nCreating temporary workspace...");

let workspace = match create_workspace(job_id).await {

    Ok(workspace) => {

        println!("\nWORKSPACE CREATED");
        println!("Root: {:?}", workspace.root);
        println!("Scene File: {:?}", workspace.scene_file);

        workspace
    }

    Err(err) => {

        eprintln!("\nFAILED TO CREATE WORKSPACE");
        eprintln!("{:#?}", err);

        if let Err(db_err) = mark_job_failed(
            state,
            job_id,
            &format!("Workspace creation failed: {}", err),
        )
        .await
        {
            eprintln!("\nFAILED TO MARK JOB FAILED");
            eprintln!("{:#?}", db_err);
        }

        return Err(err);
    }
};

// ==================================================
// Write scene.py
// ==================================================

println!("\nWriting generated code to scene.py...");

if let Err(err) = write_scene_file(
    &workspace,
    &generated_code,
)
.await
{
    eprintln!("\nFAILED TO WRITE SCENE FILE");
    eprintln!("{:#?}", err);

    // cleanup partial workspace
    let _ = cleanup_workspace(&workspace).await;

    if let Err(db_err) = mark_job_failed(
        state,
        job_id,
        &format!("Failed to write scene.py: {}", err),
    )
    .await
    {
        eprintln!("\nFAILED TO MARK JOB FAILED");
        eprintln!("{:#?}", db_err);
    }

    return Err(err);
}

println!("\nscene.py written successfully");

    



// ==================================================
// Run Docker Render
// ==================================================

println!("\nStarting Docker render...");

if let Err(err) = run_manim_container(&workspace).await {

    eprintln!("\nDOCKER RENDER FAILED");
    eprintln!("{:#?}", err);

    let _ = cleanup_workspace(&workspace).await;

    if let Err(db_err) = mark_job_failed(
        state,
        job_id,
        &format!("Docker render failed: {}", err),
    )
    .await
    {
        eprintln!("\nFAILED TO MARK JOB FAILED");
        eprintln!("{:#?}", db_err);
    }

    return Err(err);
}


    println!(
    "\nScene file exists: {}",
    workspace.scene_file.exists()
);

    // ==================================================
// Find Final Rendered Video
// ==================================================

println!("\nFinding final rendered video...");

let rendered_video_path = match find_rendered_video(
    &workspace
) {
    Ok(path) => {

        println!("\nFINAL VIDEO FOUND");
        println!("{:?}", path);

        path
    }

    Err(err) => {

        eprintln!("\nFAILED TO FIND FINAL VIDEO");
        eprintln!("{:#?}", err);

        let _ = cleanup_workspace(&workspace).await;

        if let Err(db_err) = mark_job_failed(
            state,
            job_id,
            &format!("Failed to find rendered video: {}", err),
        )
        .await
        {
            eprintln!("\nFAILED TO MARK JOB FAILED");
            eprintln!("{:#?}", db_err);
        }

        return Err(err);
    }
};

// ==================================================
// Move Final Video To Static Storage
// ==================================================

println!("\nMoving rendered video...");

let public_video_path = match move_rendered_video(
    &rendered_video_path,
    job_id,
)
.await
{
    Ok(path) => {

        println!("\nVIDEO MOVED SUCCESSFULLY");
        println!("Public Path: {}", path);

        path
    }

    Err(err) => {

        eprintln!("\nFAILED TO MOVE VIDEO");
        eprintln!("{:#?}", err);

        let _ = cleanup_workspace(&workspace).await;

        if let Err(db_err) = mark_job_failed(
            state,
            job_id,
            &format!("Failed to move rendered video: {}", err),
        )
        .await
        {
            eprintln!("\nFAILED TO MARK JOB FAILED");
            eprintln!("{:#?}", db_err);
        }

        return Err(err);
    }
};

// ==================================================
// Get Final File Metadata
// ==================================================

let metadata = tokio::fs::metadata(
    format!("./static{}", public_video_path)
)
.await?;

// ==================================================
// Real Render Result
// ==================================================

let result = RenderResult {

    video_path: public_video_path,

    duration_seconds: 0.0,

    file_size_bytes: metadata.len(),

    render_time_ms: 0,
};

println!("\nREAL RENDER RESULT CREATED");


    // ==================================================
    // Mark Completed
    // ==================================================

    println!("\nMarking job as completed...");

    if let Err(err) = mark_job_completed(
        state,
        job_id,
        result,
    )
    .await
    {

        eprintln!("\nFAILED TO MARK JOB COMPLETED");
        eprintln!("{:#?}", err);

        // ==============================================
        // Mark Failed
        // ==============================================

        if let Err(db_err) = mark_job_failed(
            state,
            job_id,
            &err.to_string(),
        )
        .await
        {
            eprintln!("\nFAILED TO MARK JOB FAILED");
            eprintln!("{:#?}", db_err);
        }

        return Err(err.into());
    }






// ==================================================
// Cleanup Workspace
// ==================================================

println!("\nCleaning up workspace...");

if let Err(err) = cleanup_workspace(&workspace).await {

    eprintln!("\nFAILED TO CLEANUP WORKSPACE");
    eprintln!("{:#?}", err);
}
else {

    println!("Workspace cleaned successfully");
}

    // ==================================================
    // Success
    // ==================================================

    println!("\n=================================================");
    println!("JOB COMPLETED SUCCESSFULLY");
    println!("JOB ID: {}", job_id);
    println!("=================================================\n");

    Ok(())
}