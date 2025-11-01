<!--
Sync Impact Report:
- Version change: 1.1.0 → 1.2.0
- List of modified principles: None
- Added sections:
  - Principle VII. Semantic Markup for AI-First Development
- Removed sections: None
- Templates requiring updates:
  - ✅ .specify/templates/plan-template.md
- Follow-up TODOs: None
-->
# Trogue Constitution

## Core Principles

### I. CLI-First Interface
The core functionality of Trogue MUST be exposed through a command-line interface. The CLI design SHOULD prioritize ease of use, clarity, and consistency.

### II. Performance-focused
Trogue MUST be fast and efficient. All contributions SHOULD be mindful of performance implications, aiming to minimize resource usage (CPU, memory) without sacrificing correctness.

### III. Extensibility
The architecture MUST be modular to allow for future extensions. New features, such as support for new gaming platforms or data sources, SHOULD be implemented in a way that is decoupled from the core application logic.

### IV. Test-Driven Development
All new features MUST be accompanied by a comprehensive suite of automated tests. Development SHOULD follow a Test-Driven Development (TDD) approach where appropriate.

### V. Clear and Concise Code
The codebase MUST be readable, maintainable, and well-documented. Code SHOULD adhere to Rust best practices and community style guides.

### VI. Up-to-Date Documentation
Before using a library or framework, its up-to-date documentation MUST be checked using the Context7 MCP server. This ensures that the project relies on the latest and most accurate information, avoiding deprecated features and adopting best practices.

### VII. Semantic Markup for AI-First Development
A semantic markup block is a formal, XML-like specification placed in a documentation comment (docstring) directly above a function or class. It serves as a detailed "brief" for the AI, providing a clear contract for what the code should do, how it should do it, and how to verify its correctness.

This structured approach is more robust than informal instructions, enabling the AI to generate higher-quality, context-aware code with greater reliability.

#### Semantic Markup Rules

- The comments describing the purpose of each changed or updated logical block of code, function or class should be added when applying those changes. These comments should focus on documenting the reasons behind the code, not the implementation details - i.e. focus should be on WHY, not HOW. The documenting comment should contain sections: purpose, inputs, outputs, and side effects.
- These comments should be added to all new or modified code blocks, ensuring clarity and maintainability for future developers. When modifying existing code, please ensure to read and understand the existing doc comments, and make sure that updates won't break the contract established by those comments. If there are no such comments in the existing code, please add them as you are applying changes.
- Sections should be clearly formatted with XML-like tags or sections for easier reading and parsing. Best practice is to add such comment to each logical block of the code, like class, function or component. This structure provides better anchors by using clear unique (in the scope of each doc comment) delimitiers that improves readability and provides additional embedded context to help with understanding the implmentation and making changes that will take into account existing architecture, implementation and any potential side-effects of changes in a particular code block.

Example of the comment for Rust:
```rust
    // Draws the content of a plain text file to the terminal.
    //
    // <purpose-start>
    // This function is responsible for rendering the text content within the
    // specified area of the terminal. It also handles the display of the cursor
    // when in Edit mode, which is crucial for providing visual feedback to the
    // user about their current position in the text.
    // <purpose-end>
    //
    // <inputs-start>
    // - `frame`: The `Frame` to draw on.
    // - `area`: The `Rect` in which to draw the content.
    // - `content`: The string slice of the content to be rendered.
    // - `mode`: The current `Mode` of the application, which determines whether
    //   to display the cursor.
    // <inputs-end>
    //
    // <outputs-start>
    // - `Ok(())` if the drawing was successful.
    // - `Err(anyhow::Error)` if an error occurs during drawing.
    // <outputs-end>
    //
    // <side-effects-start>
    // - **Draws to the terminal**: The content is rendered to the screen.
    // - **Sets the cursor position**: If in Edit mode, the cursor is positioned
    //   at the appropriate location.
    // <side-effects-end>
```

## Additional Constraints

Trogue is to be developed exclusively in Rust, leveraging its performance, safety, and concurrency features.

## Development Workflow

Development will be conducted using Git and hosted on a platform that supports pull requests (e.g., GitHub). All changes MUST be submitted via pull requests and require at least one approval from a core contributor before being merged.

## Governance

This constitution is the supreme governing document for the Trogue project. All development practices, contributions, and reviews must align with its principles. Amendments to this constitution require a formal proposal, review, and majority approval from the core contributors.

**Version**: 1.2.0 | **Ratified**: 2025-10-10 | **Last Amended**: 2025-10-10