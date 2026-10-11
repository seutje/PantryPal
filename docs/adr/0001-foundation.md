# ADR 0001: Foundation tooling and native boundary

Date: 2026-10-11

Status: Accepted

## Context

Phase 0 needs a small runnable Windows shell, repeatable checks and explicit module boundaries without starting the database or importer phases. DESIGN.md specifies React, Tailwind, accessible primitives, TanStack Query and Rust services with structured errors.

## Decision

Use a root Cargo workspace containing `src-tauri` and a root npm/Vite project. Pin Rust in `rust-toolchain.toml`, require Node 24/npm 11, and commit Cargo/npm lockfiles. Use Tailwind's Vite integration, ESLint flat configuration, Prettier, strict TypeScript, Vitest/Testing Library and Rust's built-in test runner. Native semantic buttons and links are the equivalent accessible primitives for the foundation; introduce a specialized accessible primitive library when complex controls require it.

The five sections use local shell state; no route library or persisted navigation is necessary yet. TanStack Query handles native reads. A browser preview honestly reports the absence of the desktop connection. Feature pages are placeholders, and the importer and persistence modules contain only boundary documentation.

IPC is a named typed frontend function backed by a single read-only Rust identity command. Future fallible commands return `AppResult<T>` with a stable code and safe message. Source errors must be handled within Rust without exposing sensitive details to UI.

Register application commands with `tauri_build::AppManifest::commands` so they are ACL controlled. Explicitly select one capability restricted to local content in the `main` window, granting only `allow-get-app-info`. No default core permission set and no remote origins are granted. CSP blocks remote scripts, frames and network in production; development permits only the Vite loopback endpoint in addition to IPC.

## Consequences and verification

The shell cannot read files, execute processes, fetch URLs or change stored data. New commands need explicit manifest entries, grants and tests. Static capability checks guard configuration drift but cannot substitute for the isolated remote-page runtime spike required in Phase 6. SQLx, database paths, conversion rules and importer mechanisms remain decisions for their planned phases.

The toolchain and lockfiles make checkouts repeatable; upgrades require deliberate revalidation. Current official references reviewed on 2026-10-11: [Tauri capabilities and custom-command manifest](https://v2.tauri.app/security/capabilities/), [Tauri permissions](https://v2.tauri.app/security/permissions/), [Windows prerequisites](https://v2.tauri.app/start/prerequisites/), [Tailwind Vite setup](https://tailwindcss.com/docs/installation/using-vite).
