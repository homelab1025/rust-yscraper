## Context

The current `rust-yscraper` webapp is built using React and Material UI. While functional, the design is generic and doesn't provide a modern dashboard experience. The goal is to implement a new, custom design based on the provided prototype (`design/code.html` and `design/screen.png`), which uses a sidebar-based layout and Tailwind CSS for styling.

## Goals / Non-Goals

**Goals:**
- Implement a modern, responsive, and aesthetically pleasing UI based on the design prototype.
- Integrate Tailwind CSS into the project's build pipeline (Vite).
- Refactor the webapp's layout to use a sidebar for navigation and thread management.
- Add summary statistics cards to the main dashboard.
- Update the thread list table with improved status indications and icon actions.
- Ensure a clean and modern Light Mode experience.

**Non-Goals:**
- Backend API changes or refactoring.
- Modifying the core triaging logic (e.g., keyboard shortcuts `j`/`k`/`p`/`d`).
- Completely removing Material UI in one go (though its usage will be significantly reduced).

## Decisions

- **Tailwind CSS Adoption**: Tailwind CSS will be the primary styling method going forward. This provides a more flexible and developer-friendly way to achieve the custom design than the existing Material UI theme overrides.
  - *Alternatives considered*: Staying with Material UI and using extensive theme customization. *Decision Rationale*: Tailwind allows for a much closer match to the provided HTML/CSS design and reduces the dependency on a large UI library for simple styling needs.
- **Layout Architecture**: The `App.tsx` will be refactored to a flexible flexbox/grid layout that incorporates a persistent `Sidebar` and a scrollable `MainContent` area.
- **Global Theme Configuration**: Tailwind will be configured with the specific color palette (primary: `#ec5b13`, background-light: `#f8f6f6`, background-dark: `#221610`) and font (Public Sans) from the design.
- **Status Badges**: Implement a dedicated `StatusBadge` component that maps thread states (e.g., In Progress, Completed) to the specific visual styles (amber/emerald) defined in the design.
- **Icon Set**: Use Material Symbols Outlined, as specified in the prototype, ensuring consistency with the visual identity.

## Risks / Trade-offs

- **Risk**: Potential for styling conflicts if Material UI components are still mixed with Tailwind CSS.
  - *Mitigation*: Gradually replace Material UI components with custom Tailwind-styled equivalents, starting with the layout and primary navigation.
- **Risk**: Ensuring consistent responsiveness across different screen sizes.
  - *Mitigation*: Leverage Tailwind's mobile-first responsive utilities (`md:`, `lg:`, etc.) and test on common breakpoints.
- **Trade-off**: The shift to Tailwind requires adding new dependencies and build steps.
  - *Justification*: The improved design flexibility and reduced runtime overhead of CSS-in-JS (if used by MUI) justify the slightly more complex build setup.
