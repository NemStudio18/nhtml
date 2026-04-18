# Nhtml Project Audit: 05 - Roadmap: Ncss Specification and v2.0 Architecture

## 1. Introduction: Completing the N-Suite ecosystem

The stabilization of Nhtml v1.0 has provided a robust foundation for markup and logic. However, to fulfill our vision of a truly "Unified Web Meta-Language," we must address the remaining bottleneck in the styling layer. 

The next phase of the project, spanning 2026-2027, focuses on the introduction of **Ncss** (Next-CSS) and the architectural leap into Nhtml v2.0.

## 2. Ncss (Next-CSS): The Spec Evolution

Ncss is envisioned as a **Reactive Stylesheet Transpiler**. Its goal is to allow CSS to interact with the Nhtml state directly without requiring JavaScript glue-code.

### 2.1 Proposed Syntax (Semantic Variables)
Ncss will support native nesting and state-aware variables.
```ncss
@define theme_primary = nhtml.site_color || '#1a1a2e';

.main-button {
    background: {theme_primary};
    padding: 10px 20px;
    
    &:hover {
        brightness: 1.1;
    }
    
    @if (is_admin) {
        border: 2px solid var(--accent);
    }
}
```

### 2.2 Key Features
1.  **Reactive Styles**: Properties that update automatically when an Nhtml variable changes.
2.  **Conditional Styling (@if/@else)**: Apply style blocks based on application state.
3.  **Automatic Scoping**: Styles defined within a component are isolated to that component's UID, eliminating "CSS Leakage."

## 3. Nhtml v2.0: The Architectural Leap

As the NCMS Proof-of-Concept grows, we have identified areas for high-scale optimization. 

### 3.1 The Hybrid Virtual DOM
V2.0 will introduce a **Partial Virtual DOM** specially optimized for loops and recursive components. This will allow for ultra-high-speed updates even for tables with 10,000+ rows.

### 3.2 Component Life-Cycle Hooks
V2.0 will introduce native lifecycle attributes:
- `on:mount`: Triggered when the component is injected.
- `on:update`: Triggered when props change.
- `on:destroy`: Triggered before removal.

## 4. Short-Term Technical Roadmap (2026-2027)

| Milestone | Feature | Description |
| :--- | :--- | :--- |
| **v1.1** | **Hot Reload** | Live-updating of Nhtml templates without page refresh. |
| **v1.2** | **Ncss Alpha** | First release of the CSS-logic engine. |
| **v1.5** | **Native PWA** | Automatic Service Worker generation for NCMS. |
| **v2.0** | **The Hybrid Engine** | Full rewrite of the client runtime for high-scale performance. |

## 5. Vision: The "No-JS" Future

The long-term goal of the Nhtml ecosystem is to reach a state where **90% of dynamic web logic can be written without ever touching a .js file**. By expressing logic in the Markup and Style layers, we reduce the cognitive load for developers and the battery drain for mobile users.

## 6. Closing Statement

The Nhtml project is at a pivot point. With a stable NCMS proving the validity of our concepts, we are ready to scale our ambitions. The roadmap presented here is not just a list of features, but a commitment to maintaining the power of minimalist engineering in an era of digital excess.

---
*Nhtml Project Lead — April 2026*
