# Image

An Image is a visual presentation primitive for displaying image content.

Images are used throughout an application to communicate information, establish visual hierarchy, represent people or objects, display media, and provide visual context.

The Image component should remain intentionally simple.

It presents an image.

It does not become an image database, asset manager, upload system, image editor, or application state system.

> **The Image displays the asset. The application owns the data.**

## Basic Usage

An Image is created with `image()` and configured using the same fluent API used throughout Beverly:

```rust
image()
    .src("link.jpg")
```

Accessibility should be configured explicitly:

```rust
image()
    .src("link.jpg")
    .aria("my image")
```

For user-facing applications, the accessible description should normally be localized:

```rust
image()
    .src("profile.jpg")
    .aria(localize("user.avatar.description"))
```

The API follows Beverly's familiar pattern:

```text
Create → Configure → Compose
```

`image()` creates the primitive.

`.src()` provides the image source.

The remaining methods configure how the image is presented.

---

## Source

The image source is provided through `.src()`:

```rust
image()
    .src("link.jpg")
```

The source may eventually represent different kinds of image resources, but the application-facing concept remains the same:

**Tell the Image what to display.**

The source could eventually come from:

- a local asset
- a packaged application resource
- a remote URL
- application data
- generated content
- another resource system

The Image remains responsible for presentation rather than storage or ownership of the underlying asset.

> **The Image knows how to display an image. It does not need to know how your application stores it.**

---

## Accessibility

Images should make their accessible meaning explicit.

```rust
image()
    .src("link.jpg")
    .aria("my image")
```

For localized application content:

```rust
image()
    .src("profile.jpg")
    .aria(localize("user.avatar.description"))
```

The accessible description communicates what the image represents to users who cannot rely on the visual presentation.

For example:

```rust
image()
    .src("product.jpg")
    .aria(localize("product.image.description"))
```

The Image itself remains a visual primitive. Accessibility metadata provides its semantic context.

> **Accessible by construction, not accessible by cleanup.**

Decorative images should also be able to communicate that they are decorative rather than forcing assistive technologies to interpret meaningless visual content.

The important question is simple:

**Does this image communicate information?**

If it does, provide its meaning.

If it does not, identify it as decorative.

---

## Sizing

Images commonly need explicit dimensions.

Beverly's fluent styling API should make this straightforward:

```rust
image()
    .src("profile.jpg")
    .width(48)
    .height(48)
```

Or:

```rust
image()
    .src("hero.jpg")
    .width(800)
    .height(450)
```

Sizing is presentation configuration.

The application does not need a separate image-sizing system.

---

## Aspect Ratio and Fit

Images should normally preserve their intended aspect ratio.

Beverly can provide familiar presentation controls for fitting an image into its available space:

```rust
image()
    .src("hero.jpg")
    .width(800)
    .height(450)
    .fit(ImageFit::Cover)
```

Common presentation modes can include:

```rust
ImageFit::Contain
ImageFit::Cover
ImageFit::Fill
ImageFit::None
```

These describe **how the image is presented**, not what the image means.

The distinction is important:

```text
Image Source
     ↓
   Image
     ↓
Presentation
```

The source identifies what is displayed.

The presentation determines how it appears.

---

## Styling

Image uses the same fluent styling model as other Beverly components.

```rust
image()
    .src("profile.jpg")
    .width(48)
    .height(48)
    .radius(999)
```

Or:

```rust
image()
    .src("hero.jpg")
    .width(800)
    .height(450)
    .radius(16)
    .opacity(0.9)
```

There is no separate image styling system.

Image is a normal Beverly component.

> **If ordinary component styling can solve it, use ordinary component styling.**

---

## Image Is Presentation

An Image does not own the data it displays.

This:

```rust
image()
    .src("profile.jpg")
```

does not own `profile.jpg`.

It simply presents it.

Likewise, an application might eventually derive an image source from its Model:

```text
Model
  ↓
Image Source
  ↓
Image
  ↓
Rendered Visual
```

The Model owns application data.

The Image presents that data.

This keeps the responsibility clear and prevents presentation components from becoming accidental state-management systems.

> **The View presents data. The Model owns data.**

---

## Image and Domain Data

Applications often have domain-specific concepts that happen to be represented by images:

- user avatars
- product images
- document covers
- device photos
- artwork
- maps
- charts
- generated images

Beverly does not need to know what any of these things mean.

An application can create higher-level components from the Image primitive using ordinary Rust functions.

For example:

```rust
fn avatar(src: &str) -> Image {
    image()
        .src(src)
        .width(48)
        .height(48)
        .radius(999)
}
```

Now the application can use:

```rust
avatar("profile.jpg")
```

There is no nested `Image` component.

`avatar()` **is simply a function that returns an Image**.

This is an important Beverly principle:

> **Higher-level components are ordinary Rust functions that return lower-level components.**

A more specialized component can therefore be built without introducing another framework, component class, or abstraction layer.

For example:

```rust
fn product_image(src: &str) -> Image {
    image()
        .src(src)
        .width(320)
        .height(240)
        .radius(12)
        .fit(ImageFit::Cover)
}
```

Then:

```rust
product_image("product.jpg")
```

The implementation remains completely transparent.

A developer can jump to the function and immediately see how the component is constructed.

This is exactly the kind of code that is easy for both humans and AI systems to understand.

> **Build the primitive once. Build higher-level components with ordinary Rust.**

---

## Image and Text

Image and Text are both presentation primitives, but they communicate different kinds of information.

```rust
text("Product Name")
```

presents textual information.

```rust
image()
    .src("product.jpg")
    .aria("Product image")
```

presents visual information.

They can be composed naturally:

```rust
card()
    .children([
        image()
            .src("product.jpg")
            .aria(localize("product.image.description")),

        text(Product::name),
    ])
```

Neither primitive needs to know about the other.

The parent component determines their composition.

---

## Image and Badge

Image can be composed with Badge to create compact information-rich interfaces:

```rust
card()
    .children([
        image()
            .src("product.jpg")
            .aria(localize("product.image.description")),

        text(Product::name),

        badge(Product::status),
    ])
```

The responsibilities remain separate.

The Image provides visual content.

The Badge provides compact contextual information.

The Model owns the underlying application state.

> **Composition creates richer interfaces without making the primitives more complicated.**

---

## Image and Tooltip

An Image can provide additional context through a Tooltip:

```rust
image()
    .src("product.jpg")
    .aria(localize("product.image.description"))
    .tooltip(localize("product.image.tooltip"))
```

The Tooltip remains supplementary.

The Image remains the visual element.

The accessible description and Tooltip may communicate related information, but they serve different purposes.

The `aria` configuration provides accessible semantics.

The Tooltip provides additional contextual information during interaction.

---

## Interactive Images

An Image is not inherently interactive.

This:

```rust
image()
    .src("product.jpg")
    .aria(localize("product.image.description"))
```

simply displays an image.

If an image participates in an interaction, the interaction should be explicit.

For example:

```rust
image()
    .src("product.jpg")
    .aria(localize("product.image.description"))
    .on("click", Command::Product::Open)
```

Or the Image can be placed inside an explicitly interactive component:

```rust
button()
    .label(localize("product.open.label"))
    .aria(localize("product.open.description"))
    .children([
        image()
            .src("product.jpg")
            .aria(localize("product.image.description")),
    ])
    .on("click", Command::Product::Open)
```

The second form makes the interaction boundary especially clear:

**Button is the control. Image is the content.**

The Image itself does not need to understand the application action.

> **The Image displays. The Button interacts. The Command expresses application action.**

---

## Image Events

Like other presentation primitives, Image does not need an application Event system simply to exist on the screen.

If the Image participates in interaction, it can expose the appropriate UI lifecycle Events:

```rust
image()
    .src("product.jpg")
    .aria(localize("product.image.description"))
    .on("click_start", Ui::ProductImage::Pressed)
    .on("click_end", Ui::ProductImage::Released)
    .on("click", Ui::ProductImage::Clicked)
```

The application can then connect those UI Events to Commands:

```rust
.on("click", Command::Product::Open)
```

The same distinction used throughout Beverly remains:

```text
UI Event
   ↓
Controller
   ↓
Application Command
   ↓
Model / Application
```

The Image reports interaction.

The application decides what that interaction means.

---

## Composition

Image should compose naturally with every other Beverly primitive.

```rust
card()
    .children([
        image()
            .src("document.jpg")
            .aria(localize("document.cover.description")),

        text(Document::title),

        badge(Document::status),
    ])
```

Or:

```rust
row()
    .children([
        image()
            .src("avatar.jpg")
            .aria(localize("user.avatar.description")),

        text(User::name),

        badge(User::status),
    ])
```

There is no special composition model.

Image uses the same `.children()` system as the rest of Beverly.

Higher-level components remain ordinary Rust functions.

For example:

```rust
fn user_row(user: &User) -> impl Component {
    row()
        .children([
            avatar(user.avatar()),
            text(user.name()),
            badge(user.status()),
        ])
}
```

The function is the component.

The components inside it are simply the values it returns or composes.

> **One composition model. Ordinary Rust. Reusable components.**

---

## Image Is Not an Asset Manager

Image should not become responsible for everything surrounding images.

It is not:

- an asset database
- an upload service
- an image editor
- a media library
- a CDN abstraction
- a storage system
- an image synchronization system

Those concerns belong elsewhere.

Beverly's Image primitive should remain focused:

```text
Asset / Data System
        ↓
      Image
        ↓
   Presentation
```

This makes Image useful whether the application is a small desktop application, an enterprise data interface, an offline device, or an AI-generated application.

---

## The Principle

Image demonstrates a core Beverly principle:

> **Hide implementation complexity without hiding application meaning.**

The developer should be able to write:

```rust
image()
    .src("link.jpg")
    .aria("my image")
```

without needing to understand the rendering machinery underneath it.

At the same time, the API makes the important information explicit:

- this is an Image
- this is its source
- this is its accessible meaning

Everything else can remain ordinary component configuration.

Higher-level abstractions do not require another framework.

They are ordinary Rust:

```rust
fn avatar(src: &str) -> Image {
    image()
        .src(src)
        .width(48)
        .height(48)
        .radius(999)
}
```

> **Magic in implementation. Explicitness at the API.**

---

## Why Image Is Simple

Image should not become:

- an asset database
- an upload system
- an image editor
- a media library
- an application state system
- a networking abstraction
- a second composition framework

It displays images.

The Model owns application data.

The application owns the domain meaning.

The View determines how that data is presented.

Beverly handles the rendering details.

Higher-level components remain ordinary Rust functions that return Beverly components.

> **The Image displays. The Model owns. The application decides.**

That simplicity makes Image predictable, composable, and easy for both humans and AI systems to understand.

## Summary

The Image follows Beverly's broader architecture:

- **`image()`** creates the primitive.
- **`.src()`** provides the image source.
- **`.aria()`** provides accessible meaning.
- **Styling methods** control presentation.
- **Model** owns application data.
- **Events** describe meaningful interaction when the Image participates in a control.
- **Controller** connects interaction to application behavior.
- **Composition** determines how the Image participates in the interface.
- **Higher-level components** are ordinary Rust functions that return Beverly components.

The common case stays deliberately small:

```rust
image()
    .src("link.jpg")
    .aria("my image")
```

A specialized component is just as simple:

```rust
fn avatar(src: &str) -> Image {
    image()
        .src(src)
        .width(48)
        .height(48)
        .radius(999)
}
```

From there, the same primitive can be sized, styled, fitted, composed, given contextual help, or incorporated into interactive components.

> **Create. Configure. Compose. Extend with ordinary Rust.**
