## 1. Setup and Configuration

- [x] 1.1 Install Tailwind CSS, PostCSS, and Autoprefixer dependencies in `webapp/package.json`.
- [x] 1.2 Initialize Tailwind CSS configuration (`tailwind.config.js`) with custom colors, fonts, and dark mode support.
- [x] 1.3 Add Public Sans and Material Symbols Outlined font links to `index.html`.
- [x] 1.4 Configure PostCSS in `webapp/vite.config.ts` or `postcss.config.js`.
- [x] 1.5 Create a global CSS file (`index.css`) with Tailwind directives and Public Sans font-family.

## 2. Layout and Sidebar Implementation

- [x] 2.1 Create a new `Sidebar` component in `webapp/src/components/` with navigation links and the "Add New Link" form.
- [x] 2.2 Refactor `webapp/src/App.tsx` to use the new sidebar-based layout with a persistent sidebar and a scrollable main content area.
- [x] 2.3 Implement the responsive "Mobile Header" for smaller screen sizes as seen in the design prototype.
- [x] 2.4 Style navigation links with hover and active states using Tailwind classes.

## 3. Dashboard Summary and Statistics

- [x] 3.1 Create a new `SummaryStats` component to display the Reviewed, Picked, and Discarded comment cards.
- [x] 3.2 Update `App.tsx` or the main dashboard page to calculate and pass aggregate statistics to `SummaryStats`.
- [x] 3.3 Add the "Live Update" badge with a subtle animation to the dashboard header.

## 4. Thread List (Task Control Center) Update

- [x] 4.1 Redesign the `Links` table in `webapp/src/components/` (likely in `LinksTable.tsx` or similar) to match the "Task Control Center" design.
- [x] 4.2 Create a reusable `StatusBadge` component for the "In Progress" and "Completed" states.
- [x] 4.3 Replace standard Material UI buttons in the table with the icon-based action buttons using Material Symbols Outlined.
- [x] 4.4 Style the table with Tailwind, including hover states and appropriate padding/spacing.

## 5. Comments Page Update

- [x] 5.1 Refactor `CommentsPage.tsx` to use the shared header+sidebar layout and remove all MUI imports.
- [x] 5.2 Add a "← Links" back-link at the top of the comments main content area.
- [x] 5.3 Implement custom sortable column headers using Material Symbols icons (`unfold_more`, `expand_less`, `expand_more`).
- [x] 5.4 Rewrite `CommentRow` to use Tailwind classes: default/picked (`bg-emerald-50 border-l-2 border-emerald-400`)/discarded (`bg-slate-50 border-l-2 border-slate-300`) row states.
- [x] 5.5 Apply selected row highlight: `bg-primary/10 ring-1 ring-inset ring-primary/20` via a conditional Tailwind class.
- [x] 5.6 Replace MUI Pick/Discard buttons with icon buttons using `check_circle` and `cancel` Material Symbols icons.
- [x] 5.7 Replace `TablePagination` with a custom footer row matching the dashboard table style ("Showing X of Y comments" + Previous/Next).
- [x] 5.8 Add a keyboard hint bar below the table: `j/k navigate · p pick · d discard`.

## 6. Visual Audit and Final Polish

- [x] 6.1 Conduct a comprehensive visual audit to ensure all text and background colors match the design prototype.
- [x] 6.2 Test responsive behavior and adjust Tailwind breakpoints as needed for a seamless experience.
- [x] 6.3 Remove unused Material UI components and styles to reduce bundle size.
