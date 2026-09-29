# Avatar

An Avatar is a circular image used to represent a person, account, or other identity.

It is intentionally a very small abstraction over the Image primitive. It does not introduce a new image system, state system, or composition model.

**An Avatar is an Image with an opinionated shape.**

## Basic Usage

Create an Avatar with `avatar()` and configure it with the same fluent API used by other Beverly components.

```rust
avatar()
    .src("profile.jpg")
```

The default presentation is circular.

For accessibility, provide a description of who or what the image represents:

```rust
avatar()
    .src("profile.jpg")
    .aria(localize("user.avatar.description"))
```

The Avatar does not require application state. It simply presents an image.

## Sizing

Use the same sizing primitives available to Image:

```rust
avatar()
    .src("profile.jpg")
    .width(48)
    .height(48)
```

Because an Avatar is circular, width and height are normally equal.

Different sizes can be used for different contexts:

```rust
avatar()
    .src("profile.jpg")
    .width(32)
    .height(32)
```

```rust
avatar()
    .src("profile.jpg")
    .width(64)
    .height(64)
```

```rust
avatar()
    .src("profile.jpg")
    .width(96)
    .height(96)
```

## Styling

Avatar provides a convenient circular presentation without requiring the application to manually configure the image radius.

Additional styling can still be applied through the normal fluent API:

```rust
avatar()
    .src("profile.jpg")
    .width(48)
    .height(48)
    .border_width(2)
```

The Avatar remains an Image underneath. Its purpose is simply to establish a useful default presentation for identity images.

## Data-Driven Avatars

Avatars commonly come from application data.

```rust
avatar()
    .src(user.avatar_url())
    .aria(localize("user.avatar.description"))
```

The Avatar does not care where the image came from.

The Model might provide the URL from a database, API, local file, or another source. The View simply presents it.

```text
Model → View → Avatar → Image
```

The Avatar does not own the user's identity. It presents it.

## Composition

Avatar composes with other Beverly primitives like any other component.

```rust
card()
    .children([
        avatar()
            .src(user.avatar_url())
            .aria(localize("user.avatar.description")),

        text(user.name()),
        badge(user.status()),
    ])
```

It can also be used anywhere an Image can be used:

```rust
row()
    .children([
        avatar().src("alice.jpg"),
        text("Alice"),
    ])
```

There is no special Avatar composition system.

**One composition model. Ordinary Rust.**

## Higher-Level Components

Building a higher-level Avatar component is just an ordinary Rust function that returns an Avatar.

```rust
fn user_avatar(user: &User) -> Avatar {
    avatar()
        .src(user.avatar_url())
        .width(48)
        .height(48)
        .aria(localize("user.avatar.description"))
}
```

Use it like any other component:

```rust
user_avatar(&user)
```

A more specialized component can build on the same primitive:

```rust
fn profile_avatar(user: &User) -> Avatar {
    avatar()
        .src(user.avatar_url())
        .width(64)
        .height(64)
        .aria(localize("user.profile_avatar.description"))
}
```

There is no need for a nested Avatar component or a special extension mechanism.

**Higher-level components are ordinary Rust functions that return lower-level components.**

## Interaction

An Avatar is not inherently interactive.

If an application wants an Avatar to perform an action, the application can attach the appropriate UI event:

```rust
avatar()
    .src(user.avatar_url())
    .aria(localize("user.avatar.description"))
    .on("click", Command::User::OpenProfile)
```

The Avatar reports the interaction. The application decides what that interaction means.

An Avatar can also be placed inside another interactive component when that better represents the interface.

## Accessibility

An Avatar is an image, so its accessible meaning should describe the identity or purpose represented by the image.

```rust
avatar()
    .src(user.avatar_url())
    .aria(localize("user.avatar.description"))
```

For example, the localized description might communicate that the image represents a particular user.

If the Avatar is interactive, its surrounding interaction should also have an appropriate accessible label and description.

**Accessible by construction, not accessible by cleanup.**

## What Avatar Does Not Do

Avatar does not:

- own user state
- manage identity
- load or cache application data
- manage authentication
- provide a separate image system
- define application behavior
- create application Events
- replace the Image primitive

Those concerns belong elsewhere.

The Avatar only provides a convenient semantic presentation for a circular image.

## Avatar and Image

| Component  | Purpose                                            |
| ---------- | -------------------------------------------------- |
| `image()`  | General-purpose image presentation                 |
| `avatar()` | Circular image used for identity or representation |

When you need a general image, use `image()`.

When you need a circular identity image, use `avatar()`.

Internally, the distinction should remain small.

**The Avatar is simply an Image with an opinionated presentation.**

## Design Principle

Beverly should not create a new abstraction unless the abstraction carries useful meaning.

`Image` already provides the underlying image behavior. `Avatar` adds a common semantic pattern: a circular image representing identity.

That makes the abstraction useful without making it complicated.

**Create. Configure. Compose. Extend with ordinary Rust.**

**The framework provides the primitive. Your application provides the identity.**
