# Content

Content components give a Beverly view its internal structure. They decide how information is grouped, ordered, and related so that a screen remains readable as its content grows.

The goal is not to make every view look the same. It is to give each piece of content a clear responsibility and a predictable place in the composition.

```text
Content
│
├── Container
│   └── Defines the boundary of a content region
│
├── Row / Column
│   └── Arrange related content along a clear axis
│
├── Stack
│   └── Sequences content with consistent spacing
│
└── Children
	└── Express the relationship between a parent and its content
```

Content components shape the View, but they do not own application state or business rules. The Model remains the source of truth, while the View uses these primitives to present that state and collect interaction.

When choosing a content primitive, start with the relationship you need to express:

- use a container to establish a boundary and shared context
- use a row or column to make an explicit horizontal or vertical arrangement
- use a stack when the main concern is sequencing items with consistent space
- use children to make parent-child composition clear

This keeps the structure easy to read, easy to change, and local to the part of the interface it describes.
