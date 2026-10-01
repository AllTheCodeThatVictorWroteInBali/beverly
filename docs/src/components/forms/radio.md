# Radio

> **Current status:** `.bind(...)` examples describe planned model-setter
> integration and are not currently executable against the crate.

A radio control represents a choice from a set of mutually exclusive options.

A radio can be used **as a group of options** or, when the application only needs a single explicit choice, as a **singular radio control**.

The underlying contract is the same as every Beverly form element:

> **The radio represents the choice. The binding connects it. The setter decides whether the choice can change. The Model owns the state.**

## Radio Groups

The most common use is a group of mutually exclusive options.

For example, a user selecting a subscription plan:

```rust
radio_group()
    .label(localize("subscription.plan.label"))
    .aria(localize("subscription.plan.description"))
    .bind(User::plan)
    .children([
        radio("free")
            .label(localize("subscription.plan.free.label"))
            .aria(localize("subscription.plan.free.description")),

        radio("pro")
            .label(localize("subscription.plan.pro.label"))
            .aria(localize("subscription.plan.pro.description")),

        radio("enterprise")
            .label(localize("subscription.plan.enterprise.label"))
            .aria(localize("subscription.plan.enterprise.description")),
    ])
```

Only one option can be selected at a time.

The group binds to the Model field:

```rust
#[model]
struct User {
    plan: Plan,
}
```

The Model owns the selected value:

```rust
enum Plan {
    Free,
    Pro,
    Enterprise,
}
```

The radio group is therefore not its own state-management system. It is a visual and interactive representation of `User::plan`.

## Binding

Binding works the same way as `input()`, `textarea()`, and `checkbox()`.

```text
Model → Binding → Radio Group
Radio → Binding → Setter → Model
```

When the Model changes, the selected radio changes.

When the user selects a radio, the binding sends the new value to the Model.

The setter is called whenever the selection changes.

```rust
impl User {
    fn set_plan(
        &mut self,
        plan: Plan,
    ) -> Result<(), UserError> {
        if self.requires_upgrade_confirmation(&plan) {
            return Err(UserError::UpgradeRequiresConfirmation);
        }

        self.plan = plan;

        Ok(())
    }
}
```

The UI does not need to reproduce this rule.

An agent, API, or another part of the application changing `User::plan` uses the same Model mechanics.

> **The UI is one client of the application. It is not the authority over the application.**

## Singular Radio

A radio can also be used as a single explicit choice.

This is useful when the application wants a boolean-like decision but wants the visual semantics of a radio control.

```rust
radio("yes")
    .label(localize("user.contact.label"))
    .aria(localize("user.contact.description"))
    .bind(User::contact_me)
```

The same binding rules apply.

If the application instead needs a true/false toggle, `checkbox()` is generally the more direct representation. Radio is most naturally used when the user is choosing among alternatives.

## Radio Options Are Data

Radio groups become particularly useful when options come from application data.

```rust
radio_group()
    .label(localize("subscription.plan.label"))
    .aria(localize("subscription.plan.description"))
    .bind(User::plan)
    .children(
        plans.iter().map(|plan| {
            radio(plan.id())
                .label(localize(plan.label_key()))
                .aria(localize(plan.description_key()))
        })
    )
```

The View does not need to know where the options came from.

They can come from:

- Model state
- A database
- An API
- Configuration
- A local file
- A stream
- AI-generated application data

The View simply renders the available choices.

## Validation

Validation belongs in the Model.

For example, the application might prevent a user from selecting an unavailable plan:

```rust
impl User {
    fn set_plan(
        &mut self,
        plan: Plan,
    ) -> Result<(), UserError> {
        if !self.available_plans.contains(&plan) {
            return Err(UserError::PlanUnavailable);
        }

        self.plan = plan;

        Ok(())
    }
}
```

The radio does not need to know why the option is unavailable.

If the setter rejects the change, the Model remains unchanged and the binding keeps the radio selection synchronized with the actual application state.

## Field Errors

Errors belong to the binding associated with the radio group.

```rust
radio_group()
    .label(localize("subscription.plan.label"))
    .aria(localize("subscription.plan.description"))
    .bind(User::plan)
    .children([
        radio("free")
            .label(localize("subscription.plan.free.label"))
            .aria(localize("subscription.plan.free.description")),

        radio("pro")
            .label(localize("subscription.plan.pro.label"))
            .aria(localize("subscription.plan.pro.description")),
    ])
    .error(
        self.children([
            text(|errors: &[String]| errors.join(" "))
        ])
    )
```

The error text is not hard-coded into the View.

The Model returns the semantic error. Beverly exposes the resulting error context to the field presentation.

> **The Model returns meaning. The UI presents language.**

## Async Changes

A radio selection can trigger asynchronous Model logic just like any other bound field.

```rust
impl User {
    async fn set_plan(
        &mut self,
        plan: Plan,
    ) -> Result<(), UserError> {
        check_plan_availability(&plan).await?;

        self.plan = plan;

        Ok(())
    }
}
```

While the transition is pending, the radio group can present a submitting state:

```rust
radio_group()
    .label(localize("subscription.plan.label"))
    .aria(localize("subscription.plan.description"))
    .bind(User::plan)
    .submitting(
        self.opacity(0.5)
    )
```

Or provide a richer pending presentation:

```rust
radio_group()
    .label(localize("subscription.plan.label"))
    .aria(localize("subscription.plan.description"))
    .bind(User::plan)
    .submitting(
        self.children([
            spinner(),
            text(localize("subscription.plan.updating")),
        ])
    )
```

The `self` here refers to the **radio group itself**.

It does not refer to the Event that caused the state change.

The Event changes application state. The component hook declares how the component should present itself while that state is active.

> **State determines when. Components determine what.**

## Accessibility

Accessibility is part of the radio's definition, not cleanup work.

The radio group has a label and accessible description:

```rust
radio_group()
    .label(localize("subscription.plan.label"))
    .aria(localize("subscription.plan.description"))
```

Each individual option also has its own label and accessible description:

```rust
radio("pro")
    .label(localize("subscription.plan.pro.label"))
    .aria(localize("subscription.plan.pro.description"))
```

A radio without its required accessibility metadata is not a valid Beverly form element.

Beverly can therefore catch missing required accessibility declarations at compile time rather than relying entirely on manual review.

> **Accessible by construction, not accessible by cleanup.**

## Styling

Radio groups and individual radio controls use the same chained styling API as other Beverly components.

```rust
radio_group()
    .padding(16)
    .radius(12)
    .gap(8)
    .width(400)
```

Individual options can be styled independently:

```rust
radio("pro")
    .label(localize("subscription.plan.pro.label"))
    .aria(localize("subscription.plan.pro.description"))
    .padding(8)
    .radius(8)
```

State-specific styling follows the same pattern:

```rust
radio_group()
    .error(
        self.color("red")
    )
    .submitting(
        self.opacity(0.5)
    )
```

The same API works because these are not special radio behaviors. They are component state presentations.

## Complete Example

Putting the pieces together:

```rust
radio_group()
    .label(localize("subscription.plan.label"))
    .aria(localize("subscription.plan.description"))
    .bind(User::plan)
    .children([
        radio("free")
            .label(localize("subscription.plan.free.label"))
            .aria(localize("subscription.plan.free.description")),

        radio("pro")
            .label(localize("subscription.plan.pro.label"))
            .aria(localize("subscription.plan.pro.description")),

        radio("enterprise")
            .label(localize("subscription.plan.enterprise.label"))
            .aria(localize("subscription.plan.enterprise.description")),
    ])
    .error(
        self.children([
            text(|errors: &[String]| errors.join(" "))
        ])
    )
    .submitting(
        self.opacity(0.5)
    )
```

There is no manual change handler.

There is no second copy of `plan` inside the View.

There is no UI-specific validation system.

The Model owns the value. The binding keeps the interface synchronized. The radio group presents the available choices.

## Radio API

| API                                                | Purpose                                                |
| -------------------------------------------------- | ------------------------------------------------------ |
| `radio_group()`                                    | Creates a group of mutually exclusive choices          |
| `radio(value)`                                     | Creates an individual radio option                     |
| `.children([...])`                                 | Adds radio options to the group                        |
| `.label(...)`                                      | Defines the accessible name                            |
| `.aria(...)`                                       | Defines additional accessible semantics or description |
| `.bind(...)`                                       | Connects the group to Model state                      |
| `.error(...)`                                      | Defines the field-error presentation                   |
| `.submitting(...)`                                 | Defines the pending-state presentation                 |
| `.on(...)`                                         | Connects Events or Commands                            |
| `.padding(...)`, `.gap(...)`, `.radius(...)`, etc. | Applies styling                                        |

Radio follows the same Beverly philosophy as every other form element:

> **Create. Configure. Compose. Bind. Declare state. Emit intent.**

The radio is simple because the system around it is consistent.
