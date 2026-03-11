# Web UI Architecture Review

## Current Architecture: Tera (Server-Rendered Templates)
Currently, the `xet-backend` project uses **Tera** for server-side HTML rendering. Tera is a Jinja2-inspired template engine for Rust.

**Pros:**
- **Simplicity:** Easy to understand, standard MVC-like pattern.
- **Fast Build Times:** Templates are evaluated at runtime (or cached), so modifying a template doesn't require recompiling the entire Rust backend.
- **No JS Build Step:** No need for Webpack, Vite, or npm in the build process.

**Cons:**
- **Runtime Errors:** Template errors (e.g., missing variables, syntax errors) often only surface at runtime.
- **Interactivity:** Building highly interactive features (like dynamic token management or live repository file browsing) requires writing vanilla JavaScript alongside the templates.

## Alternative 1: Askama + HTMX
**Askama** is a type-safe, compiled template engine for Rust. **HTMX** is a lightweight JavaScript library that allows accessing AJAX, CSS Transitions, WebSockets, and Server Sent Events directly in HTML using attributes.

**Pros:**
- **Compile-Time Safety:** Askama catches template errors during `cargo build`. If you pass the wrong struct or misspell a variable, it won't compile.
- **High Performance:** Compiled templates are extremely fast.
- **Rich Interactivity:** HTMX allows building SPA-like experiences (e.g., live search, inline editing) without writing custom JavaScript. It pairs perfectly with Rust backends returning HTML fragments.

**Cons:**
- **Slightly Slower Builds:** Compiling templates adds a small amount of overhead to the Rust build time.

## Alternative 2: Rust WASM SPAs (Leptos / Yew / Dioxus)
Frameworks like **Leptos**, **Yew**, and **Dioxus** allow writing full Single Page Applications (SPAs) entirely in Rust, compiled to WebAssembly (WASM).

**Pros:**
- **One Language:** Shared types between backend and frontend. Everything is Rust.
- **Component-Based:** React-like component models for highly interactive UIs.

**Cons:**
- **Heavy Tooling:** Requires `trunk` or `cargo-leptos`, WASM targets, and more complex build pipelines.
- **Large Binary Sizes:** WASM payloads can be heavy for simple pages.
- **Overkill:** For a lightweight Hub interface that mostly displays tables (repos, files) and simple forms (login, tokens), a full SPA is often unnecessary.

## Alternative 3: Separate Node.js/TS SPA (React / Vue / Svelte)
Decoupling the frontend into a standalone Next.js, Nuxt, or Vite+React app communicating with the Rust backend via REST API.

**Pros:**
- **Ecosystem:** Massive ecosystem of UI libraries (Tailwind UI, Radix, etc.).
- **Separation of Concerns:** Clear boundary between API and UI.

**Cons:**
- **Operational Complexity:** Requires managing two separate codebases, two build systems (Cargo + NPM), and potentially two deployment artifacts.
- **Loss of Unified Server:** Breaks the current "unified server" model where one binary serves everything.

## Recommendation
For the `xet-backend` Hub UI, **Askama + HTMX** is the strongest recommendation for a potential upgrade, though **Tera** remains a perfectly valid choice if the team prefers runtime templates.

1. **Why not a separate SPA or WASM?** The current unified server approach is a massive strength for deploying a private self-hosted alternative to HuggingFace. Requiring Node.js or heavy WASM builds complicates deployment for minimal gain, given that the Hub UI is mostly read-heavy (viewing repos, viewing files) and form-heavy (auth, tokens).
2. **Why Askama + HTMX over Tera?** Askama provides compile-time safety, ensuring that renaming a database model field won't silently break a template. HTMX is the perfect addition to provide modern, SPA-like interactivity (e.g., deleting a token without refreshing the page, or expanding a file tree) while keeping all logic in the Rust backend.

**Next Steps:**
If the current Tera implementation works and development speed is high, there is no urgent need to rewrite. However, as the UI grows (especially for dynamic features like token generation and file browsing), incrementally introducing HTMX into the existing Tera templates will provide immediate UX benefits without a full rewrite. If type safety becomes an issue, migrating from Tera to Askama is a relatively straightforward translation of Jinja-style syntax.
