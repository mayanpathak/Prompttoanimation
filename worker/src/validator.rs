
use anyhow::{
    anyhow,
    Result,
};

use regex::Regex;

// ======================================================
// Validate Generated Manim Code
// ======================================================

pub fn validate_generated_code(
    code: &str,
) -> Result<()> {

    validate_imports(code)?;

    validate_forbidden_patterns(code)?;

    validate_scene_structure(code)?;

    validate_camera_usage(code)?;

    validate_updater_safety(code)?;

    validate_object_limits(code)?;

    validate_runtime_safety(code)?;

    validate_basic_quality(code)?;

    Ok(())
}

// ======================================================
// Import Validation
// ======================================================

fn validate_imports(
    code: &str,
) -> Result<()> {

    let allowed_imports = [
        "from manim import *",
        "import numpy as np",
    ];

    for line in code.lines() {

        let trimmed = line.trim();

        if trimmed.starts_with("import ")
            || trimmed.starts_with("from ")
        {
            if !allowed_imports.contains(&trimmed) {

                return Err(anyhow!(
                    "Forbidden import detected: {}",
                    trimmed
                ));
            }
        }
    }

    Ok(())
}

// ======================================================
// Forbidden Pattern Validation
// ======================================================

fn validate_forbidden_patterns(
    code: &str,
) -> Result<()> {

    let forbidden_patterns = [
        "os.",
        "subprocess",
        "socket",
        "requests",
        "pathlib",
        "shutil",
        "threading",
        "multiprocessing",
        "asyncio",
        "open(",
        "exec(",
        "eval(",
        "compile(",
        "__import__",
        "input(",
        "globals(",
        "locals(",
        "setattr(",
        "getattr(",
        "delattr(",
        "breakpoint(",
        "while True",
        "while(True)",
        "sys.",
        "tempfile",
        "pickle",
        "marshal",
        "ctypes",
        "resource.",
        "fork(",
        "signal.",
    ];

    for pattern in forbidden_patterns {

        if code.contains(pattern) {

            return Err(anyhow!(
                "Forbidden pattern detected: {}",
                pattern
            ));
        }
    }

    Ok(())
}

// ======================================================
// Scene Structure Validation
// ======================================================

fn validate_scene_structure(
    code: &str,
) -> Result<()> {

    if !code.contains("class GeneratedScene") {

        return Err(anyhow!(
            "GeneratedScene class missing"
        ));
    }

    let valid_scene_types = [
        "class GeneratedScene(Scene):",
        "class GeneratedScene(MovingCameraScene):",
        "class GeneratedScene(ThreeDScene):",
        "class GeneratedScene(ZoomedScene):",
    ];

    let mut valid = false;

    for scene_type in valid_scene_types {

        if code.contains(scene_type) {
            valid = true;
            break;
        }
    }

    if !valid {

        return Err(anyhow!(
            "Invalid scene inheritance"
        ));
    }

    if !code.contains("def construct(self):") {

        return Err(anyhow!(
            "construct method missing"
        ));
    }

    if !code.contains("self.wait(") {

        return Err(anyhow!(
            "self.wait() missing"
        ));
    }

    Ok(())
}

// ======================================================
// Camera Validation
// ======================================================

fn validate_camera_usage(
    code: &str,
) -> Result<()> {

    let uses_camera_frame = code.contains("self.camera.frame");

    let uses_moving_camera_scene =
        code.contains("MovingCameraScene");

    if uses_camera_frame
        && !uses_moving_camera_scene
    {

        return Err(anyhow!(
            "self.camera.frame requires MovingCameraScene"
        ));
    }

    Ok(())
}

// ======================================================
// Updater Validation
// ======================================================

fn validate_updater_safety(
    code: &str,
) -> Result<()> {

    if code.contains("add_updater(") {

        if code.contains("for ")
            && code.contains("lambda")
        {
            let safe_lambda_pattern = Regex::new(
                r"lambda\s+\w+\s*,\s*\w+\s*,"
            )?;

            if !safe_lambda_pattern.is_match(code) {

                return Err(anyhow!(
                    "Updater lambda likely has closure capture bug"
                ));
            }
        }

        if !code.contains("clear_updaters()") {

            return Err(anyhow!(
                "Updaters must be cleared before scene ends"
            ));
        }
    }

    Ok(())
}

// ======================================================
// Object Count Validation
// ======================================================

fn validate_object_limits(
    code: &str,
) -> Result<()> {

    let object_creations = [
        "Circle(",
        "Square(",
        "Rectangle(",
        "Dot(",
        "Line(",
        "Arrow(",
        "Text(",
        "MathTex(",
        "Axes(",
        "NumberPlane(",
        "Polygon(",
        "VGroup(",
    ];

    let mut total_objects = 0;

    for pattern in object_creations {

        total_objects += code.matches(pattern).count();
    }

    if total_objects > 25 {

        return Err(anyhow!(
            "Scene too complex: {} visible objects detected",
            total_objects
        ));
    }

    Ok(())
}

// ======================================================
// Runtime Safety Validation
// ======================================================

fn validate_runtime_safety(
    code: &str,
) -> Result<()> {

    let dangerous_runtime_patterns = [
        "np.random",
        "random.",
        "random_bright_color",
        "always_redraw(",
        "TracedPath(",
        "UpdateFromFunc(",
        "rate_func=there_and_back_with_pause",
        "for _ in range(1000)",
        "for _ in range(500)",
        "for i in range(1000)",
        "for i in range(500)",
    ];

    for pattern in dangerous_runtime_patterns {

        if code.contains(pattern) {

            return Err(anyhow!(
                "Potentially expensive runtime pattern detected: {}",
                pattern
            ));
        }
    }

    Ok(())
}

// ======================================================
// Basic Quality Validation
// ======================================================

fn validate_basic_quality(
    code: &str,
) -> Result<()> {

    let animation_calls = code.matches("self.play(").count();

    if animation_calls < 2 {

        return Err(anyhow!(
            "Scene too simple: insufficient animations"
        ));
    }

    if animation_calls > 15 {

        return Err(anyhow!(
            "Scene too complex: too many animation calls"
        ));
    }

    let line_count = code.lines().count();

    if line_count > 300 {

        return Err(anyhow!(
            "Generated code too large"
        ));
    }

    Ok(())
}

// ======================================================
// Unit Tests
// ======================================================

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn valid_scene_passes() {

        let code = r#"
from manim import *
import numpy as np

class GeneratedScene(Scene):
    def construct(self):
        circle = Circle()
        self.play(Create(circle))
        self.play(circle.animate.shift(RIGHT))
        self.wait()
"#;

        let result = validate_generated_code(code);

        assert!(result.is_ok());
    }

    #[test]
    fn forbidden_import_fails() {

        let code = r#"
from manim import *
import os

class GeneratedScene(Scene):
    def construct(self):
        self.wait()
"#;

        let result = validate_generated_code(code);

        assert!(result.is_err());
    }

    #[test]
    fn camera_frame_without_moving_camera_scene_fails() {

        let code = r#"
from manim import *
import numpy as np

class GeneratedScene(Scene):
    def construct(self):
        self.camera.frame.animate.scale(2)
        self.wait()
"#;

        let result = validate_generated_code(code);

        assert!(result.is_err());
    }
}
