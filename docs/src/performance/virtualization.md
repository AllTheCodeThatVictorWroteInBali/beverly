# Virtualization

Virtualization limits work for large data surfaces. In a UI framework like Beverly, the main idea is simple: do not create, measure, and render every row in a giant list if the user can only see a small portion of it at any given moment. Virtualization keeps the interface responsive by rendering only the items that are effectively visible, while still preserving the illusion of a complete dataset.

## Why virtualization is necessary

Large collections are common in modern apps: logs, chat threads, search results, transaction tables, file explorers, dashboards, and data feeds. A naive implementation may create and paint thousands or even tens of thousands of visible rows, even though the user only sees a small window at a time.

That creates several problems:

- expensive layout work for off-screen items
- unnecessary DOM or entity creation
- slower input handling and scrolling
- excess memory use and worse frame pacing
- a UI that feels sluggish even when the data itself is not large

Virtualization is the standard solution: keep the full dataset in memory, but only create and update the visible slice.

## The core idea

Instead of rendering every item, the system calculates a visible window:

- the viewport size
- the scroll offset
- the estimated height of each item
- the number of items in the dataset

Using that information, it determines which items are currently visible or near-visible, then only mounts, updates, and renders those rows.

The rest of the list is treated as a virtualized “space” that exists logically, but not physically in the UI tree. This keeps the data model intact without paying the full layout cost for everything at once.

## Virtualization in ECS terms

This fits well with ECS because the UI is defined as a set of entities with measurable state, not as one large object tree for every single item.

A virtualized list can be thought of as a narrow rendering window over a large collection. The actual list data remains in a data model, while the visible entities represent only the subset currently needed for display. Systems can update:

- scroll position
- visible item range
- row measurements
- item transforms and layout
- selection, hover, and keyboard focus state

When the user scrolls, the system changes the visible range and reuses existing row entities rather than recreating the entire list from scratch.

## Reuse instead of rebuild

A major benefit of virtualization is that it reuses rendered rows. When the user scrolls a list, the system may keep the same entities and simply move their content to different data slots.

This pattern is much more efficient than rebuilding thousands of widgets each frame. It reduces churn and helps maintain a stable UI during interaction.

This is especially important for lists that need to remain interactive: selecting a row, keyboard traversal, hover feedback, and focus management should continue to work smoothly even when the visible set is small.

## Why this matters for performance

Virtualization is not only about reducing rendering cost. It also reduces the cost of:

- text measurement and layout calculations
- CPU work spent updating large numbers of components
- memory pressure from large widget trees
- scrolling jank in dense interfaces

For large products, these effects compound quickly. A list that looks trivial at 50 rows may become unusable at 10,000 rows if the content is rendered indiscriminately.

## A practical model

A typical virtualized list system operates in stages:

1. determine container height and scroll offset
2. estimate the item height or measure a consistent row size
3. compute the visible start and end indexes
4. create or recycle only those row entities
5. update their content and layout based on the current scroll position
6. skip rendering and measuring off-screen rows

This keeps the visible UI bounded to the user’s actual viewport while preserving a coherent dataset behind it.

## Virtualization versus pagination

Virtualization and pagination solve related but distinct problems.

- Pagination loads a fixed page of results at a time.
- Virtualization keeps a continuous feeling of a long list while only drawing the active slice.

Pagination is often better for large server-backed datasets where a full collection is not practical to fetch at once. Virtualization is usually better when the user is scrolling through a long local dataset and expects smooth motion.

Both can be used together in a product strategy, but because Bevy and Beverly are oriented around composable UI state, virtualization often fits elegantly into the ECS model for dense lists and tables.

## Important trade-offs

Virtualization is powerful, but it requires careful design:

- row heights must be predictable or measurable
- scroll anchoring must remain stable when content changes
- keyboard navigation should still map to logical item indexes
- selection and focus state must survive window changes
- empty or sparse regions need clear handling

If these details are ignored, the UI can feel jumpy or inconsistent even though it is technically faster.

## The key takeaway

Virtualization is the technique of keeping the logical dataset large while only rendering the active portion. In an ECS-first UI system, this means maintaining a clear data model and a narrow set of visible entities that are updated as the user scrolls.

This is one of the clearest examples of performance work being architecture-aware rather than an optimization hack: it preserves usability, keeps the interface smooth, and reduces system cost without sacrificing a rich data model.

In short, virtualization keeps the app honest. It acknowledges that users only interact with a small window of a large dataset, and it designs the UI around that reality.
