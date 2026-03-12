## Why

The current user interface uses a standard Material UI layout which, while functional, lacks a modern and custom feel. Updating the design to match the provided prototype will improve the user experience by providing a more visually appealing and organized dashboard, better supporting the triaging process with a cleaner layout and clearer status indications.

## What Changes

- **Visual Overhaul**: Replace the Material UI-based design with a custom look based on the provided design prototype.
- **Layout Change**: Introduce a sidebar for navigation and link management (Add New Link), moving away from the top navigation bar.
- **Dashboard Stats**: Add summary statistics (Reviewed Comments, Picked Ideas, Discarded Ideas) to the main dashboard.
- **Table Redesign**: Update the "Task Control Center" table with a more modern aesthetic, including status badges and icon-based actions.
- **Technology Update**: Transition from Material UI-specific styling to Tailwind CSS for more flexible and lightweight styling, while keeping React for component logic.

## Capabilities

### New Capabilities
- `dashboard-summary`: A new component to display aggregate statistics of the triaging progress across all tracked threads.

### Modified Capabilities
- `user_interface`: Update the overall layout, navigation, and page designs to follow the new aesthetic. Requirements for responsive behavior and keyboard navigation remain unchanged, but the visual implementation will be completely overhauled.

## Impact

- **Webapp**: Significant changes to `App.tsx`, `components/`, and `pages/`.
- **Dependencies**: Add Tailwind CSS to `webapp/package.json`. Potentially reduce reliance on Material UI (`@mui/material`).
- **Styling**: Shift from Material UI theme configuration to Tailwind CSS configuration.
