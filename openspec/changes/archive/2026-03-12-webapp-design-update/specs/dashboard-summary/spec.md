## ADDED Requirements

### Requirement: Dashboard Summary Statistics
The system SHALL display summary statistics for all tracked threads on the dashboard. These statistics include:
- **Reviewed Comments**: The total number of comments that have been either Picked or Discarded across all tracked links.
- **Picked Ideas**: The total number of comments in the "Picked" state across all tracked links.
- **Discarded Ideas**: The total number of comments in the "Discarded" state across all tracked links.

#### Scenario: Display aggregate stats on the dashboard
- **WHEN** the user navigates to the dashboard (links page)
- **THEN** the summary cards show the total counts for Reviewed, Picked, and Discarded comments.

### Requirement: Real-time Update Indication
The dashboard SHALL include a "Live Update" indicator to inform the user that the statistics and thread list are being updated.

#### Scenario: Visual indicator for live updates
- **WHEN** the user is on the dashboard
- **THEN** a "Live Update" badge is visible with a subtle animation (e.g., a pulsing dot).
