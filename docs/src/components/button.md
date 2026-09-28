# Button

A Button is a UI element that lets an end user interact with an application. It does not know what the interaction means, why the user clicked it, or what application behavior should follow. A Button provides a visual and interactive surface that produces UI Events; the application decides what those Events mean.

This distinction keeps the Button reusable. The same Button can participate in different application behaviors, while UI-specific interactions such as pressed or released states can remain completely independent from application Commands.

## Basic Button

A Button starts with user-facing content and its accessibility semantics.

```rust
button(localize("user.save"))
    .label(localize("user.save.label"))
    .aria(localize("user.save.description"))
```

The text displayed by the Button is separate from its accessibility metadata. `.label()` identifies the control, while `.aria()` provides additional accessible meaning or description.

User-facing strings should normally come from the application's localization system rather than being embedded directly in the component.

## UI Events

Buttons produce Events describing interaction with the UI.

```rust
button(localize("user.save"))
    .label(localize("user.save.label"))
    .aria(localize("user.save.description"))
    .on("click_start", Ui::SaveButton::Pressed)
    .on("click_end", Ui::SaveButton::Released)
    .on("click", Ui::SaveButton::Clicked)
```

These Events describe **what happened to the Button**, not what the application should do about it.

A useful interaction lifecycle is:

```text
Pointer Down
    ↓
click_start
    ↓
pressed
    ↓
Pointer Up
    ↓
click_end
    ↓
click
```

`click_start` can represent the beginning of an interaction, while `click_end` represents its completion. `click` represents the completed UI interaction.

The important distinction is that `Ui::SaveButton::Pressed` is still a UI concept. It does not mean "save the user." It means that the Save Button entered its pressed interaction state.

That makes UI Events useful for effects that have nothing to do with the application's primary action. Another component, animation, sound effect, visual indicator, telemetry system, or other UI behavior could react to the same Event without changing the Button itself.

## UI Events vs. Commands

A Button should not directly express application intent.

The UI can report:

```rust
Ui::SaveButton::Clicked
```

The application can then connect that Event to a Command:

```rust
Ui::SaveButton::Clicked
    → Command::User::Save
```

The Command describes an application operation. The Button does not need to know that the operation is saving a user, creating an order, submitting a payment, opening a document, or doing anything else.

For example:

```rust
button(localize("user.create"))
    .label(localize("user.create.label"))
    .aria(localize("user.create.description"))
    .on("click", Command::User::Create)
```

The Button only knows that it was clicked. The Controller or application wiring determines that the resulting UI Event should produce `Command::User::Create`.

This separation keeps the component generic while making application behavior explicit.

> **The Button reports interaction. The Command expresses application action.**

## One UI Event, Multiple Behaviors

A single UI Event can be observed by multiple parts of an application.

```rust
button(localize("user.save"))
    .label(localize("user.save.label"))
    .aria(localize("user.save.description"))
    .on("click", Command::User::Save)
    .on("click", Analytics::SaveClicked)
    .on("click", Agent::ObserveSave)
```

The Button still has no knowledge of any of these behaviors. It simply produces the `click` Event.

This allows the same interaction to drive application behavior, analytics, agent observation, or other systems without adding that knowledge to the Button component.

Likewise, UI-specific Events can be handled independently:

```rust
.on("click_start", Ui::SaveButton::Pressed)
.on("click_end", Ui::SaveButton::Released)
```

A visual effect could respond to `Ui::SaveButton::Pressed` without affecting the application's save operation.

## Interaction State

Buttons have their own UI state. Beverly exposes that state so the visual presentation can respond directly to interaction.

```rust
button(localize("user.save"))
    .label(localize("user.save.label"))
    .aria(localize("user.save.description"))
    .hover(self.opacity(0.9))
    .focused(self.outline_width(2))
    .pressed(self.scale(0.98))
    .disabled(User::is_saving)
    .submitting(self.opacity(0.5))
    .error(self.color("red"))
    .success(self.color("green"))
```

These declarations describe how the Button should present itself in different states. They do not contain the application's business logic.

`self` refers to the Button being configured. For example:

```rust
.pressed(self.scale(0.98))
```

means that the Button scales itself when it is pressed. It does not refer to the Event that caused the state change.

The Button can therefore have distinct presentations for:

- **Default** — normal presentation.
- **Hover** — pointer is over the Button.
- **Focused** — Button has keyboard or programmatic focus.
- **Pressed** — interaction is currently being activated.
- **Disabled** — interaction is unavailable.
- **Submitting** — an associated operation is in progress.
- **Error** — associated application state indicates an error.
- **Success** — associated application state indicates success.

These are UI presentations. The state that determines whether a Button is disabled, submitting, successful, or in error can come from the Model or application state.

> **State determines when. Components determine what.**

## Pressed Is Not the Command

The distinction between `pressed` and `click` is particularly important.

A Button becoming pressed is a UI fact:

```rust
Ui::SaveButton::Pressed
```

It can be used for visual feedback:

```rust
.on("click_start", Ui::SaveButton::Pressed)
```

It could also trigger another UI-level behavior:

```rust
.on("click_start", Ui::Toolbar::ShowPressedState)
```

None of these Events need to know what the eventual application action is.

The completed interaction can then produce an application Command:

```rust
.on("click", Command::User::Save)
```

This creates a clean boundary:

```text
UI interaction
     ↓
UI Event
     ↓
application wiring
     ↓
Command
     ↓
application behavior
```

The Button remains a reusable primitive. The application remains responsible for deciding what the interaction means.

## Children

A Button can contain composed content rather than only a string.

```rust
button()
    .label(localize("user.save.label"))
    .aria(localize("user.save.description"))
    .children([
        icon("save"),
        text(localize("user.save")),
    ])
```

This makes it possible to compose icons, text, badges, loading indicators, or other Beverly components inside the Button.

For example, a submitting presentation can replace or augment its content:

```rust
button()
    .label(localize("user.save.label"))
    .aria(localize("user.save.description"))
    .children([
        icon("save"),
        text(localize("user.save")),
    ])
    .submitting(self.children([
        spinner(),
        text(localize("user.saving")),
    ]))
```

The Button still does not know why it is submitting. It only presents the state it has been given.

## Disabled State

Disabled state belongs to the interface, while the reason for that state belongs to application state.

```rust
button(localize("user.save"))
    .label(localize("user.save.label"))
    .aria(localize("user.save.description"))
    .disabled(User::is_saving)
    .on("click", Command::User::Save)
```

If `User::is_saving` becomes true, the Button can become unavailable without the Button itself implementing any saving logic.

This keeps the UI declarative:

```text
Model/Application State
        ↓
     Button State
        ↓
     Presentation
```

The Button presents the condition. The application owns the condition.

## Events Are UI Vocabulary

Beverly keeps UI Events intentionally abstract.

Events such as:

```rust
Ui::SaveButton::Pressed
Ui::SaveButton::Released
Ui::SaveButton::Clicked
```

describe the interface.

Commands such as:

```rust
Command::User::Create
Command::User::Save
Command::Payment::Confirm
Command::Order::Submit
```

describe application operations.

That distinction gives Beverly two useful vocabularies. The UI can describe what happened without knowing the application's domain, while the application can describe what it wants to accomplish without knowing how the interaction occurred.

A Command could come from a Button, keyboard shortcut, command palette, API request, agent, or another Event source. The Button does not need to change when the source of the Command changes.

> **UI Events describe interaction. Commands describe application behavior.**

## Accessibility

Every Button declares its accessible label and additional accessible semantics.

```rust
button(localize("user.create"))
    .label(localize("user.create.label"))
    .aria(localize("user.create.description"))
```

Accessibility is part of the Button's construction rather than something added after the component is finished.

The same declarations should remain present when the Button becomes more complex:

```rust
button()
    .label(localize("user.create.label"))
    .aria(localize("user.create.description"))
    .children([
        icon("plus"),
        text(localize("user.create")),
    ])
```

This makes the accessible meaning of the control explicit at the API level.

> **Accessible by construction, not accessible by cleanup.**

## Composition

Buttons are ordinary Beverly components. They can be styled, composed, bound to Events, and configured with state-specific presentations without becoming application-specific.

A complete example might look like:

```rust
button()
    .label(localize("user.create.label"))
    .aria(localize("user.create.description"))
    .children([
        icon("plus"),
        text(localize("user.create")),
    ])
    .hover(self.opacity(0.9))
    .focused(self.outline_width(2))
    .pressed(self.scale(0.98))
    .disabled(User::is_creating)
    .submitting(self.opacity(0.5))
    .error(self.color("red"))
    .success(self.color("green"))
    .on("click_start", Ui::CreateButton::Pressed)
    .on("click_end", Ui::CreateButton::Released)
    .on("click", Command::User::Create)
```

The component describes the complete UI surface without embedding the implementation of `User::Create`.

The Button knows how to look, how to expose its accessibility semantics, and how to report interaction. The application decides what happens next.

## The Button Contract

The Button has a deliberately small conceptual contract:

**Create. Configure. Compose. Present state. Emit UI Events.**

It does not own application logic, business rules, or Commands. It provides the interface through which a human can interact with the application, while keeping the meaning of that interaction outside the component.

This separation is what makes the Button reusable by both humans and AI-generated application code. A developer or agent can understand the Button without understanding the entire application, while the Controller remains the place where UI Events become application behavior.

> **The Button is the interface. The application decides what the interaction means.**
