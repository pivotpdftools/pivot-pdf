# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Purpose

The purpose of this project is to provide a pdf creation library with multiple language bindings. The library will provide low level pdf functions and high level pdf functions which allow text, graphics, and images to be formatted and fit.

### Use Cases
The memory and cpu consumption is low, even for documents containing hundreds of pages making it ideal for saas/web applications.
- Reports
- Documents such as Contracts, Invoices, Bills of Material, etc

## Build & Run Commands

```bash
cargo build                # Debug build
cargo build --release      # Release build
cargo run                  # Build and run 
cargo check                # Type-check without building
cargo test                 # Run all tests
```

## Architecture

This workspace is organized as:
- pdf-core - This is the main pdf library
- pdf-cli - A Cli which uses the core to generate pdfs
- pdf-php - (future) php extentesion exposing the core library functions

## Code
- Ideal max line length is 100 characters. Hard max length is 120 characters.
- Directory Structure
  - Each section of the workspace contains a `src` and `tests` directory

## Working on Issues
- Consult the ISSUES.md file contains descriptions, tasks, and statuses of work to be done.
- Try not to make too many assumptions. Ask user if you have questions
- Work on 1 issue at a time
- Only work on an issue if the status is `ready`.
- When starting work on an issue, Agent should update the status to `in-progress`.
- Test first or Test last can be employed. Either way, the only way to ensure something is complete is when there are tests for it and the tests pass.
- If an Issue does not contain any Tasks, enter plan mode and update the issue with the lists of Tasks
- Once each task of the plan is complete and tests/verification are complete, mark the task as done and a commit to develop branch can be done.
- Once the Issue is complete, marke the status as complete
- The Agent make work on Tasks to completion but must get user confirmation to move to another Issue.