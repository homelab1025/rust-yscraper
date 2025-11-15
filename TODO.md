## TODO
1. add a web interface to this to list the comments
2. thru the web interface user should be able to select which comments to keep and which to discard
3. the discard and keep actions should be done thru keys (no mouse needed) for quick action
4. show a confirmation of whether the action was keep or discard and allow the user to change it
5. web interface should be able to list the kept comments
6. move to postgres

## Notes (To be ignored by AI)
In the comments table add the month(extracted from the thread title) as a separate column. If the url is scraped again, do an update on the new column like you do with the text and url reference.
/// Discard a comment (idea)
async fn discard_comment(&self, id: i64) -> Result<(), sqlx::Error>;

    /// Approve a comment (keep the idea)
    async fn approve_comment(&self, id: i64) -> Result<(), sqlx::Error>;

    /// Cleanup the database by removing discarded comments.
    async fn cleanup(&self) -> Result<(), sqlx::Error>;
## DONE