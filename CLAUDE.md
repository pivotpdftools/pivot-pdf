# CLAUDE.md

This file provides guidance to Claude when working with code in this repository.

## Project Purpose

This project provides a PDF creation library with multiple language bindings. The library offers both low-level PDF functions and high-level functions for formatting and fitting text, graphics, and images.

### Use Cases
Low memory and CPU consumption, even for documents with hundreds of pages, makes this ideal for SaaS/web applications:
- Reports
- Documents (contracts, invoices, bills of material, etc)

## Language & Framework
- Rust (stable channel)
- Workspace-based project structure

## Workspace Architecture
The workspace is organized as:

- **pdf-core** - Main PDF library (core functionality)
- **pdf-cli** - CLI tool that uses the core library to generate PDFs
- **pdf-php** - (future) PHP extension exposing core library functions

## Code Style
- Ideal max line length: 100 characters (hard max: 120)
- Follow Rust standard formatting (cargo fmt)
- Use Result<T, E> for fallible operations
- Prefer the ? operator over explicit matching when appropriate

## Clean Code Standards
Apply these principles consistently:
- **Functions:** Single responsibility, max ~50 lines, descriptive names
- **Variables:** Meaningful names (no abbreviations unless standard: `idx`, `ptr`)
- **Nesting:** Max 3 levels deep (use early returns, extract functions)
- **DRY:** Extract repeated logic into functions
- **Comments:** Explain *why*, not *what* (code should be self-documenting)

## Regular Refactoring Cadence
- **After each task:** Quick scan for immediate improvements
- **After completing an issue:** Review all changes for refactoring opportunities before marking complete
- **When adding features:** If touching existing code, leave it cleaner than you found it (Boy Scout Rule)
- **Red flags to address:** Long functions, duplicate code, complex conditionals, unclear names

## Directory Structure
- Each workspace member contains `src/` and `tests/` directories

## PDF Specification Reference
- **Full spec:** `docs/PDF32000_2008.pdf` (ISO 32000-1:2008 - PDF 1.7)
  - 22 MB, too large to load into context
  - Use as reference for edge cases only
  - Ask user to fetch specific sections when needed
- **Working reference:** `docs/pdf-reference-curated.md`
  - Condensed reference covering key sections for this library
  - Load this when needing spec details

## Testing Standards
- Test only public APIs (not private functions)
- Run `cargo test` before marking tasks complete
- Both test-first and test-last approaches are acceptable
- All tests must pass before committing

## Cross-Component Changes
When modifying pdf-core's public API:
1. Update the PHP extension bindings
2. Update pdf-php.stubs.php accordingly
3. Ensure all workspace components still build

## Working on Issues
- Consult ISSUES.md for issue descriptions, tasks, and statuses
- Consult files located in `/docs` to learn how the library works
- Only work on issues marked as ready
- Work on one issue at a time
- Update status to in-progress when starting work

## Planning & Execution
- If an issue lacks tasks, break it down into concrete tasks and update the issue before beginning work
- Don't make too many assumptions—ask the user if you have questions
- Complete all tasks for an issue before moving on

## Completion Criteria
- All tasks must be completed
- Tests must be written and passing
- Status marked as complete
- Committed to develop branch

## Before Moving to Next Issue
- Await user confirmation before starting a new issue

## Domain Knowledge Documentation

### Documentation Structure
The `docs/` directory contains domain knowledge that explains the "why" and "how" of the system:
- **architecture/** - System design decisions and core concepts
- **features/** - How major features work and their design rationale
- **configuration/** - Configuration options, their purpose, and impact

### Using Documentation
- Reference relevant docs when working on related features
- If unclear about design decisions, check docs before asking user
- User may point you to specific docs: "see docs/features/text-layout.md"

### Maintaining Documentation

## Living Documentation Philosophy

Documentation must evolve with the code. When modifying existing features:

1. **Read the existing doc first** - Understand current behavior and design decisions
2. **Update the doc as you code** - Don't wait until after
3. **Add to the history** - Briefly note what changed and why
4. **Update current behavior** - Keep the "how it works now" section current
5. **Preserve rationale** - Don't delete old design decisions, add new ones

**Anti-pattern:** Creating new docs for modifications. Instead, evolve existing docs.

**Why this matters:** 
- Prevents knowledge fragmentation
- Maintains historical context
- Makes future changes easier
- Eliminates "archaeology" through tickets/commits

**When to Update Documentation:**
- Adding a new feature → Create or update relevant feature doc
- Changing behavior → Update affected feature/architecture docs
- Adding configuration options → Document in configuration/ with rationale
- Making architectural decisions → Document why in architecture/

**What to Include in Feature Docs:**
- **Purpose:** Why this feature exists, what problem it solves
- **How it works:** High-level explanation (not code-level detail)
- **Design decisions:** Why this approach over alternatives
- **Configuration:** Related settings and their effects
- **Limitations:** Known constraints or edge cases
- **Examples:** Real-world usage scenarios

**Documentation Quality Standards:**
- Write for someone learning the system, not just maintaining it
- Explain the "why" behind non-obvious decisions
- Include diagrams for complex concepts (ASCII art is fine)
- Keep examples concrete and realistic
- Update related docs when making changes

## Code Review & Knowledge Transfer
Documentation serves as the primary mechanism for code review and knowledge transfer:
- User reviews documentation to understand what was built
- Documentation explains design decisions that aren't obvious from code
- Future sessions (human or AI) use documentation to understand existing systems

This is why documentation quality matters and why it's non-negotiable.

### Documentation is Part of the Task
Creating or updating domain documentation is required when:
- Implementing new features
- Changing system behavior
- Adding configuration options
- Making architectural changes

If domain docs aren't updated, the task isn't complete.

## Build & Run Commands

```bash
cargo build                # Debug build
cargo build --release      # Release build
cargo run                  # Build and run 
cargo check                # Type-check without building
cargo test                 # Run all tests
cargo fmt                  # Format code
```