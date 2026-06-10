# Promptoanime

> **AI-powered prompt-to-animation rendering platform built in Rust.**
> Type a natural language description. Get a rendered Manim video back.

---
Showcase------


https://github.com/user-attachments/assets/8bd1c22b-985b-45be-83ac-c51d4a200a2a



## What This Is

`promptoanime` is a distributed backend system that converts plain English prompts into rendered mathematical animation videos using [Manim (ManimGL)](https://github.com/3b1b/manim) — the same engine behind 3Blue1Brown's videos.

You describe an animation. The system generates valid Manim Python code via the Gemini API, validates and sandboxes it inside a Docker container, renders it to an MP4, and serves the video back to your browser.

The engineering focus is not "AI video generation." The real value is the **distributed async rendering orchestration system written in Rust** — a production-grade demonstration of async job queues, concurrent workers, containerized execution, and fault-tolerant state machines.

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        CLIENT (Browser)                         │
│              POST /api/jobs  ──►  GET /api/jobs/:id             │
│                      (poll until completed)                     │
└────────────────────────────┬────────────────────────────────────┘
                             │ HTTP
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                   rustattennew  (Axum API Server)               │
│                                                                 │
│   ┌─────────────┐   ┌──────────────┐   ┌────────────────────┐  │
│   │ Auth Routes │   │  Job Routes  │   │  Static File Serve │  │
│   │  /api/auth/ │   │  /api/jobs/  │   │  /renders/:id.mp4  │  │
│   └─────────────┘   └──────┬───────┘   └────────────────────┘  │
│                            │                                    │
│   ┌──────────────────────────────────────────────────────────┐  │
│   │                     AppState                             │  │
│   │         MongoDB Client  +  Redis Pool                    │  │
│   └──────────────────────────────────────────────────────────┘  │
└──────────────┬──────────────────────────────┬───────────────────┘
               │ mongo write                  │ LPUSH render_queue
               ▼                              ▼
┌──────────────────────┐         ┌────────────────────────┐
│       MongoDB        │         │         Redis          │
│   jobs collection    │◄────────│    render_queue list   │
│   users collection   │  status │   (job_id strings)     │
└──────────────────────┘ updates └────────────┬───────────┘
                                              │ BRPOP
                                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                     worker  (Tokio Worker Binary)               │
│                                                                 │
│  ┌──────────┐   ┌─────────────┐   ┌───────────┐  ┌──────────┐  │
│  │ queue.rs │──►│ processor.rs│──►│generator.rs│►│validator │  │
│  │  BRPOP   │   │ orchestrate │   │Gemini API  │  │  .rs     │  │
│  └──────────┘   └──────┬──────┘   └───────────┘  └──────────┘  │
│                        │                                        │
│               ┌────────▼────────┐                               │
│               │  filesystem.rs  │  /tmp/renders/<job_id>/       │
│               │  write scene.py │  scene.py                     │
│               └────────┬────────┘                               │
│                        │                                        │
│               ┌────────▼────────┐                               │
│               │   docker.rs     │  docker run --network none    │
│               │ sandbox render  │  --memory 512m --cpus 1.0     │
│               └────────┬────────┘  manim scene.py GeneratedScene│
│                        │                                        │
│               ┌────────▼────────┐                               │
│               │  renderer.rs    │  find output .mp4             │
│               │  move to static │  mv → /static/renders/        │
│               └────────┬────────┘                               │
│                        │                                        │
│               ┌────────▼────────┐                               │
│               │  job_service.rs │  status = Completed           │
│               │  update mongo   │  video_path = /renders/id.mp4 │
│               └─────────────────┘                               │
│                                                                 │
│   Concurrency: Semaphore-limited Tokio tasks (2–3 workers)      │
└─────────────────────────────────────────────────────────────────┘
                             │
                             ▼
                    /static/renders/<job_id>.mp4
                    served directly by Axum
```

---

## Repository Structure

```
promptoanime/
├── rustattennew/          # Axum API server (producer)
│   └── src/
│       ├── main.rs        # Bootstrap: DB, Redis, routes, AppState
│       ├── controller/    # HTTP handlers (auth + jobs)
│       ├── services/      # Business logic, DB operations
│       ├── models/        # Domain types: RenderJob, JobStatus, User
│       ├── schema/        # Request/response validation structs
│       ├── middleware/     # JWT auth middleware
│       ├── routes/        # Route definitions
│       ├── utils/         # JWT, password hashing, cookie helpers
│       ├── emails/        # Email templates + lettre integration
│       ├── config/        # Redis pool, mailer config
│       └── db/            # MongoDB connection + ping
│
├── worker/                # Tokio worker binary (consumer)
│   └── src/
│       ├── main.rs        # Bootstrap: Gemini config, DB, Redis, state
│       ├── worker.rs      # BRPOP loop, Semaphore, task spawning
│       ├── processor.rs   # Full job pipeline orchestration
│       ├── generator.rs   # Gemini API + code extraction
│       ├── validator.rs   # AST safety checks, import allowlist
│       ├── docker.rs      # Docker container spawn + log capture
│       ├── renderer.rs    # Locate .mp4, move to static/
│       ├── filesystem.rs  # Temp workspace create/cleanup
│       ├── services/
│       │   └── job_service.rs  # MongoDB status transitions
│       ├── models.rs      # Shared job/result types
│       ├── configgemini.rs
│       ├── config.rs
│       ├── errors.rs
│       └── utils.rs
│
├── tests/                 # Integration test suite
│   └── src/
│       ├── main.rs
│       ├── test_payloads.rs
│       └── test_users.rs
│
├── examples/              # Sample requests and demo payloads
├── test-render/           # Render output artifacts and test scenes
└── test1payload/          # Targeted payload test harness
```

---

## Tech Stack

| Layer | Technology | Purpose |
|---|---|---|
| Language | Rust 2024 | Both API server and worker |
| Async Runtime | Tokio | Concurrent workers, async I/O |
| Web Framework | Axum 0.7 | REST API, middleware, static files |
| Database | MongoDB | Persistent job state, user accounts |
| Queue | Redis (LPUSH / BRPOP) | Job queue between API and worker |
| Redis Pool | deadpool-redis | Async connection pooling |
| Auth | jsonwebtoken + httpOnly cookies | Stateless JWT session |
| Validation | validator crate + serde | Input validation at API boundary |
| AI Generation | Google Gemini API | Manim Python code generation |
| HTTP Client | reqwest | Gemini API calls from worker |
| Rendering | Manim (ManimGL) + FFmpeg | Scene rendering to MP4 |
| Sandboxing | Docker | Isolated, resource-limited execution |
| Email | lettre | Password reset emails |
| Error Handling | thiserror + anyhow | Typed errors throughout |
| Serialization | serde / serde_json | All data layer + API contracts |

---

## Full Execution Flow

### 1. Client submits a prompt

```
POST /api/jobs
Authorization: httpOnly JWT cookie

{
  "prompt": "Animate a sine wave transforming into a cosine wave",
  "duration": 5
}
```

### 2. API server validates and enqueues

- Payload validated via `validator` + serde
- Auth middleware extracts `user_id` from JWT cookie
- New `RenderJob` document inserted into MongoDB with `status: Pending`
- Job ID pushed onto Redis queue via `LPUSH render_queue <job_id>`
- API returns immediately — **no blocking**:

```json
{ "job_id": "abc123", "status": "Pending" }
```

### 3. Worker wakes up

- Worker is blocking on `BRPOP render_queue 0`
- Redis returns `job_id` the moment it's enqueued
- Tokio task is spawned; `Semaphore` enforces max concurrency (2–3 workers)

### 4. Worker processes the job

```
fetch job from MongoDB
    ↓
mark status = GeneratingCode
    ↓
build system prompt (Manim docs + examples + constraints)
    ↓
call Gemini API
    ↓
extract raw Python from response (strip markdown, explanations)
    ↓
validate code (import allowlist, reject eval/exec/os/socket)
    ↓
create /tmp/renders/<job_id>/scene.py
    ↓
mark status = Rendering
    ↓
docker run --network none --memory 512m --cpus 1.0
           -v /tmp/renders/<job_id>:/workspace
           manim-image scene.py GeneratedScene -ql
    ↓
capture logs, detect output .mp4
    ↓
move video → /static/renders/<job_id>.mp4
    ↓
mark status = Completed, video_path = "/renders/<job_id>.mp4"
    ↓
cleanup /tmp/renders/<job_id>/
```

### 5. Client polls for result

```
GET /api/jobs/abc123
→ { "status": "Completed", "video_path": "/renders/abc123.mp4" }
```

Browser renders:
```html
<video src="/renders/abc123.mp4" controls />
```

---

## Job Status State Machine

```
PENDING
   │
   ▼ (worker picks up)
GENERATING_CODE
   │
   ├──► (Gemini fails / invalid code)  ──► FAILED
   │
   ▼ (code validated)
RENDERING
   │
   ├──► (Docker crash / timeout / FFmpeg error) ──► FAILED
   │
   ▼ (mp4 produced)
COMPLETED
```

Each transition is a MongoDB write. The API exposes current state on every `GET /api/jobs/:id` poll.

---

## Authentication System

Full JWT-based auth with httpOnly cookie transport:

| Route | Method | Description |
|---|---|---|
| `/api/auth/signup` | POST | Create account, send verification email |
| `/api/auth/login` | POST | Issue JWT in httpOnly cookie |
| `/api/auth/logout` | POST | Clear cookie |
| `/api/auth/check-auth` | GET | Validate current session |
| `/api/auth/forgot-password` | POST | Send reset email via lettre |
| `/api/auth/reset-password` | POST | Validate token, update password |

Auth middleware (`authmiddleware.rs`) extracts and verifies the JWT on every protected route, injecting `user_id` as an Axum request extension.

---

## Code Safety & Sandboxing

AI-generated Python is untrusted by default. Two layers of protection:

**Layer 1 — Static Validator (`validator.rs`)**

Rejects code containing:
- Disallowed imports (`os`, `subprocess`, `socket`, `requests`, `sys`)
- Dangerous builtins (`eval`, `exec`, `open`, `__import__`)
- Missing `class GeneratedScene(Scene):` definition

Only `from manimlib import *` and `import numpy as np` are permitted.

**Layer 2 — Docker Sandbox (`docker.rs`)**

Every render runs inside an isolated container:
- `--network none` — zero network access
- `--memory 512m` — hard memory cap
- `--cpus 1.0` — CPU throttle
- `--rm` — container auto-removed after exit
- Read-only filesystem except `/workspace` mount
- Hard timeout enforced by worker

This means even if the validator misses something, the container cannot touch the host system.

---

## Concurrency Model

The worker uses Tokio's `Semaphore` to limit parallel renders:

```rust
let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_JOBS)); // 2–3

loop {
    let job_id = brpop(&redis).await?;
    let permit = semaphore.clone().acquire_owned().await?;
    tokio::spawn(async move {
        process_job(job_id, state).await;
        drop(permit); // slot released
    });
}
```

- New jobs are accepted from the queue continuously
- Rendering is bounded by the semaphore
- Each job runs in its own Tokio task — non-blocking

---

## Getting Started

### Prerequisites

- Rust (stable, 2021 edition)
- MongoDB running locally
- Redis running locally
- Docker (for the worker renderer)
- `manimgl` Docker image built (`manim-image`)

### Environment Variables

Create `.env` in both `rustattennew/` and `worker/`:

```env
# Shared
MONGODB_URI=mongodb://localhost:27017
DB_NAME=promptoanime
REDIS_URL=redis://localhost:6379
PORT=8080

# API server
JWT_SECRET=your_jwt_secret_here
NODE_ENV=development
EMAIL_HOST=smtp.example.com
EMAIL_PORT=587
EMAIL_USER=your@email.com
EMAIL_PASS=yourpassword
SENDER_NAME=promptoanime

# Worker
GEMINI_API_KEY=your_gemini_key
GEMINI_MODEL=gemini-1.5-flash
MAX_CONCURRENT_JOBS=2
RENDER_TIMEOUT_SECS=120
STATIC_OUTPUT_DIR=../rustattennew/static/renders
```

### Run

```bash
# Terminal 1 — API server
cd rustattennew
cargo run

# Terminal 2 — Worker
cd worker
cargo run

# Terminal 3 — Integration tests
cargo test --manifest-path tests/Cargo.toml
```

---

## API Reference

### Jobs

**Create a render job**
```
POST /api/jobs
Content-Type: application/json

{ "prompt": "Animate a parabola morphing into a sine wave" }
```

Response:
```json
{ "job_id": "abc123", "status": "Pending", "created_at": "..." }
```

**Poll job status**
```
GET /api/jobs/:job_id
```

Response (completed):
```json
{
  "job_id": "abc123",
  "status": "Completed",
  "video_path": "/renders/abc123.mp4",
  "created_at": "...",
  "completed_at": "..."
}
```

**Serve rendered video**
```
GET /renders/:job_id.mp4
```

---

## Design Decisions

**Why Axum?**
Axum's extractor pattern maps cleanly to typed middleware. The `FromRequestParts` trait makes auth extraction composable and testable without coupling controllers to auth logic.

**Why MongoDB?**
Job documents are schemaless by nature — status fields, result payloads, and error messages vary by state. MongoDB's document model fits this better than rigid relational rows, and the `mongodb` async driver integrates naturally with Tokio.

**Why Redis LPUSH/BRPOP?**
For an MVP, BRPOP is the right tradeoff. Blocking pop means zero polling overhead — the worker sleeps at the OS level until a job arrives. The acknowledged tradeoff is that a crashed worker mid-job loses that job ID. A production upgrade would use Redis Streams with consumer groups for acknowledgement and recovery.

**Why separate worker binary?**
Separating the API server and worker into independent processes means the render pipeline can be scaled horizontally without touching the HTTP layer. It also means a crashing render cannot take down the API server.

**Why Docker for rendering?**
The system executes AI-generated Python. Without sandboxing, a malicious or broken generation could delete files, open sockets, or spin in an infinite loop. Docker gives hard resource limits and complete filesystem isolation with minimal overhead for this use case.

---

## Roadmap

- [ ] Redis Streams with consumer groups (acknowledgement + recovery)
- [ ] Retry logic with exponential backoff on generation failure
- [ ] Dead letter queue for permanently failed jobs
- [ ] Structured logging with `tracing` + `tracing-subscriber`
- [ ] Health check endpoint (`GET /health`)
- [ ] Rate limiting per user on job submission
- [ ] Job TTL — automatic video cleanup after N hours
- [ ] OpenAPI / Swagger documentation
- [ ] CI/CD pipeline (GitHub Actions)
- [ ] S3/MinIO for video storage (scalable artifact store)
- [ ] WebSocket progress streaming (replace polling)
- [ ] Frontend dashboard (Next.js)

---


## What This Demonstrates

This project was built specifically to demonstrate senior-level Rust backend engineering:

- **Async system design** — Tokio throughout, no blocking the event loop
- **Distributed processing** — decoupled producer/consumer across two binaries
- **Job queue architecture** — Redis-backed queue with real concurrency control
- **State machine persistence** — explicit status transitions in MongoDB
- **Secure auth** — JWT in httpOnly cookies, typed middleware extractors
- **Containerized execution** — Docker sandbox for untrusted AI code
- **Error propagation** — `thiserror` typed errors, `AppError` → HTTP status mapping
- **Modular architecture** — controller/service/model/schema separation, no god files
- **AI integration** — structured prompting, response cleaning, output validation









Showcase Videos ------------








https://github.com/user-attachments/assets/a85b7cb8-855e-46d1-b420-859206df094b



https://github.com/user-attachments/assets/94fe239b-b8bc-41c4-82b9-b6534be5a32c



https://github.com/user-attachments/assets/7f2b6cc2-fa62-4c3e-ae23-69d7b0d24f4d



https://github.com/user-attachments/assets/4ccce4a4-2e0f-4fd7-9029-fdb6829fbc39



https://github.com/user-attachments/assets/646612d2-8c5b-4975-80e5-6a7bd9d3f604



https://github.com/user-attachments/assets/e66414c2-b032-49c1-8c55-97e7f59a86f8



https://github.com/user-attachments/assets/6ff25910-1935-4a80-a463-a6d44138440e



https://github.com/user-attachments/assets/ed3272d0-f8f1-49b1-8ccd-cb7cd0357cf6












---

## License

MIT
