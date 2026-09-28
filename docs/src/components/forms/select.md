# Select

`select()` lets a user choose a value from a list of available choices.

It follows the same Beverly form-element contract as `input()`, `textarea()`, `checkbox()`, and `radio_group()`:

> **The Select presents choices. The binding connects them. The setter validates and transforms the value. The Model owns the state.**

The individual choices are represented by `option()`, which is covered separately.

## Basic Select

A Select binds directly to Model state:

```rust id="5w8f2x"
select()
    .label(localize("user.country.label"))
    .aria(localize("user.country.description"))
    .bind(User::country)
```

The Model owns the selected value:

```rust id="q2m7yr"
#[model]
struct User {
    country: Country,
}
```

The Select does not maintain a second authoritative copy of `country`.

When the Model changes, the Select reflects the new value.

When the user makes a selection, the binding sends the new value to the Model.

```text id="k7c1pz"
Model → Binding → Select
Select → Binding → Setter → Model
```

## Options

Options are added through `children()`:

```rust id="8x5kq1"
select()
    .label(localize("user.country.label"))
    .aria(localize("user.country.description"))
    .bind(User::country)
    .children([
        option("us")
            .label(localize("country.us")),
        option("ca")
            .label(localize("country.ca")),
        option("id")
            .label(localize("country.id")),
    ])
```

`select()` owns the selection behavior.

`option()` represents an individual available choice.

This separation keeps the API composable and makes the same options reusable wherever appropriate.

## Data-Driven Options

Options can come directly from application data.

```rust id="qf4n3a"
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

The options might come from:

- Model state
- A database
- An API
- Configuration
- A local file
- A stream
- AI-generated data

The Select only needs to render the choices provided to it.

## Binding

Binding handles the conversion between the selected option and the typed Model value.

```rust id="5p8f4v"
select()
    .label(localize("user.role.label"))
    .aria(localize("user.role.description"))
    .bind(User::role)
```

When the user selects a different option, the Model setter is called.

For example:

```rust id="0q7s2n"
impl User {
    fn set_role(
        &mut self,
        role: Role,
    ) -> Result<(), UserError> {
        if !self.allowed_roles.contains(&role) {
            return Err(UserError::RoleNotAllowed);
        }

        self.role = role;

        Ok(())
    }
}
```

The Select does not need to know why a role is unavailable.

The Model owns that rule.

If the setter rejects the change, the Model remains unchanged and the Select remains synchronized with the actual application state.

> **The input reports the choice. The Model decides what that choice means.**

## Transformation

The setter can also transform the selected value before storing it.

```rust id="1p9j4e"
impl User {
    fn set_country(
        &mut self,
        country: String,
    ) -> Result<(), UserError> {
        let country = normalize_country_code(&country)?;

        self.country = country;

        Ok(())
    }
}
```

The Select does not need to understand normalization.

The binding translates the selection into the Model's type, and the Model remains the source of truth.

## Validation Errors

Validation errors belong to the bound field.

```rust id="m5k3x8"
select()
    .label(localize("user.role.label"))
    .aria(localize("user.role.description"))
    .bind(User::role)
    .error(
        self.children([
            text(|errors: &[String]| errors.join(" "))
        ])
    )
```

The error context is supplied automatically to the field presentation.

`self` refers to the Select itself.

`errors` represents the current errors associated with its binding.

The UI does not hard-code validation messages. The Model returns the semantic error, and the application's localization layer provides the appropriate language.

> **The Model returns meaning. The UI presents language.**

## Async Validation

A selection can trigger asynchronous validation or state transitions.

```rust id="7t4m2n"
impl User {
    async fn set_role(
        &mut self,
        role: Role,
    ) -> Result<(), UserError> {
        check_role_availability(&role).await?;

        self.role = role;

        Ok(())
    }
}
```

While the transition is pending, the Select can change its presentation:

```rust id="c8v6w2"
select()
    .label(localize("user.role.label"))
    .aria(localize("user.role.description"))
    .bind(User::role)
    .submitting(
        self.opacity(0.5)
    )
```

Or provide a richer pending presentation:

```rust id="r3k7m9"
select()
    .label(localize("user.role.label"))
    .aria(localize("user.role.description"))
    .bind(User::role)
    .submitting(
        self.children([
            spinner(),
            text(localize("user.role.updating")),
        ])
    )
```

Again, `self` refers to the Select component.

It does not refer to the Event that caused the state change.

The Event or Model transition changes state. The component hook declares how the Select presents that state.

> **State determines when. Components determine what.**

## Styling

Select uses the same fluent styling API as other Beverly components:

```rust id="e1v6q4"
select()
    .padding(12)
    .radius(8)
    .width(300)
```

State-specific styling uses the same mechanism:

```rust id="z6p2r8"
select()
    .error(
        self.color("red")
    )
    .submitting(
        self.opacity(0.5)
    )
```

There is no separate styling system for form states.

The component API remains consistent.

## Accessibility

A Select is not a valid Beverly form element without its required accessibility metadata.

At minimum, the Select declares both its label and accessible description:

```rust id="n4c8y2"
select()
    .label(localize("user.country.label"))
    .aria(localize("user.country.description"))
    .bind(User::country)
```

The label identifies the control.

The ARIA declaration provides additional accessible semantics or description.

These are part of the component contract, not cleanup work.

Beverly can enforce required accessibility metadata at compile time, helping prevent expensive accessibility and compliance failures.

> **Accessible by construction, not accessible by cleanup.**

## Complete Example

A complete Select combines styling, binding, options, validation, and state presentation:

```rust id="p7d3k9"
select()
    .label(localize("user.country.label"))
    .aria(localize("user.country.description"))
    .bind(User::country)
    .padding(12)
    .radius(8)
    .width(300)
    .children([
        option("us")
            .label(localize("country.us")),

        option("ca")
            .label(localize("country.ca")),

        option("id")
            .label(localize("country.id")),
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

The Select itself remains simple.

It presents choices.

The binding connects those choices to typed application state.

The Model owns validation, transformation, and state.

## Select API

| API                                                | Purpose                                                |
| -------------------------------------------------- | ------------------------------------------------------ |
| `select()`                                         | Creates a Select control                               |
| `.children([...])`                                 | Adds available options                                 |
| `.label(...)`                                      | Defines the control's accessible name                  |
| `.aria(...)`                                       | Defines additional accessible semantics or description |
| `.bind(...)`                                       | Connects the Select to Model state                     |
| `.error(...)`                                      | Defines the field-error presentation                   |
| `.submitting(...)`                                 | Defines the pending-state presentation                 |
| `.on(...)`                                         | Connects Events or Commands                            |
| `.padding(...)`, `.gap(...)`, `.radius(...)`, etc. | Applies styling                                        |

`option()` defines the choices inside the Select and will be covered next.

> **Select chooses. Binding connects. The Model owns the value.**
