## Project Structure

The `biobit` project is organized as a Cargo workspace with several member crates, facilitating modular development and
clear separation of concerns:

* **`biobit/py`**: This is the primary Python package crate. It acts as an umbrella, integrating all Rust modules and
  exposing them as a unified Python package named `biobit`.
    * Its `src/bindings.rs` is the **central FFI definition point**. It constructs the main Python extension module (
      imported as `biobit.rs` in Python) and attaches all other Rust-backed submodules (e.g., `biobit.rs.core`,
      `biobit.rs.io`). This centralized approach is adopted to manage
      dependencies between PyO3 modules effectively (see `dev_notes.md` for background on PyO3 module dependency
      challenges).
* **`modules/`**: This directory houses the core Rust logic and their corresponding Python bindings. It is divided into:
    * **Functional Modules**: These provide fundamental bioinformatics capabilities (e.g., `core`, `io`, `alignment`,
      `collections`).
    * **Toolkit Modules**: These offer higher-level tools and pipelines (e.g., `countit`, `reaper`, `repeto`).
    * Each module under `modules/` (whether functional or toolkit) typically follows a consistent structure:
        * **`rs/`**: Contains the pure Rust implementation of the module's logic (e.g., `modules/core/rs/`). This crate
          focuses on performance and Rust-idiomatic APIs.
        * **`py/`**: Contains the PyO3 bindings for the corresponding `rs` crate (e.g., `modules/core/py/`). This crate
          is responsible for exposing Rust functionality to Python in a user-friendly way and links against the `rs`
          crate of the same module.
* **`resources/`**: This top-level directory contains test files, example data, and other static assets. See
  the [Resource Management](#resource-management) section below for details.

The overall workspace structure is defined in the main `biobit/Cargo.toml` file.

## Resource Management

Static assets like test files and example data are centralized in the top-level `resources/` directory to ensure easy
access and consistent organization. Commonly used data files (e.g., FASTA, BAM, BED) shared across multiple tests or
modules are also housed here. It's generally encouraged to reuse these files in individual modules
to avoid duplication and maintain consistency.

### Resources Inventory

A comprehensive inventory and description of all data files within the `resources/` folder **must** be
maintained in `resources/README.md`. This document serves as the definitive guide to understanding the purpose,
origin, and structure of each resource, ensuring they are discoverable and well-understood.

### Accessing Resources in Tests

Resources folder is defined via an environment variable in the project configuration, allowing tests from all languages
to access the same resources without hardcoding paths:

* **Rust**: `env!("BIOBIT_RESOURCES")`.
* **Python**: `os.environ['BIOBIT_RESOURCES']`.
