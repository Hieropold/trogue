# GLOSSARY.md Format

> During `refine-task`, GLOSSARY.md changes are **proposed** in the session summary and written later by
> `update-jira-ticket` or `implement-jira-ticket` — never by the refining session itself.

## Structure

```md
# Glossary

Short definitions of common terms used in {Project Name}. Aimed at developers and AI
assistants who need a quick, plain-English reference.

## {Subheading, e.g., Core}

{Optional: Mermaid diagram showing relationships between terms}

- **{Term}** — {A one or two sentence description of the term. Focus on what it IS, not what it does.}
  _Avoid_: {Synonyms to avoid}

- **{Another Term}** — {Description}
```

## Rules

- **Be opinionated.** When multiple words exist for the same concept, pick the best one and list the others as aliases to
  avoid.
- **Flag conflicts explicitly.** If a term is used ambiguously, call it out with a clear resolution.
- **Keep definitions tight.** One or two sentences max. Define what it IS, not what it does.
- **Show relationships.** Use bold term names and express cardinality where obvious (use Mermaid for complex ones).
- **Only include terms specific to this project's context.** General programming concepts don't belong even if the project
  uses them extensively. Before adding a term, ask: is this a concept unique to this context, or a general programming
  concept? Only the former belongs.
- **Group terms under subheadings** when natural clusters emerge. If all terms belong to a single cohesive area, a flat
  list is fine.

## Single vs multi-context repos

**Single context (most repos):** One `GLOSSARY.md` at the repo root.

**Multiple contexts:** A `GLOSSARY-MAP.md` at the repo root lists the contexts, where they live, and how they relate to
each other:

```md
# Glossary Map

## Contexts

- [Ordering](./src/ordering/GLOSSARY.md) — receives and tracks customer orders
- [Billing](./src/billing/GLOSSARY.md) — generates invoices and processes payments
- [Fulfillment](./src/fulfillment/GLOSSARY.md) — manages warehouse picking and shipping

## Relationships

- **Ordering → Fulfillment**: Ordering emits `OrderPlaced` events; Fulfillment consumes them to start picking
- **Fulfillment → Billing**: Fulfillment emits `ShipmentDispatched` events; Billing consumes them to generate invoices
- **Ordering ↔ Billing**: Shared types for `CustomerId` and `Money`
```

The skill infers which structure applies:

- If `GLOSSARY-MAP.md` exists, read it to find contexts
- If only a root `GLOSSARY.md` exists, single context
- If neither exists, note this — a new `GLOSSARY.md` is a candidate for the session summary

When multiple contexts exist, infer which one the current ticket relates to. If unclear, ask.
