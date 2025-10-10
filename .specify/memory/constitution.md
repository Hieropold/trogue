<!--
Sync Impact Report:
- Version change: 1.0.0 → 1.1.0
- List of modified principles: None
- Added sections:
  - Principle VI. Up-to-Date Documentation
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

## Additional Constraints

Trogue is to be developed exclusively in Rust, leveraging its performance, safety, and concurrency features.

## Development Workflow

Development will be conducted using Git and hosted on a platform that supports pull requests (e.g., GitHub). All changes MUST be submitted via pull requests and require at least one approval from a core contributor before being merged.

## Governance

This constitution is the supreme governing document for the Trogue project. All development practices, contributions, and reviews must align with its principles. Amendments to this constitution require a formal proposal, review, and majority approval from the core contributors.

**Version**: 1.1.0 | **Ratified**: 2025-10-10 | **Last Amended**: 2025-10-10