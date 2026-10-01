# Option

> **Current status:** `.bind(...)` examples describe planned model-setter
> integration and are not currently executable against the crate.

`option()` represents a single available choice.

It is primarily used inside `select()`, but the component is intentionally designed as a simple, composable representation of a value and its presentation.

> **The Option represents a choice. The Select manages the choices. The Model owns the selected value.**

## Basic Option

An Option has a value and a label:

```rust id="m7q2vx"
option("pro")
    .label(localize("subscription.plan.pro.label"))
```

The value is what the application receives.

The label is what the user sees.

These do not have to be the same.

```rust id="f3k8pz"
option("enterprise-plan-v2")
    .label(localize("subscription.plan.enterprise.label"))
```

The application can use a stable internal value while presenting a human-readable, localized label.

## Inside a Select

Options are normally composed inside a Select:

```rust id="x4n6qt"
select()
    .label(localize("subscription.plan.label"))
    .aria(localize("subscription.plan.description"))
    .bind(User::plan)
    .children([
        option("free")
            .label(localize("subscription.plan.free.label")),

        option("pro")
            .label(localize("subscription.plan.pro.label")),

        option("enterprise")
            .label(localize("subscription.plan.enterprise.label")),
    ])
```

The Select determines which Option is selected.

The Option does not maintain its own copy of application state.

## Value and Model State

The Option's value is translated by the binding into the Model's type.

For example:

```rust id="q8w5nr"
#[model]
struct User {
    plan: Plan,
}
```

with:

```rust id="c6r2mk"
enum Plan {
    Free,
    Pro,
    Enterprise,
}
```

The Select can present string values while the binding maps them to the appropriate application type.

The important boundary is:

```text id="v5j8rq"
Option value
    ↓
Select
    ↓
Binding
    ↓
Model
```

The Option does not need to understand the Model.

## Labels

Every Option should have a user-facing label.

```rust id="h2m7wc"
option("pro")
    .label(localize("subscription.plan.pro.label"))
```

The label is presentation.

The value is application data.

This separation makes Options useful for localization, changing copy, accessibility, and data-driven interfaces without changing the underlying application value.

## Accessibility

Options participate in the accessibility semantics of their parent choice control.

When an Option exposes user-facing content, that content should be explicitly defined rather than relying on incidental implementation details.

```rust id="r9k4tp"
option("pro")
    .label(localize("subscription.plan.pro.label"))
```

The parent `select()` still requires its own `.label(...)` and `.aria(...)` declarations:

```rust id="n6p3yw"
select()
    .label(localize("subscription.plan.label"))
    .aria(localize("subscription.plan.description"))
    .children([
        option("free")
            .label(localize("subscription.plan.free.label")),

        option("pro")
            .label(localize("subscription.plan.pro.label")),
    ])
```

Beverly treats accessibility as part of the component contract.

> **Accessible by construction, not accessible by cleanup.**

## Disabled Options

An Option can be unavailable without removing it from the list.

```rust id="k8v3mz"
option("enterprise")
    .label(localize("subscription.plan.enterprise.label"))
    .disabled(true)
```

This is useful when the application wants to communicate that a choice exists but cannot currently be selected.

The reason for that state belongs to application data and logic, not the Option itself.

For example, the Model may determine that a plan is unavailable for the current account.

The View simply reflects that state.

## Data-Driven Options

Options can be generated directly from Model data:

```rust id="p4x7ns"
select()
    .label(localize("user.country.label"))
    .aria(localize("user.country.description"))
    .bind(User::country)
    .children(
        countries.iter().map(|country| {
            option(country.code())
                .label(localize(country.label_key()))
        })
    )
```

This keeps the Option deliberately lightweight.

The application determines which choices exist.

The Option determines how each choice is represented.

## Conditional Options

Options can also be composed conditionally:

```rust id="w6q2fj"
select()
    .label(localize("user.role.label"))
    .aria(localize("user.role.description"))
    .bind(User::role)
    .children([
        option("user")
            .label(localize("role.user")),

        option("admin")
            .label(localize("role.admin")),

        if permissions.can_manage_billing() {
            option("billing")
                .label(localize("role.billing"))
        }
    ])
```

The same `children()` composition model applies everywhere in Beverly.

An Option is just another component that can be created, configured, and composed.

## Styling

Options can expose styling hooks where the underlying choice component supports them:

```rust id="t3r8kp"
option("pro")
    .label(localize("subscription.plan.pro.label"))
    .padding(8)
```

However, the Select generally owns the overall presentation of its option list.

This distinction keeps responsibilities clear:

- **Select** — selection behavior and list presentation
- **Option** — individual choice and its presentation
- **Binding** — connection to application state
- **Model** — value, validation, and domain rules

## No Option-Level State Management

An Option should not become a miniature state-management system.

It does not need its own copy of the selected value.

It does not need its own event architecture.

It does not decide whether a choice is valid.

Those responsibilities already have a home.

The Option simply describes a choice.

> **If ordinary Rust and the parent component can solve it, Option does not need another abstraction.**

## Complete Example

```rust id="b7m4qx"
select()
    .label(localize("user.plan.label"))
    .aria(localize("user.plan.description"))
    .bind(User::plan)
    .children([
        option("free")
            .label(localize("user.plan.free.label")),

        option("pro")
            .label(localize("user.plan.pro.label")),

        option("enterprise")
            .label(localize("user.plan.enterprise.label"))
            .disabled(!permissions.can_use_enterprise()),
    ])
```

The structure is intentionally obvious.

The user sees a list of choices.

The Select manages the selection.

The binding connects that selection to the Model.

The Model remains the source of truth.

## Option API

| API              | Purpose                                            |
| ---------------- | -------------------------------------------------- |
| `option(value)`  | Creates an individual choice                       |
| `.label(...)`    | Defines the user-facing label                      |
| `.disabled(...)` | Controls whether the choice can be selected        |
| Styling methods  | Controls the Option's presentation where supported |
| `.children(...)` | Composes additional content where supported        |

Option is intentionally one of the smallest Beverly primitives.

> **Create. Label. Configure. Compose.**

The complexity belongs in the components that actually own the behavior — not in the individual choice.
