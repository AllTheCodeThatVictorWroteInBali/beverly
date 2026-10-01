# Table

The `Table` is Beverly's primitive for displaying tabular data.

At its core, a Table is simply a two-dimensional collection of values:

```text
Table
  ├── Row
  │    ├── Cell
  │    ├── Cell
  │    └── Cell
  │
  ├── Row
  │    ├── Cell
  │    ├── Cell
  │    └── Cell
  │
  └── ...
```

The API intentionally stays simple.

Rows are represented as an array of values, and the table is therefore a 2D array:

```rust
[
    ["Name", "Email", "Status"],
    ["Alice", "alice@example.com", "Active"],
    ["Bob", "bob@example.com", "Pending"],
]
```

This makes the primitive easy for both humans and AI systems to construct, inspect, transform, and reason about.

> **A Table is a two-dimensional array of values.**

---

# Basic Usage

A basic table can be created by passing rows:

```rust id="8s2m1k"
table([
    ["Name", "Email", "Status"],
    ["Alice", "alice@example.com", "Active"],
    ["Bob", "bob@example.com", "Pending"],
])
```

Each inner array represents one row.

Each value within that row represents one cell.

The structure is therefore directly reflected in the code:

```text
[
    [cell, cell, cell],
    [cell, cell, cell],
    [cell, cell, cell],
]
```

There is no separate row model or table state system required.

---

# Rows

Rows are arrays of values.

```rust id="p6x4wd"
[
    ["Alice", "alice@example.com", "Active"],
    ["Bob", "bob@example.com", "Pending"],
    ["Charlie", "charlie@example.com", "Inactive"],
]
```

The first array is the first row.

The second array is the second row.

And so on.

This makes a Table naturally compatible with data that already exists as collections.

For example:

```rust id="q7v3ka"
users.iter()
    .map(|user| [
        user.name(),
        user.email(),
        user.status(),
    ])
```

The Table does not require the application to transform its data into a special Beverly table format.

> **Use your data. Don't build a second data model just to display it.**

---

# Cells

Every value in a row becomes a cell.

Cells can be styled individually when the application needs more control.

```rust id="z4n8rc"
table()
    .cell(0, 0)
    .color(red)
```

The coordinates identify the cell:

```text
cell(row, column)
```

For example:

```rust id="v5k2qa"
table()
    .cell(2, 1)
    .color(red)
```

targets the cell at row `2`, column `1`.

The important distinction is that the Table owns the overall structure while the Cell provides the individual presentation.

---

# Cell Styling

Individual cells can be configured independently.

```rust id="r3m7yf"
table()
    .cell(1, 2)
    .color(red)
```

Multiple properties can be chained:

```rust id="c9w4hx"
table()
    .cell(1, 2)
    .color(red)
    .padding(8)
    .radius(4)
```

This makes cell-level styling explicit.

You don't need a separate stylesheet or selector language to say which cell should look different.

The cell is directly addressable.

> **Style the cell where you use the cell.**

---

# Table Styling

The Table itself can be styled using the same fluent API as other Beverly components.

```rust id="m2p8vb"
table([
    ["Name", "Email", "Status"],
    ["Alice", "alice@example.com", "Active"],
])
    .padding(8)
    .radius(12)
```

Table-level styling establishes the presentation of the overall table.

Cell-level styling establishes the presentation of individual cells.

```text
Table styling
      ↓
Overall presentation

Cell styling
      ↓
Individual presentation
```

---

# Headers

A table can represent its header as the first row.

```rust id="f6t1qx"
table([
    ["Name", "Email", "Status"],
    ["Alice", "alice@example.com", "Active"],
    ["Bob", "bob@example.com", "Pending"],
])
```

This keeps the underlying representation simple.

The application can establish that the first row is a header through table configuration or a higher-level component.

For example:

```rust id="k8d3ns"
table([
    ["Name", "Email", "Status"],
    ["Alice", "alice@example.com", "Active"],
    ["Bob", "bob@example.com", "Pending"],
])
    .header(0)
```

The important point is that headers do not require a separate data structure.

They are still cells in the same two-dimensional representation.

---

# Data-Driven Tables

Tables are particularly useful when the data already exists as a collection.

For example:

```rust id="y2r6mc"
fn user_table(users: &[User]) -> Table {
    table(
        users.iter()
            .map(|user| [
                user.name(),
                user.email(),
                user.status(),
            ])
            .collect()
    )
}
```

The resulting component can then be used normally:

```rust id="e7q4jw"
user_table(&users)
```

This is ordinary Rust.

There is no special Beverly data-mapping language required.

> **Higher-level components are ordinary Rust functions that return Beverly components.**

---

# Conditional Cell Presentation

Because cells are independently configurable, applications can present values differently based on their data.

For example:

```rust id="u4k9ps"
for user in users {
    let status = user.status();

    let mut cell = table.cell(...);

    if status == "Active" {
        cell = cell.color(green);
    }
}
```

The exact implementation can vary depending on how the table is constructed, but the principle remains the same:

**Application data determines presentation.**

A higher-level component can encapsulate common presentation rules:

```rust id="h6n2xr"
fn status_cell(status: UserStatus) -> Cell {
    match status {
        UserStatus::Active =>
            cell("Active").color(green),

        UserStatus::Pending =>
            cell("Pending").color(yellow),

        UserStatus::Inactive =>
            cell("Inactive").color(gray),
    }
}
```

Then the table remains simple:

```rust id="w3p7qa"
table([
    ["Alice", status_cell(UserStatus::Active)],
    ["Bob", status_cell(UserStatus::Pending)],
])
```

The primitive provides the structure.

The application provides the meaning.

---

# Composition

A cell does not have to be limited to plain text.

Cells can contain Beverly components when the application needs richer presentation.

For example:

```rust id="n8v5kc"
table([
    [
        text("Alice"),
        badge(BadgeType::Success, "Active"),
    ],
    [
        text("Bob"),
        badge(BadgeType::Warning, "Pending"),
    ],
])
```

A cell might contain:

- Text
- Badges
- Icons
- Images
- Buttons
- Links
- Other composed components

This allows a Table to remain a primitive while still supporting sophisticated interfaces.

For example, an action cell could contain a Button:

```rust id="t5q1mz"
button().text(localize("user.edit"))
    .on("click", Command::User::Edit)
```

The Button remains responsible for interaction.

The Table remains responsible for tabular layout.

---

# Cell Interaction

A Table does not make every cell interactive.

A cell can simply display information:

```rust id="b7k3xp"
cell(text("Alice"))
```

Or it can contain an interactive component:

```rust id="c4m8yd"
cell(
    button().text(localize("user.edit"))
        .on("click", Command::User::Edit)
)
```

This follows a fundamental Beverly principle:

> **A container does not become an application just because it contains an interactive component.**

The Table provides the layout.

The component inside the Cell provides the interaction.

---

# Accessibility

Tables have semantic meaning, so accessibility should be part of their construction.

A simple table can provide an accessible label and description:

```rust id="x9q2vf"
table([
    ["Name", "Email", "Status"],
    ["Alice", "alice@example.com", "Active"],
])
    .label(localize("users.table.label"))
    .aria(localize("users.table.description"))
```

For example:

```text
Label:
Users

ARIA:
List of users with name, email, and account status.
```

When a table contains interactive controls, those controls remain independently accessible.

For example:

```rust id="j6w4nb"
button().text(localize("user.edit"))
```

The Table's accessibility describes the table.

The Button's accessibility describes the Button.

Each component owns its own semantic responsibility.

> **Accessibility follows composition.**

---

# Table State

The Table itself does not need to own application data.

For example, it should not become a second source of truth for users:

```text
Model
  ↓
Users
  ↓
Table
  ↓
Cells
```

The Table projects application data into a visual representation.

If a user's status changes in the Model, the table reflects the new value.

The Table does not independently decide what the user's status is.

This follows the same Model → View relationship used throughout Beverly.

> **The Table displays the data. The Model owns the data.**

---

# Selection

Selection is an application concern rather than an inherent requirement of a Table.

A higher-level table component can provide selection behavior when needed.

For example:

```rust id="s8d5qk"
selectable_table(users)
```

can be implemented as an ordinary Rust function that builds on the Table primitive.

The underlying Table remains simple.

This distinction is important because not every table needs:

- Row selection
- Cell selection
- Sorting
- Filtering
- Pagination
- Virtualization
- Editing

Those can be built as higher-level components when the application requires them.

---

# Tables and Data Grids

A basic Table should not be confused with a full data grid.

A Table provides:

- Two-dimensional data
- Rows
- Cells
- Cell composition
- Cell-level styling
- Table-level presentation

A higher-level data grid may additionally provide:

- Sorting
- Filtering
- Column resizing
- Column pinning
- Row selection
- Cell editing
- Virtualization
- Streaming updates
- Keyboard navigation
- Large dataset optimization

Those capabilities can be layered on top of the Table primitive rather than being required by it.

This keeps the foundational component small while allowing Beverly to build much more sophisticated data interfaces.

> **Start with the primitive. Add capability when the application needs it.**

---

# Large Data Sets

The same two-dimensional representation can serve as the conceptual foundation for much larger tables.

The important distinction is between the **data representation** and the **rendering strategy**.

The application can have a large dataset:

```text
Application Data
      ↓
Rows
      ↓
Table
      ↓
Visible Cells
      ↓
GPU
```

A higher-level data table can virtualize which rows and cells are actually rendered without changing the application's conceptual model.

The developer still reasons about rows and cells.

Beverly handles the rendering strategy underneath.

This allows the API to remain simple even when the implementation becomes highly optimized.

---

# Dynamic Updates

Because the Table is a View, it can react to changes in application state.

For example:

```rust
table(users.rows())
```

When the underlying Model changes, the View can produce the corresponding updated table representation.

This means applications can use Tables for:

- Live data
- Monitoring interfaces
- Financial data
- Logs
- AI agent activity
- Device telemetry
- Database results
- Search results
- Administrative interfaces

The Table does not need to know where the data originated.

It only needs a representation of rows and cells.

---

# Custom Table Components

A higher-level table is simply an ordinary Rust function.

For example:

```rust id="q5m8vz"
fn user_table(users: &[User]) -> Table {
    table(
        users.iter()
            .map(|user| [
                text(user.name()),
                text(user.email()),
                badge(user.status()),
            ])
            .collect()
    )
}
```

Another application can build a completely different table:

```rust id="r7k2yc"
fn transaction_table(transactions: &[Transaction]) -> Table {
    table(
        transactions.iter()
            .map(|transaction| [
                text(transaction.id()),
                text(transaction.amount()),
                badge(transaction.status()),
            ])
            .collect()
    )
}
```

Both are ordinary Rust functions returning a Beverly component.

There is no need to create a new framework abstraction for every domain-specific table.

> **Build the primitive once. Build domain-specific tables with ordinary Rust.**

---

# What Table Does Not Do

The foundational Table does not inherently own:

- Application data
- Database state
- Sorting state
- Filtering state
- Selection state
- Business rules
- Authorization
- Navigation
- Data fetching
- AI behavior

Those concerns belong to the application architecture.

A higher-level component can add any of these capabilities while still using the underlying Table.

---

# Table vs Data Grid

|                         | Table                      | Data Grid           |
| ----------------------- | -------------------------- | ------------------- |
| Basic rows and cells    | Yes                        | Yes                 |
| 2D array representation | Yes                        | Yes                 |
| Cell composition        | Yes                        | Yes                 |
| Cell-level styling      | Yes                        | Yes                 |
| Sorting                 | Application / higher-level | Built-in capability |
| Filtering               | Application / higher-level | Built-in capability |
| Selection               | Application / higher-level | Built-in capability |
| Editing                 | Application / higher-level | Built-in capability |
| Virtualization          | Implementation-dependent   | Core capability     |
| Large datasets          | Foundation                 | Optimized           |
| Streaming data          | Foundation                 | Optimized           |

The Table is the primitive.

The Data Grid is a higher-level application component built when the additional behavior is required.

---

# The Principle

The Table deliberately starts with a very small idea:

**Rows are arrays. Cells are values. The table is two-dimensional.**

Everything else builds from there.

The Table provides the structure.

Cells provide the individual presentation.

Components inside cells provide interaction.

The Model provides the data.

Higher-level components provide specialized behavior.

And ordinary Rust provides the mechanism for composing it all.

> **A Table is a two-dimensional array of values with composable cells.**

> **Simple enough to understand. Powerful enough to build a data interface on top of.**
