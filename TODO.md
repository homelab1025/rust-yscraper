## TODO
- add tests for the handlers
- add webpage for URLs
- refresh the comments (scrape again) over time with a limit of X days
- deploy to k8s - docker imgs & kustomize
- thru the web interface user should be able to select which comments to keep and which to discard
- the discard and keep actions should be done thru keys (no mouse needed) for quick action
- show a confirmation of whether the action was keep or discard and allow the user to change it
- web interface should be able to list the kept comments
- move to postgres

## Notes (To be ignored by AI)
In the comments table add the month(extracted from the thread title) as a separate column. If the url is scraped again, do an update on the new column like you do with the text and url reference.
/// Discard a comment (idea)
async fn discard_comment(&self, id: i64) -> Result<(), sqlx::Error>;

    /// Approve a comment (keep the idea)
    async fn approve_comment(&self, id: i64) -> Result<(), sqlx::Error>;

    /// Cleanup the database by removing discarded comments.
    async fn cleanup(&self) -> Result<(), sqlx::Error>;
## DONE
- switch to postgresql