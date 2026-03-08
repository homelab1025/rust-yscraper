# User Interface

The `rust-yscraper` user interface is designed for efficient management and triaging of Hacker News threads and comments.

## Layout and Navigation

The interface is built with a standard Material UI layout:
- **Navigation Bar**: A top bar provides quick access to the main sections of the tool:
    - **Links**: The primary page for managing tracked Hacker News threads.
    - **About**: A description of the tool and its purpose.
- **Content Area**: The main content area where pages and lists are displayed.

## Key Pages

### Link Management Page (`/links`)
The default landing page for the application. It allows users to:
- Add new Hacker News threads by item ID.
- View the list of currently tracked threads.
- Monitor comment counts (picked / total) for each thread.
- Navigate to the triage interface for "New", "Picked", or "Discarded" comments.
- Delete tracked threads and their associated comments.

### Comments Page (`/comments`)
The core triage interface for a specific thread. It includes:
- A paginated table of comments.
- Support for sorting by Date and Subcomment count.
- Filters based on comment state (New, Picked, Discarded).
- Visual indication of the currently selected row for keyboard navigation.

## Keyboard Shortcuts (Triage Mode)

The triage interface is optimized for speed using the following keyboard shortcuts:
- `j`: Select the **next** comment (move down).
- `k`: Select the **previous** comment (move up).
- `p`: Mark the selected comment as **Picked**.
- `d`: Mark the selected comment as **Discarded**.

### Scrolling Behavior
When navigating through comments with the keyboard, the interface automatically scrolls to keep the selected row (and the row below it when going down) visible. This ensures a seamless, high-speed triaging experience.

## Styling and Aesthetics
- **Theme**: Uses a light Material UI theme with a custom primary color (`#ff6600`) to match the Hacker News visual identity.
- **Responsiveness**: The interface is built to be responsive, using Material UI's grid system and container layouts.
