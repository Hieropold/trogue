# Writing Code Instructions

## Semantic markup doc comments
- The comments describing the purpose of each changed or updated logical block of code, function or class should be added when applying those changes. These comments should focus on documenting the reasons behind the code, not the implementation details - i.e. focus should be on WHY, not HOW. The documenting comment should contain sections: purpose, inputs, outputs, and side effects.
- These comments should be added to all new or modified code blocks, ensuring clarity and maintainability for future developers. When modifying existing code, please ensure to read and understand the existing doc comments, and make sure that updates won't break the contract established by those comments. If there are no such comments in the existing code, please add them as you are applying changes.
- Sections should be clearly formatted with XML-like tags or sections for easier reading and parsing. Best practice is to add such comment to each logical block of the code, like class, function or component. This structure provides better anchors by using clear unique (in the scope of each doc comment) delimitiers that improves readability and provides additional embedded context to help with understanding the implmentation and making changes that will take into account existing architecture, implementation and any potential side-effects of changes in a particular code block.

### Semantic markup doc comment example
```rust
    /// Draws the content of a plain text file to the terminal.
    ///
    /// <purpose-start>
    /// This function is responsible for rendering the text content within the
    /// specified area of the terminal. It also handles the display of the cursor
    /// when in Edit mode, which is crucial for providing visual feedback to the
    /// user about their current position in the text.
    /// <purpose-end>
    ///
    /// <inputs-start>
    /// - `frame`: The `Frame` to draw on.
    /// - `area`: The `Rect` in which to draw the content.
    /// - `content`: The string slice of the content to be rendered.
    /// - `mode`: The current `Mode` of the application, which determines whether
    ///   to display the cursor.
    /// <inputs-end>
    ///
    /// <outputs-start>
    /// - `Ok(())` if the drawing was successful.
    /// - `Err(anyhow::Error)` if an error occurs during drawing.
    /// <outputs-end>
    ///
    /// <side-effects-start>
    /// - **Draws to the terminal**: The content is rendered to the screen.
    /// - **Sets the cursor position**: If in Edit mode, the cursor is positioned
    ///   at the appropriate location.
    /// <side-effects-end>
```