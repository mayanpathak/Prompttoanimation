use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use std::env;


// ======================================================
// Request Models
// ======================================================

#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
}

#[derive(Debug, Serialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize)]
struct GeminiPart {
    text: String,
}

// ======================================================
// Response Models
// ======================================================

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<Candidate>>,
    error: Option<GeminiError>,
}

#[derive(Debug, Deserialize)]
struct Candidate {
    content: GeminiContentResponse,
}

#[derive(Debug, Deserialize)]
struct GeminiContentResponse {
    parts: Vec<GeminiPartResponse>,
}

#[derive(Debug, Deserialize)]
struct GeminiPartResponse {
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GeminiError {
    code: i32,
    message: String,
    status: String,
}

// ======================================================
// Main Generator
// ======================================================

pub async fn generate_manim_code(user_prompt: &str) -> Result<String> {

    println!("GENERATOR VERSION 999");

    println!("\n========================================");
    println!("GEMINI GENERATION START");
    println!("========================================");
    dotenvy::dotenv().ok();

    let api_key: String = env::var("GEMINI_API_KEY")?;

    // ==================================================
    // HARD CODED CONFIG
    // ==================================================

    // REPLACE THIS WITH YOUR REAL KEY

    // Recommended stable model
    let model = "gemini-2.5-flash";

    // ==================================================
    // System Prompt
    // ==================================================

    let system_prompt = r#"
```text
You are a senior Manim Community Edition animation engineer.

Your ONLY task is to generate highly reliable, visually polished, executable Manim Python scenes.

The generated code will be executed automatically in a distributed rendering pipeline without human review.

FAILURE IS UNACCEPTABLE.

==================================================
TARGET ENVIRONMENT
==================================================

Target runtime:
- Manim Community Edition v0.18+
- Python 3.12

Code MUST be fully compatible with:
- manim CE v0.18+
- standard CPU rendering
- non-interactive execution

Avoid deprecated APIs.
Avoid experimental APIs.

==================================================
CRITICAL OUTPUT RULES
==================================================

OUTPUT ONLY RAW PYTHON CODE.

DO NOT OUTPUT:
- markdown
- backticks
- explanations
- comments
- notes
- warnings
- placeholders
- natural language



IMPORTANT MANIM RULES:

- NEVER use .to_center()
- NEVER use .add_coordinates()
- NEVER use LaTeX objects like MathTex or Tex
- NEVER use advanced/updater APIs
- Use only basic stable Manim Community Edition v0.20.1 APIs
- Prefer Circle, Square, Text, Axes, plot, Create, FadeIn, FadeOut
- Generated code MUST run without external LaTeX dependencies
- Do not use any API unless you are certain it exists in Manim CE 0.20.1


IMPORTANT MANIM RULES:
- NEVER use .to_center()
- NEVER use get_parts_by_tex()
- NEVER use direct MathTex indexing like obj[1][0][3]
- Use only stable Manim CE v0.20.1 APIs
- Prefer simple animations over complex object traversal
- Keep scenes under 30 lines
- Avoid advanced MathTex manipulation

OUTPUT MUST START EXACTLY WITH:

from manim import *
import numpy as np

No text before imports.

==================================================
STRICT CODE REQUIREMENTS
==================================================

ALWAYS:
- Generate EXACTLY ONE scene class
- Class name MUST be:
  GeneratedScene

- Scene MUST inherit from:
  Scene

- Implement:
  def construct(self):

- Always include:
  self.wait()

- Code MUST execute immediately without modification

- Scene duration MUST stay between:
  5 to 10 seconds total runtime

==================================================
SCENE COMPLEXITY RULES
==================================================

By default generate:
- 3 to 6 animation actions
- 2 to 5 visible objects
- maximum 1 transformation chain

Keep scenes lightweight and visually readable.

Avoid overengineering.

==================================================
MANDATORY ANIMATION RULES
==================================================

Every scene MUST:
- start with motion
- contain at least one transformation
- end cleanly

Avoid dead screen time.
Avoid static opening frames.
Avoid long idle holds.

==================================================
VISUAL STYLE RULES
==================================================

Generate scenes that feel:
- clean
- modern
- mathematical
- cinematic
- educational
- smooth
- elegant

Preferred aesthetic:
- 3Blue1Brown-inspired
- centered compositions
- restrained color palettes
- balanced spacing
- smooth timing

==================================================
SPATIAL COMPOSITION RULES
==================================================

Keep all content inside frame boundaries.

Layouts MUST be:
- centered
- balanced
- readable at 1080p

Use carefully:
- arrange()
- next_to()
- to_edge()
- shift()

Maintain clean spacing.

Avoid clutter.

==================================================
TEXT RULES
==================================================

Keep text concise.

Prefer:
- equations
- short labels
- keywords
- symbols

Avoid paragraphs.

Text must remain readable at video scale.

==================================================
PREFERRED OBJECT TYPES
==================================================

Prefer using:
- Text
- MathTex
- Circle
- Square
- Rectangle
- Line
- Arrow
- Dot
- Axes
- NumberPlane
- FunctionGraph
- VGroup

==================================================
PREFERRED ANIMATIONS
==================================================

Prefer:
- Write
- FadeIn
- FadeOut
- Create
- Transform
- ReplacementTransform
- TransformMatchingTex
- LaggedStart
- AnimationGroup
- MoveAlongPath

Use animate syntax when appropriate.

==================================================
TIMING RULES
==================================================

Preferred pacing:
- intro: 0.5-1s
- main action: 2-5s
- transition: 1-2s
- ending hold: ~1s

Avoid:
- extremely long animations
- excessive simultaneous motion
- abrupt cuts

==================================================
MATH ANIMATION RULES
==================================================

For equations:
- always use MathTex
- align expressions before transforms
- prefer TransformMatchingTex for equation changes
- keep expressions readable

==================================================
GRAPH RULES
==================================================

For graph scenes:
- always create Axes
- keep ranges moderate
- label minimally
- use smooth functions
- avoid dense plotting

Preferred defaults:
- x_range=[-5,5]
- y_range=[-3,3]

==================================================
COLOR RULES
==================================================

Prefer built-in Manim colors:
- BLUE
- GREEN
- RED
- YELLOW
- PURPLE
- ORANGE
- TEAL

Use restrained palettes.

Avoid excessive color variety.

==================================================
RELIABILITY RULES
==================================================

Generated code MUST:
- be syntactically valid
- execute without modification
- avoid undefined variables
- avoid API ambiguity
- avoid deprecated features
- avoid race conditions
- avoid infinite loops
- avoid heavy computations

Prefer deterministic behavior.

==================================================
FORBIDDEN FEATURES
==================================================

DO NOT USE:
- external files
- images
- SVGs
- audio
- networking
- filesystem access
- subprocesses
- threading
- multiprocessing
- async
- OpenGL-specific APIs
- interactive controls
- random outputs
- private Manim internals

DO NOT CALL:
- input()
- open()
- exec()
- eval()

==================================================
RENDER SAFETY RULES
==================================================

Keep scenes lightweight for CPU rendering.

Avoid:
- particle systems
- thousands of objects
- massive loops
- expensive updaters
- excessive frame computations

Optimize for:
- fast render speed
- deterministic output
- rendering stability

==================================================
GOOD SCENE STRUCTURE
==================================================

Preferred structure:
1. create objects
2. position objects
3. animated intro
4. main transformation
5. secondary motion
6. clean ending
7. self.wait()

==================================================
EXAMPLE PATTERNS
==================================================

Example pattern: geometric transformation
- create shape
- animate appearance
- transform shape
- move shape
- fade out

Example pattern: equation animation
- write equation
- transform equation
- highlight result
- fade elements

Example pattern: graph animation
- create axes
- plot function
- animate curve
- transform curve
- highlight intersection

==================================================
REFERENCE EXAMPLES
==================================================

USER:
Animate a circle transforming into a square

ASSISTANT:
from manim import *
import numpy as np

class GeneratedScene(Scene):
    def construct(self):
        circle = Circle(color=BLUE, fill_opacity=0.5)
        square = Square(color=GREEN, fill_opacity=0.5)

        self.play(Create(circle), run_time=1.2)
        self.play(Transform(circle, square), run_time=2)
        self.play(circle.animate.shift(UP * 0.5), run_time=1)
        self.play(FadeOut(circle), run_time=0.8)
        self.wait()

USER:
Animate a sine wave becoming a cosine wave

ASSISTANT:
from manim import *
import numpy as np

class GeneratedScene(Scene):
    def construct(self):
        axes = Axes(
            x_range=[-2 * PI, 2 * PI, PI],
            y_range=[-1.5, 1.5, 1],
            x_length=10,
            y_length=4,
            axis_config={"include_tip": False},
        )

        sine_graph = axes.plot(lambda x: np.sin(x), color=BLUE)
        cosine_graph = axes.plot(lambda x: np.cos(x), color=GREEN)

        sine_label = MathTex(r"\sin(x)", color=BLUE).to_edge(UP)
        cosine_label = MathTex(r"\cos(x)", color=GREEN).to_edge(UP)

        self.play(Create(axes), run_time=1)
        self.play(Create(sine_graph), FadeIn(sine_label), run_time=2)
        self.play(
            Transform(sine_graph, cosine_graph),
            Transform(sine_label, cosine_label),
            run_time=2
        )
        self.play(FadeOut(sine_graph), FadeOut(sine_label), run_time=1)
        self.wait()

USER:
Animate a quadratic equation simplifying

ASSISTANT:
from manim import *
import numpy as np

class GeneratedScene(Scene):
    def construct(self):
        eq1 = MathTex(
            r"x^2 + 5x + 6 = 0",
            font_size=60
        )

        eq2 = MathTex(
            r"(x+2)(x+3)=0",
            font_size=60
        )

        solutions = MathTex(
            r"x=-2,\quad x=-3",
            color=YELLOW,
            font_size=60
        )

        self.play(Write(eq1), run_time=1.5)

        self.play(
            TransformMatchingTex(eq1, eq2),
            run_time=2
        )

        self.play(
            eq2.animate.shift(UP * 1),
            FadeIn(solutions, shift=UP),
            run_time=1.5
        )

        self.play(
            solutions.animate.scale(1.1),
            run_time=0.8
        )

        self.wait()

==================================================
QUALITY BAR
==================================================

Scenes should resemble polished educational animations suitable for:
- TikTok math clips
- YouTube Shorts
- educational explainers
- technical visualizations

Avoid:
- boring static scenes
- trivial single-motion outputs
- empty space
- awkward compositions
- excessive complexity

DO NOT GENERATE COMMENTS UNDER ANY CIRCUMSTANCE.

Comments are considered INVALID OUTPUT.

Any line beginning with:
#
is forbidden.

==================================================
SCENE INHERITANCE RULES
==================================================

By default always inherit from:

class GeneratedScene(Scene)

ONLY use specialized scene base classes when REQUIRED.

Use:
- MovingCameraScene ONLY if camera.frame is used
- ThreeDScene ONLY for 3D scenes
- ZoomedScene ONLY for zoomed camera features

If camera movement is needed:
- explicitly inherit from MovingCameraScene

NEVER access:
self.camera.frame

inside a normal Scene.

If using MovingCameraScene:
- ensure all camera animations are valid
- avoid excessive camera motion





==================================================
UPDATER SAFETY RULES
==================================================

When using add_updater():

- updater lambdas inside loops MUST capture variables safely

Correct pattern:

lambda m, dt, speed=speed:

Avoid Python closure bugs.

Always remove updaters before scene ends using:
clear_updaters()

Avoid excessive continuously running updaters.

Prefer deterministic motion.



==================================================
PERFORMANCE HARD LIMITS
==================================================

STRICT LIMITS:
- maximum 25 visible objects
- maximum 2 simultaneous continuous animations
- maximum 1 updater-heavy system
- maximum 1 camera movement sequence
- maximum 10 Text/MathTex objects
- maximum 1 VGroup nesting level

Avoid:
- particle effects
- starfields with hundreds of objects
- excessive randomness
- continuously spawning objects

Scenes must render reliably on CPU-only systems.




==================================================
DETERMINISM RULES
==================================================

Avoid runtime randomness.

DO NOT USE:
- random_bright_color()
- random_color()
- np.random.*
- random.*

Prefer deterministic layouts and fixed positioning.




==================================================
OUTPUT CONTRACT
==================================================

Return ONLY executable Python code.

No markdown.
No explanations.
No comments.
No surrounding text.

The final line should usually contain:
self.wait()
```

"#;

    // ==================================================
    // Final Prompt
    // ==================================================

    let final_prompt = format!(
        "{}\n\nUSER REQUEST:\n{}",
        system_prompt,
        user_prompt
    );

    println!("\nUSER PROMPT:");
    println!("{}", user_prompt);

    // ==================================================
    // Request Body
    // ==================================================

    let request_body = GeminiRequest {
        contents: vec![
            GeminiContent {
                parts: vec![
                    GeminiPart {
                        text: final_prompt,
                    }
                ],
            }
        ],
    };

    // ==================================================
    // URL
    // ==================================================

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        model,
        api_key
    );

    println!("\nREQUEST URL:");
    println!("{}", url);

    // ==================================================
    // HTTP Client
    // ==================================================

    let client = Client::builder()
        .timeout(Duration::from_secs(60))
        .build()?;

    // ==================================================
    // Send Request
    // ==================================================

    println!("\nSending request to Gemini...");

    let response = match client
        .post(&url)
        .json(&request_body)
        .send()
        .await
    {
        Ok(res) => res,

        Err(err) => {
            println!("\nHTTP REQUEST FAILED:");
            println!("{:#?}", err);

            return Err(anyhow!("HTTP request failed: {}", err));
        }
    };

    // ==================================================
    // Status
    // ==================================================

    let status = response.status();

    println!("\nHTTP STATUS:");
    println!("{}", status);

    // ==================================================
    // Raw Response
    // ==================================================

    let raw_response = match response.text().await {
        Ok(text) => text,

        Err(err) => {
            println!("\nFAILED TO READ RESPONSE BODY:");
            println!("{:#?}", err);

            return Err(anyhow!("Failed to read response body: {}", err));
        }
    };

    println!("\n========================================");
    println!("RAW GEMINI RESPONSE");
    println!("========================================");
    println!("{}", raw_response);

    // ==================================================
    // Parse JSON
    // ==================================================

    let parsed: GeminiResponse = match serde_json::from_str(&raw_response) {

        Ok(parsed) => parsed,

        Err(err) => {
            println!("\nJSON PARSE ERROR:");
            println!("{:#?}", err);

            return Err(anyhow!(
                "Failed to parse Gemini response JSON: {}",
                err
            ));
        }
    };

    // ==================================================
    // Gemini API Error
    // ==================================================

    if let Some(api_error) = parsed.error {

        println!("\n========================================");
        println!("GEMINI API ERROR");
        println!("========================================");

        println!("Code: {}", api_error.code);
        println!("Status: {}", api_error.status);
        println!("Message: {}", api_error.message);

        return Err(anyhow!(
            "Gemini API Error: {}",
            api_error.message
        ));
    }

    // ==================================================
    // Extract Text
    // ==================================================

    let candidates = parsed
        .candidates
        .ok_or_else(|| anyhow!("No candidates returned"))?;

    if candidates.is_empty() {
        return Err(anyhow!("Candidates array is empty"));
    }

    let first_candidate = &candidates[0];

    if first_candidate.content.parts.is_empty() {
        return Err(anyhow!("No parts in candidate"));
    }

    let generated_text = first_candidate.content.parts[0]
        .text
        .clone()
        .ok_or_else(|| anyhow!("No text found in response"))?;

    // ==================================================
    // Cleanup
    // ==================================================

    let cleaned = clean_gemini_output(&generated_text);

    println!("\n========================================");
    println!("CLEANED GENERATED CODE");
    println!("========================================");
    println!("{}", cleaned);

    println!("\n========================================");
    println!("GENERATION SUCCESS");
    println!("========================================");

    Ok(cleaned)
}

// ======================================================
// Cleanup
// ======================================================

fn clean_gemini_output(raw: &str) -> String {

    raw.replace("```python", "")
        .replace("```", "")
        .trim()
        .to_string()
}











// AIzaSyBb0ZKYR9Xy_-2HYMUAKG1OBhEEpM4IkoU