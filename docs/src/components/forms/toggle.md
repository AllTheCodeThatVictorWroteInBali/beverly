# Toggle Button

> **Current status:** `.bind(...)` examples describe planned model-setter
> integration and are not currently executable against the crate.

The `ToggleButton` is a form control that represents an on/off state while presenting itself as a button.

It is useful when the user is choosing between two states rather than simply activating an action.

Examples include:

- Bold / not bold
- Favorite / not favorite
- Mute / unmute
- Grid view / list view
- Active / inactive
- Show / hide
- Enabled / disabled

The Toggle Button combines two familiar ideas:

**Checkbox semantics + Button presentation.**

Like a Checkbox, it represents a boolean value.

Like a Button, it provides a visually prominent interactive surface.

> **The Toggle Button represents state. The binding connects it. The Model owns the state.**

---

# Basic Usage

A Toggle Button uses the same fluent API as other Beverly form controls.

```rust id="k4n7px"
toggle_button(localize("user.favorite"))
    .label(localize("user.favorite.label"))
    .aria(localize("user.favorite.description"))
    .bind(User::favorite)
```

The important pieces are familiar:

- `toggle_button(...)` establishes the control.
- `.label(...)` provides its accessible name.
- `.aria(...)` provides additional accessible context.
- `.bind(...)` connects the control to Model state.

The Model might contain:

```rust id="z8q3mv"
struct User {
    favorite: bool,
}
```

The Toggle Button simply reflects that value.

---

# Boolean State

A Toggle Button normally represents a boolean:

```rust id="p2w6kc"
struct User {
    notifications_enabled: bool,
}
```

The control binds directly to it:

```rust id="m9r4yd"
toggle_button(localize("user.notifications"))
    .label(localize("user.notifications.label"))
    .aria(localize("user.notifications.description"))
    .bind(User::notifications_enabled)
```

When the Model value is `true`, the Toggle Button presents its active state.

When the Model value is `false`, it presents its inactive state.

The UI does not maintain a second copy of the value.

> **One state. One source of truth.**

---

# Binding

Toggle Button binding follows the same contract as Checkbox.

Conceptually:

```text id="7y3kqm"
Model
  ↓
Binding
  ↓
Toggle Button

Toggle Button
  ↓
Binding
  ↓
Setter
  ↓
Model
```

When the Model changes, the Toggle Button reflects the new value.

When the user interacts with the Toggle Button, the binding passes the requested value to the Model setter.

The setter can accept or reject the change.

```rust id="d5n8vw"
impl User {
    fn set_notifications_enabled(
        &mut self,
        enabled: bool,
    ) -> Result<(), UserError> {
        self.notifications_enabled = enabled;

        Ok(())
    }
}
```

The Toggle Button does not decide whether the transition is valid.

The Model does.

---

# Validation

A boolean may look simple, but changing it can still involve application rules.

For example, enabling a feature might require permission:

```rust id="h6q2xr"
impl User {
    fn set_beta_enabled(
        &mut self,
        enabled: bool,
    ) -> Result<(), UserError> {
        if enabled && !self.can_use_beta() {
            return Err(UserError::PermissionDenied);
        }

        self.beta_enabled = enabled;

        Ok(())
    }
}
```

The Toggle Button does not need to know why the Model rejected the change.

It simply presents the result.

This is the same architecture used by every Beverly form control.

> **The UI requests the transition. The Model decides whether the transition is valid.**

---

# Active and Inactive States

A Toggle Button has two fundamental states:

```text id="n4c7zs"
Inactive
   ↕
Active
```

The application can style each state independently.

```rust id="w8m3kp"
toggle_button(localize("user.favorite"))
    .label(localize("user.favorite.label"))
    .aria(localize("user.favorite.description"))
    .active(self.color(red))
    .inactive(self.opacity(0.6))
    .bind(User::favorite)
```

Here, `self` refers to the Toggle Button being configured.

It does not refer to the Model or the Event that caused the state change.

The state determines **when** a presentation applies.

The component determines **what** that presentation looks like.

---

# Interaction States

The Toggle Button can also provide the normal UI lifecycle states of a button.

```rust id="q5r9vd"
toggle_button(localize("user.favorite"))
    .label(localize("user.favorite.label"))
    .aria(localize("user.favorite.description"))
    .hover(self.opacity(0.9))
    .focused(self.outline_width(2))
    .pressed(self.scale(0.98))
    .disabled(self.opacity(0.5))
    .bind(User::favorite)
```

This separates interaction state from application state.

For example:

- `hovered` describes pointer interaction.
- `focused` describes keyboard/input focus.
- `pressed` describes the physical interaction.
- `disabled` describes whether the control can currently be used.
- `active` describes the application's boolean state.

These are different concepts even though they affect presentation.

---

# Submitting State

A Toggle Button can also reflect a pending application operation.

```rust id="c7x4mn"
toggle_button(localize("user.notifications"))
    .label(localize("user.notifications.label"))
    .aria(localize("user.notifications.description"))
    .submitting(self.opacity(0.5))
    .bind(User::notifications_enabled)
```

For example, changing a setting might require a remote request or local processing.

The Toggle Button does not need to know what operation is taking place.

It simply reflects the application's submitting state.

---

# Error and Success

Toggle Buttons can use the same state presentation hooks as other Beverly form controls.

```rust id="v3m8qx"
toggle_button(localize("user.beta"))
    .label(localize("user.beta.label"))
    .aria(localize("user.beta.description"))
    .bind(User::beta_enabled)
    .submitting(self.opacity(0.5))
    .error(self.color(red))
    .success(self.color(green))
```

These states describe presentation.

They do not create application state.

If the Model rejects a requested transition, the error can be presented through the Toggle Button.

If the operation succeeds, the success presentation can be used.

---

# Accessibility

A Toggle Button is an interactive form control and should provide both a label and accessible description.

```rust id="x6p2kr"
toggle_button(localize("user.favorite"))
    .label(localize("user.favorite.label"))
    .aria(localize("user.favorite.description"))
    .bind(User::favorite)
```

The accessible semantics should also communicate the control's current state.

The user should be able to determine whether the toggle is currently active or inactive without relying exclusively on color or visual styling.

For example, a localized description might communicate:

```text id="r4n8yc"
Label:
Favorite

ARIA:
Mark this item as a favorite.
```

The active state should remain programmatically represented by the component rather than being conveyed only through a color change.

> **Accessible by construction, not accessible by cleanup.**

---

# Labels

Labels should describe what the boolean represents.

```rust id="j8w5qm"
toggle_button(localize("user.notifications"))
    .label(localize("user.notifications.label"))
    .aria(localize("user.notifications.description"))
    .bind(User::notifications_enabled)
```

A label such as:

```text
Notifications
```

is generally more useful than a label describing the implementation:

```text
Toggle notifications boolean
```

The label describes the user's concept.

The Model describes the application's data.

---

# Events

The Toggle Button can expose UI Events for interaction lifecycle when the application needs them.

For example:

```rust id="s3k9vp"
toggle_button(localize("user.favorite"))
    .label(localize("user.favorite.label"))
    .aria(localize("user.favorite.description"))
    .on("click_start", Ui::FavoriteToggle::Pressed)
    .on("click_end", Ui::FavoriteToggle::Released)
    .on("click", Ui::FavoriteToggle::Clicked)
    .bind(User::favorite)
```

These are UI Events.

They describe what happened at the interface.

The resulting state transition still goes through the binding and Model.

This keeps the distinction clear:

```text id="n7q4xc"
UI Event
    ↓
Binding
    ↓
Model Setter
    ↓
State
```

The Toggle Button does not need to know what the state change means to the application.

---

# Toggle Button vs Button

The difference between a normal Button and a Toggle Button is semantic.

A Button requests an action:

```rust id="f2m8qw"
button().text(localize("user.save"))
    .on("click", Command::User::Save)
```

A Toggle Button represents a state:

```rust id="c9v5mz"
toggle_button(localize("user.favorite"))
    .label(localize("user.favorite.label"))
    .aria(localize("user.favorite.description"))
    .bind(User::favorite)
```

The Button says:

> Something happened.

The Toggle Button says:

> This value is now active or inactive.

That distinction determines whether the control needs a boolean binding.

---

# Toggle Button vs Checkbox

A Toggle Button and Checkbox can represent the same underlying boolean.

The difference is primarily presentation and interaction vocabulary.

|                              | Checkbox  | Toggle Button |
| ---------------------------- | --------- | ------------- |
| Represents boolean state     | Yes       | Yes           |
| `.bind()`                    | Yes       | Yes           |
| `.label()`                   | Yes       | Yes           |
| `.aria()`                    | Yes       | Yes           |
| Active/inactive presentation | Yes       | Yes           |
| Button-like presentation     | No        | Yes           |
| Best for settings/forms      | Often     | Sometimes     |
| Best for toolbar controls    | Sometimes | Often         |
| Model owns state             | Yes       | Yes           |

The application should choose the control that best communicates the interaction.

The underlying Model does not need to change.

---

# Toggle Button vs Radio Button

Radio Buttons represent a choice among multiple mutually exclusive values.

A Toggle Button represents one boolean state.

For example:

```rust id="p4m8yd"
toggle_button(localize("view.grid"))
    .label(localize("view.grid.label"))
    .aria(localize("view.grid.description"))
    .bind(View::grid_enabled)
```

versus:

```rust id="z6q2nk"
radio_group()
    .label(localize("view.mode.label"))
    .aria(localize("view.mode.description"))
    .bind(View::mode)
```

The underlying concepts are different:

```text id="u9c5rx"
Toggle Button
    → true / false

Radio Group
    → one value from many choices
```

---

# Composition

The Toggle Button can contain ordinary Beverly components.

```rust id="k3w7qp"
toggle_button()
    .label(localize("view.grid.label"))
    .aria(localize("view.grid.description"))
    .children([
        icon("grid"),
        text(localize("view.grid")),
    ])
    .bind(View::grid_enabled)
```

This allows the visual representation to include:

- Icons
- Text
- Badges
- Images
- Other presentation components

The contents communicate the control visually.

The label and ARIA semantics communicate it accessibly.

---

# Styling

The Toggle Button uses the same fluent styling approach as other Beverly components.

```rust id="a7m4vx"
toggle_button(localize("view.grid"))
    .label(localize("view.grid.label"))
    .aria(localize("view.grid.description"))
    .padding(8)
    .radius(8)
    .width(100)
    .bind(View::grid_enabled)
```

State-specific presentation can then be layered on top:

```rust id="e2q8mc"
toggle_button(localize("view.grid"))
    .label(localize("view.grid.label"))
    .aria(localize("view.grid.description"))
    .padding(8)
    .radius(8)
    .active(self.color(blue))
    .inactive(self.opacity(0.6))
    .hover(self.opacity(0.9))
    .pressed(self.scale(0.98))
    .bind(View::grid_enabled)
```

The same component can therefore establish both its base appearance and its state-dependent appearance.

---

# Data-Driven Toggle Buttons

A higher-level Toggle Button is just an ordinary Rust function.

```rust id="m6q3wt"
fn favorite_toggle(user: &User) -> ToggleButton {
    toggle_button(localize("user.favorite"))
        .label(localize("user.favorite.label"))
        .aria(localize("user.favorite.description"))
        .bind(User::favorite)
}
```

Then:

```rust id="y8p4kc"
favorite_toggle(&user)
```

There is no need for a separate component-definition language.

> **Build higher-level components with ordinary Rust.**

---

# Custom Presentations

A Toggle Button does not require the words "On" and "Off."

The application can define whatever visual representation makes sense.

For example:

```rust id="w5n9qx"
toggle_button()
    .label(localize("editor.bold.label"))
    .aria(localize("editor.bold.description"))
    .children([
        icon("bold"),
    ])
    .active(self.color(blue))
    .inactive(self.color(gray))
    .bind(Editor::bold)
```

The same primitive could represent:

```text
Bold
Favorite
Muted
Pinned
Visible
Enabled
Selected
```

The Toggle Button provides the interaction and state semantics.

The application provides the vocabulary.

---

# What Toggle Button Does Not Do

The Toggle Button does not own:

- Application state
- Business rules
- Authorization
- Persistence
- API calls
- Workflow
- Domain semantics

It does not decide what `true` or `false` means.

For one application, `true` might mean "favorite."

For another, it might mean "notifications enabled."

For another, it might mean "show advanced options."

The Model defines that meaning.

> **The framework provides the primitive. The application provides the vocabulary.**

---

# The Principle

The Toggle Button deliberately follows the same architecture as the rest of Beverly's form controls.

**Create. Configure. Compose. Bind. Declare state. Emit intent.**

The Toggle Button provides a button-like interface for a boolean state.

The binding connects it to the Model.

The Model owns the value and decides whether it can change.

UI Events describe interaction.

Application Events and Commands describe application behavior.

And the component itself remains simple enough to understand at a glance.

> **A Checkbox represents a boolean. A Toggle Button represents the same boolean with a button-like interaction.**

> **The UI presents the state. The Model owns the state.**
