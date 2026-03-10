## ADDED Requirements

### Requirement: Modernized Layout with Sidebar
The system SHALL transition from a top-nav based layout to a sidebar-based layout for navigation and thread management. The sidebar MUST include:
- **Logo and Title**: "What HN is working on" with a terminal icon.
- **Navigation Links**: "LINKS" and "ABOUT" as the primary navigation.
- **Add New Link section**: A dedicated section in the sidebar with an "Item ID" input field and an "ADD" button.

#### Scenario: Navigate between LINKS and ABOUT
- **WHEN** the user clicks "LINKS" or "ABOUT" in the sidebar
- **THEN** the main content area updates to the corresponding page.

#### Scenario: Add a link from the sidebar
- **WHEN** the user enters a valid HN Item ID and clicks "ADD" in the sidebar
- **THEN** the system initiates the link tracking process.

### Requirement: Enhanced Thread Management Table (Task Control Center)
The thread list table SHALL be redesigned as the "Task Control Center" with the following columns:
- **ID**: The unique HN item ID.
- **Month**: The thread's "Month Year" as a clickable link to its Hacker News page.
- **Date Added**: The date the thread was first tracked.
- **Comments (Count/Total)**: A bold "Count" of picked comments vs. the total number of comments.
- **Status**: A badge indicating the current state of the thread (e.g., "In Progress", "Completed").
- **Actions**: A centered group of icon-based action buttons (Schedule Refresh, View Picked, View Dropped, View Remaining, Delete).

#### Scenario: Visual status indication
- **WHEN** a thread is being scraped or partially reviewed
- **THEN** it displays an "In Progress" badge (amber).

#### Scenario: Thread completion indication
- **WHEN** all comments for a thread have been triaged
- **THEN** it displays a "Completed" badge (emerald).

### Requirement: Comments Page
The comments page SHALL follow the same visual design language as the dashboard. It MUST:
- Use the same header and sidebar layout as the main dashboard.
- Display a back link ("← Links") in the top-left of the main content area, navigating to `/links`.
- Show a page title with the thread's month/year (e.g. "Comments — December 2025").
- Render a table with columns: Comment, Author, Subcomments (sortable), Date (sortable), Actions.
- Sort headers MUST use Material Symbols icons: `unfold_more` when inactive, `expand_less`/`expand_more` when active.
- Each row MUST be styled by its comment state:
  - **New**: default white background.
  - **Picked**: `bg-emerald-50` with a `border-l-2 border-emerald-400` left accent.
  - **Discarded**: `bg-slate-50` with a `border-l-2 border-slate-300` left accent.
- The currently selected row (via keyboard navigation) MUST show a `bg-primary/10` background tint with a subtle inset ring.
- Action buttons in each row MUST use Material Symbols icons: `check_circle` (pick, hover emerald) and `cancel` (discard, hover rose), consistent with the dashboard's action icon style.
- The date cell MUST display the state label inline in muted text (e.g. "Dec 1, 2025 · picked").
- A footer row MUST show "Showing X of Y comments" and Previous/Next pagination controls, matching the dashboard table footer style.
- A keyboard hint bar MUST appear below the table, showing: `j/k navigate · p pick · d discard`.

#### Scenario: Selected row highlight
- **WHEN** the user navigates rows with `j`/`k`
- **THEN** the selected row shows a primary-color background tint to indicate focus.

#### Scenario: Row state color
- **WHEN** a comment has been picked or discarded
- **THEN** the row background reflects its state with the corresponding color accent.

### Requirement: Tailwind CSS Integration
The entire user interface SHALL be built using Tailwind CSS for styling to allow for precise control over the design and a more lightweight bundle compared to the current Material UI implementation. The React components MUST be updated to use Tailwind utility classes.

#### Scenario: UI responsiveness
- **WHEN** the user resizes the browser window
- **THEN** the layout adapts using Tailwind's responsive grid and container system.


