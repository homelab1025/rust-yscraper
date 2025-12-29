### YScraper Web Application

The YScraper web application is a dashboard designed to manage and filter comments scraped from various sources (like Hacker News).

### Functionality

There are 2 main functionalities of the web application: viewing the links that have been scraped and keeping or discarding comments. These are the primary features that are located in different sections of the dashboard.

#### Link Management
- **Link Listing**: display the list of links that have been scraped. For each link displays the title, the URL, and the date it was last scraped. It also displays the status of the link (scraped, failed, or pending).
- **Add**: Allows users to add new links to the database, and they will be automatically scheduled for scraping.

#### Comment Management
- **Comment Listing**: Displays a paginated list of comments in a structured table. Each entry includes the comment text, the author, the associated URL ID, and the date it was posted.
- **Filtering Status**: The dashboard specifically focuses on "Not Filtered Comments," providing a workspace for users to review new data.
- **Pagination**: Supports navigating through large sets of comments with page-based navigation and a total record count.

#### User Interaction & Navigation
- **Keyboard Navigation**: Optimized for speed, allowing users to navigate the list using standard keyboard shortcuts:
  - `j`: Move selection down to the next comment.
  - `k`: Move selection up to the previous comment.
- **Comment Marking**: Users can quickly categorize comments directly from the keyboard:
  - `a`: Mark the selected comment (visualized in blue).
  - `d`: Mark the selected comment for removal or rejection (visualized in red).
- **Interactive Selection**: Supports row selection via mouse clicks, which automatically focuses the view on the chosen item.

#### Data Integration
- **Real-time Loading**: Fetches comments dynamically from the backend API.
- **Visual Feedback**: Provides loading states while fetching data to ensure a smooth user experience.
- **Auto-Scrolling**: Automatically keeps the selected comment within the visible area of the table during keyboard navigation.