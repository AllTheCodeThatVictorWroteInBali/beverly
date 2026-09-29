# File Input

The `FileInput` lets a user select a file from their device.

It follows the same binding model as every other Beverly form control: the input collects a value, the binding connects it to the Model, and the Model decides what that value means.

A file input adds a few concerns that ordinary text inputs do not have:

- Where the selected file should be saved or uploaded.
- Which file types are accepted.
- What happens when a file is selected.
- How the selected file is represented in application state.
- Optional file-size and validation constraints.
- Progress and submission state for operations that process the file.

The UI component should remain simple. It provides the interface for selecting a file; the application decides what to do with it.

> **The File Input selects. The binding connects. The Model owns the file state.**

---

## Basic Usage

A file input can be created with `file_input()` and configured using the same fluent API as other Beverly inputs.

```rust
file_input()
    .label(localize("document.file.label"))
    .aria(localize("document.file.description"))
    .placeholder(localize("document.file.placeholder"))
    .bind(Document::file)
```

The important pieces are familiar:

- `.label(...)` gives the control its accessible name.
- `.aria(...)` provides additional accessible context.
- `.placeholder(...)` provides guidance before a file has been selected.
- `.bind(...)` connects the input to the Model.

The File Input does not need to know what the application intends to do with the file.

It might be uploaded to a server, stored locally, processed immediately, passed to an AI model, imported into a database, or written to a data lake.

That decision belongs to the application.

---

# File State

A file is not just a string.

The Model should normally represent a selected file using an application type that contains the information the application actually needs.

For example:

```rust
struct FileSelection {
    name: String,
    size: u64,
    mime_type: String,
}
```

The Model might then contain:

```rust
struct Document {
    file: Option<FileSelection>,
}
```

The File Input binds to that state:

```rust
file_input()
    .label(localize("document.file.label"))
    .aria(localize("document.file.description"))
    .placeholder(localize("document.file.placeholder"))
    .bind(Document::file)
```

The important distinction is that the File Input does not become the owner of the file.

The Model owns the application state.

---

# Accepted File Types

Applications can specify which file types the input accepts.

```rust
file_input()
    .label(localize("document.file.label"))
    .aria(localize("document.file.description"))
    .placeholder(localize("document.file.placeholder"))
    .accept([
        "application/pdf",
        "text/plain",
        "text/markdown",
    ])
    .bind(Document::file)
```

Extensions can also be useful when the application's vocabulary is extension-based:

```rust
file_input()
    .label(localize("document.file.label"))
    .aria(localize("document.file.description"))
    .accept([
        ".pdf",
        ".txt",
        ".md",
    ])
    .bind(Document::file)
```

The accepted types describe what the UI should offer as valid input.

They are not a substitute for Model validation.

The Model should still validate the file when it receives it.

> **UI constraints guide input. Model validation protects application state.**

This is particularly important when files can enter the application through other sources such as APIs, automated jobs, agents, or local processes.

---

# File Location

A File Input can optionally specify where the selected file should be saved or staged.

```rust
file_input()
    .label(localize("document.file.label"))
    .aria(localize("document.file.description"))
    .placeholder(localize("document.file.placeholder"))
    .location("/documents")
    .bind(Document::file)
```

The location is an application-level destination rather than part of the file's identity.

For example:

```rust
file_input()
    .location("/imports")
```

could mean:

> When the user selects a file, make it available to the application's import pipeline at `/imports`.

The exact behavior of the location depends on the application.

A local desktop application might save directly to a filesystem location. Another application might use the location as a staging directory before uploading the file. An application could also map the location to object storage or another persistence layer.

The File Input provides the configuration point. The application owns the actual storage behavior.

---

# Placeholder

The placeholder provides guidance before a file has been selected.

```rust
file_input()
    .label(localize("document.file.label"))
    .aria(localize("document.file.description"))
    .placeholder(localize("document.file.placeholder"))
    .bind(Document::file)
```

For example, the localized value might be:

```text
Select a PDF or Markdown file
```

The placeholder should describe what the user can do rather than becoming the primary accessible name.

The `.label(...)` remains the control's name.

---

# Labels and Accessibility

A File Input is a form control, so accessibility is part of its basic construction.

Every File Input should have both a label and accessible description.

```rust
file_input()
    .label(localize("document.file.label"))
    .aria(localize("document.file.description"))
    .bind(Document::file)
```

For example:

```rust
file_input()
    .label(localize("document.file.label"))
    .aria(localize("document.file.description"))
    .placeholder(localize("document.file.placeholder"))
```

The label identifies the control.

The ARIA description can provide additional context such as supported formats, size restrictions, or what happens after selection.

For example:

```text
Label:
Upload document

ARIA:
Select a PDF, Markdown, or text document. Maximum size 25 MB.
```

The important information should remain available through the control's accessible semantics rather than relying only on visual placeholder text.

> **Accessible by construction, not accessible by cleanup.**

---

# File Size

File size is an important constraint for file inputs.

A UI can communicate a maximum size:

```rust
file_input()
    .label(localize("document.file.label"))
    .aria(localize("document.file.description"))
    .max_size(25 * 1024 * 1024)
    .bind(Document::file)
```

But the Model remains responsible for deciding whether the file is actually valid.

For example:

```rust
impl Document {
    fn set_file(&mut self, file: FileSelection) -> Result<(), DocumentError> {
        if file.size > 25 * 1024 * 1024 {
            return Err(DocumentError::FileTooLarge);
        }

        self.file = Some(file);

        Ok(())
    }
}
```

The input can prevent obviously invalid selections from reaching the application, while the Model provides the authoritative validation.

This keeps the same architecture used by text inputs, selects, checkboxes, and other controls.

---

# Binding

`.bind()` works the same way as other Beverly form controls.

```rust
file_input()
    .label(localize("document.file.label"))
    .aria(localize("document.file.description"))
    .bind(Document::file)
```

Conceptually:

```text
File Selection
      ↓
   Binding
      ↓
 Model Setter
      ↓
Application State
```

When a user selects a file, the binding passes the file to the Model.

The setter can then:

- validate the file type
- validate the file size
- normalize metadata
- save or stage the file
- initiate processing
- reject the file
- update application state

The File Input does not need to implement those rules itself.

---

# File Validation

Validation belongs in the Model.

For example:

```rust
impl Document {
    fn set_file(&mut self, file: FileSelection) -> Result<(), DocumentError> {
        if !is_supported_type(&file.mime_type) {
            return Err(DocumentError::UnsupportedFileType);
        }

        if file.size > MAX_FILE_SIZE {
            return Err(DocumentError::FileTooLarge);
        }

        self.file = Some(file);

        Ok(())
    }
}
```

The UI can provide an accepted-type hint:

```rust
.accept([
    ".pdf",
    ".md",
    ".txt",
])
```

while the Model remains authoritative.

This means the same validation applies regardless of where the file came from.

A human selecting a file and an agent submitting a file should encounter the same application rules.

> **The UI suggests. The Model decides.**

---

# File Errors

File validation errors can be presented using the same `.error(...)` mechanism used by other form controls.

```rust
file_input()
    .label(localize("document.file.label"))
    .aria(localize("document.file.description"))
    .bind(Document::file)
    .error(
        self.children([
            text(|errors: &[String]| errors.join(" ")),
        ])
    )
```

The UI does not need to hard-code validation messages.

The Model produces the semantic error, and the presentation layer determines how that error is displayed and localized.

> **The Model returns meaning. The UI presents language.**

---

# Submitting State

File processing may take time.

For example, selecting a file might initiate:

- a local copy
- parsing
- indexing
- uploading
- virus scanning
- AI processing
- document extraction

The File Input can respond to the application's submitting state:

```rust
file_input()
    .label(localize("document.file.label"))
    .aria(localize("document.file.description"))
    .bind(Document::file)
    .submitting(self.opacity(0.5))
```

The component does not need to know why the application is submitting.

It simply provides a presentation for that state.

---

# Success and Error Presentation

The File Input can provide state-specific presentation hooks:

```rust
file_input()
    .label(localize("document.file.label"))
    .aria(localize("document.file.description"))
    .bind(Document::file)
    .submitting(self.opacity(0.5))
    .error(self.children([
        text(|errors: &[String]| errors.join(" ")),
    ]))
    .success(self.opacity(1.0))
```

These hooks describe how the component should appear when the associated state exists.

They do not create or own that state.

> **State determines when. Components determine what.**

---

# Multiple Files

Some applications need a single file. Others need a collection.

A File Input can support multiple selection when the application requires it:

```rust
file_input()
    .label(localize("documents.files.label"))
    .aria(localize("documents.files.description"))
    .placeholder(localize("documents.files.placeholder"))
    .multiple(true)
    .accept([
        ".pdf",
        ".md",
        ".txt",
    ])
    .bind(DocumentImport::files)
```

The Model can then own a collection:

```rust
struct DocumentImport {
    files: Vec<FileSelection>,
}
```

The same principles apply.

The UI collects the selection.

The binding connects it.

The Model owns the resulting state.

---

# File Processing Is Not File Input

The File Input should not become an upload framework.

For example, these are separate concerns:

```text
File Input
    ↓
File Selection
    ↓
Model
    ↓
Processing / Upload / Import
```

Selecting a file is one operation.

Uploading it is another.

Parsing it is another.

Indexing it is another.

Sending it to an AI model is another.

Beverly's architecture keeps those responsibilities separate.

For example:

```rust
controller! {
    Document::Selected => [
        Document::validate,
        Document::store,
        Document::index,
    ],
}
```

The File Input does not need to understand that pipeline.

It simply reports the user's selection.

---

# File Input Events

The File Input can expose UI lifecycle Events when the application needs to observe interaction.

For example:

```rust
file_input()
    .label(localize("document.file.label"))
    .aria(localize("document.file.description"))
    .on("select", Ui::DocumentFile::Selected)
    .bind(Document::file)
```

The UI Event describes what happened at the interface.

The application Command describes what should happen as a result.

This preserves the same distinction used throughout Beverly:

```text
UI Event
    ↓
Controller
    ↓
Command
    ↓
Model
```

The File Input does not decide what selecting a file means.

---

# Data-Driven File Inputs

File inputs can be created by ordinary Rust functions.

For example:

```rust
fn document_file_input() -> FileInput {
    file_input()
        .label(localize("document.file.label"))
        .aria(localize("document.file.description"))
        .placeholder(localize("document.file.placeholder"))
        .accept([
            ".pdf",
            ".md",
            ".txt",
        ])
        .max_size(25 * 1024 * 1024)
        .bind(Document::file)
}
```

Then:

```rust
document_file_input()
```

This is not a second component system.

It is simply ordinary Rust returning a Beverly component.

> **Higher-level components are ordinary Rust functions that return lower-level components.**

---

# Styling

File Inputs use the same fluent styling model as other Beverly components.

```rust
file_input()
    .label(localize("document.file.label"))
    .aria(localize("document.file.description"))
    .padding(12)
    .radius(8)
    .border_width(1)
    .width(400)
```

Application-specific presentation can be built on top:

```rust
fn document_uploader() -> FileInput {
    file_input()
        .label(localize("document.file.label"))
        .aria(localize("document.file.description"))
        .placeholder(localize("document.file.placeholder"))
        .accept([".pdf", ".md"])
        .max_size(25 * 1024 * 1024)
        .padding(16)
        .radius(12)
        .bind(Document::file)
}
```

The primitive remains simple while higher-level components can establish the application's visual language.

---

# What File Input Does Not Do

The File Input does not own:

- application state
- business rules
- file validation policy
- upload policy
- authentication
- authorization
- database state
- AI processing
- document parsing
- indexing
- application workflow

Those responsibilities belong elsewhere.

The File Input has one clear job:

> **Provide an accessible interface for selecting files and connect that selection to application state.**

---

# File Input and the Model

The complete relationship is straightforward:

```text
User
  ↓
File Input
  ↓
Binding
  ↓
Model Setter
  ↓
Application State
```

The Model can then trigger whatever the application needs.

```text
File Input
    ↓
Document::file
    ↓
Validation
    ↓
Storage
    ↓
Processing
    ↓
Indexing
    ↓
Application
```

The File Input remains unaware of the pipeline.

That separation is what makes it reusable.

---

# File Input vs Text Input

|                  | Text Input  | File Input   |
| ---------------- | ----------- | ------------ |
| Primary value    | Text        | File / files |
| `.bind()`        | Yes         | Yes          |
| `.label()`       | Yes         | Yes          |
| `.aria()`        | Yes         | Yes          |
| `.placeholder()` | Yes         | Yes          |
| Validation       | Model       | Model        |
| Errors           | Model → UI  | Model → UI   |
| Accepted types   | —           | Yes          |
| File size        | —           | Yes          |
| Multiple values  | —           | Optional     |
| Processing       | Application | Application  |

The architecture stays the same even though the value is different.

---

# The Principle

File Input does not need to become complicated simply because files are more complicated than strings.

The same Beverly model applies:

**Create. Configure. Compose. Bind. Declare state. Emit intent.**

The File Input provides the file-selection interface.

The binding connects it to the Model.

The Model validates and owns the resulting state.

The Controller decides what happens next.

And the rest of the application can process the file however it needs.

> **The File Input selects. The Binding connects. The Model validates. The Controller orchestrates. The application decides what the file means.**
