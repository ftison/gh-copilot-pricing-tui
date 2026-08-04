    
## Tech Stack
- **Language**: 
  - shell: for very simple scripts
  - rust: for core logic
  - lua: if business logic needs this agility
- **Framework**: TO BE DEFINED

## Tool rules
- When you need to search docs, use `context7` tool before searching on Internet.
- Else continue to use all your available tools.

## Code Style
- **Imports**: Group by `std`, external crates, then local modules
- **Components**: Use `#[component]` macro; prefer `impl IntoView` return type
- **Server Functions**: Use `#[server]` macro for API endpoints (e.g., `ValidateOobCode::url()`)
- **Feature Flags**: Code using `serde_json`, `reqwest`, `google-cloud-storage` must be behind `#[cfg(feature = "ssr")]`
- **Naming**: Snake_case for functions/variables, PascalCase for components/types
- **Error Handling**: Use `Result<T, ServerFnError>` for server functions
- **Logging**: Use `tracing::debug!()` for development, avoid `println!`
- **Testing**: Place tests in `#[cfg(test)] mod tests {}` blocks at end of file

## 1. Architectural and Design Principles

### 1.1. Type-Driven Design (TDD)
The goal is to leverage Rust's type system to ensure code correctness enforced at compile time. This approach ensures that many classes of errors are detected at compile time rather than runtime, substantially improving reliability.

| Rule | Directive |
| :--- | :--- |
| **Mandatory Validity** | Types must be **valid upon construction**. Encapsulate raw data (like `String`) in *newtypes* that perform necessary validation during initialization (e.g., via a `parse` method or similar constructor). |
| **State Modeling** | Utilize the **type state pattern** to represent valid state transitions. Represent distinct states (e.g., `UnverifiedUser` and `VerifiedUser`) as separate structs. Consuming the old instance (`self`) in transition methods guarantees the original instance can no longer be used, making **invalid states impossible to represent**. |
| **Encoding Invariants** | Encode safety properties and constraints directly into the API and types (Semantic Typing) to prevent common mistakes and ensure correct usage. |

### 1.2. Code Organization and Modularity
The project structure must maximize testability, scalability, and decoupling.

| Rule | Directive |
| :--- | :--- |
| **Binary Separation** | Keep `main.rs` as **thin as possible** and reserve reusable application logic for `lib.rs`. This separates reusable logic from the binary entry point, simplifying testing and benchmarking. |
| **Strict Decoupling** | Each module must have a **single responsibility**. Always question which **transitive dependencies not to allow**. The most important aspect of a module is what modules it does not depend upon. |
| **Controlled Visibility** | Be intentional about what is exposed (`pub`). Use **`pub(crate)`** to limit functions or modules visibility only within the same crate, hiding them from the outside world. Exposing too much code makes future refactoring harder. |
| **Workspace Usage** | For projects beyond a single package (e.g., splitting into a CLI and a core library), utilize a **Cargo workspace**. Workspaces manage related packages in one place and share a single `Cargo.lock` file. |
| **Imports** | Create a **curated prelude file** to re-export commonly used types, traits, and helpers, reducing repetitive imports. Do not dump the entire crate into the prelude. |

### 1.3. Concurrency and Communication
Concurrency must be managed securely to minimize contention and avoid common memory sharing pitfalls.

| Rule | Directive |
| :--- | :--- |
| **Preferred Model** | Favor **message passing** (channels, like those in Tokio) or the **Actors model** for communication between tasks. This helps avoid relying heavily on `Arc<Mutex<T>>`. |
| **Asynchronous Contracts** | When designing asynchronous interfaces, ensure that calling `Future::poll` returning `Poll::Pending` guarantees that `wake` will be called on the provided `Waker` when the future can progress. |

## 2. Prohibitions and Error Handling

### 2.1. Unsafe Code **PROHIBITION**

| Rule | Directive |
| :--- | :--- |
| **STRICT BAN** | **The use of the `unsafe` keyword is strictly prohibited** within the Agent's source code. |
| **Rationale** | Using `unsafe` delegates memory safety guarantees to the developer, which can lead to undefined behavior or memory corruption. |
| **FFI/Low-Level Needs** | If interaction with foreign code (FFI) or low-level operations requiring unsafety is absolutely necessary, it **must** be encapsulated within a tested, audited, third-party library (e.g., using tools like `bindgen`) rather than written directly in the Agent code. |

### 2.2. Error Management
Errors must be handled explicitly and gracefully.

| Rule | Directive |
| :--- | :--- |
| **Explicit Handling** | Systematically use **`Result<T, E>`** and **`Option<T>`** to model operations that may fail or return no value. |
| **Panic Prohibition** | **Strictly prohibit the use of `unwrap()`, `expect()`** or ignoring error results (`_`). These practices lead to runtime panics or silent failures. |
| **Propagation** | Propagate errors to the caller using the `?` operator or handle them gracefully to maintain program stability. |
| **Error Typing** | Use clear and informative error types. Use **enumerated errors** (e.g., via `thiserror`) when the caller needs to distinguish the exact cause of the error. Use **opaque errors** (e.g., via `anyhow` or `Box<dyn Error>`) for application code where fine-grained error distinction is not required. |

### 2.3. Security and Configuration

| Rule | Directive |
| :--- | :--- |
| **Input Validation** | **Always validate and sanitize all external inputs** to prevent security vulnerabilities such as SQL injection or cross-site scripting (XSS). Use Rust's type system to enforce strict input validation. |
| **Secret Management** | **Never hardcode secrets** (API keys, passwords, etc.) in the source code. Use environment variables or secure configuration management systems. Regularly rotate and update secrets. |
| **Secure Communication** | Implement secure communication protocols, such as **TLS** (or mTLS), ensuring proper authentication, confidentiality, and integrity for inter-service communication. **gRPC** is favored for synchronous inter-service communication due to its strongly typed contracts via Protocol Buffers. |

## 3. Quality Assurance and Tooling

### 3.1. Code Quality and Style

| Tool/Practice | Directive |
| :--- | :--- |
| **Clippy Enforcement** | **Mandatory** usage of Clippy. Configure Clippy to scan for common errors, inefficient code, and poor practices. **Must** be configured to trigger **compile-time errors** for critical warnings (e.g., unchecked unwraps). |
| **Rust Format** | **Mandatory** usage of `cargo format` with a specified `rustformat.toml` file to ensure a unified and consistent code style across the project. |
| **Reproducibility** | Lock the exact version of Rust (compilers, Clippy, Format) using a toolchain file to ensure build reproducibility across all environments. |

### 3.2. Testing and Dependencies

| Category | Directive |
| :--- | :--- |
| **Dependency Hygiene** | **Mandatory auditing** of dependencies in CI/CD. Use **Cargo Audit** to check for known vulnerabilities and **Cargo Deny** to enforce license rules and block unwanted crates. |
| **Advanced Testing** | Complement unit and integration tests with **property-based testing** (e.g., using `proptest`) to define properties that code must satisfy and automatically generate edge test cases. |
| **Performance Testing** | **Measure before optimizing**. Use statistical *benchmarking* tools (like `criterion`) to eliminate measurement noise. |
| **Build Optimization** | Use **Cargo Chef** in the CI/CD pipeline to cache dependencies, significantly accelerating compile times. |
| **Unsafe Testing (Encapsulated)** | If necessary (e.g., for verifying external FFI wrappers), use **Miri** to detect undefined behavior and memory safety issues in `unsafe` code blocks, even though direct Agent use of `unsafe` is prohibited. |

### 3.3. Deployment and Observability

| Domain | Directive |
| :--- | :--- |
| **Monitoring** | Implement monitoring using tools like **Prometheus** for metric collection and **OpenTelemetry** for distributed tracing. |
| **Resilience** | Implement resilience patterns such as **retry policies** (often with exponential backoff) and **circuit breakers** to handle unexpected failures. |
| **Graceful Degradation** | Design the Agent for **graceful degradation**, ensuring essential features remain functional even if non-critical components fail (e.g., using cached data if real-time data is unavailable). |