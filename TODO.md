## TODO

- add integration tests for the comments api. Use the postgresql_test.rs as an example. You can extract the setup of the db in a common module for all integration tests.
- write tests for the execute function of the ScrapeTask struct (from the trait). Consider: comments returns an error, comments is empty, repo fails to store -> should return error, but it doesn't right now.
- ideas could be grouped in clusters based on functional areas/domains
- generalize this for the "Who is hiring" threads as well
- create a web interface using reactjs with the following features:
  - (page) list the comments that have been scraped and provide the possibility to filter them.
    - user should be able to select which comments to keep and which to discard
    - the discard and keep actions should be done using the keyboard (no mouse needed) for quick action
    - show a confirmation of whether the action was KEEP or DISCARD and allow the user to change it
  - (page) list comments that are selected as kept. These are potential project ideas to try out new technologies.

## DONE

- The comment gets a new attribute named "state" which reflects whether the comment was picked or discarded. This can be encoded in a number in the application code. The user can pick or discard a comment by pressing the "p" for pick or "d" for discard key when the comment row is selected in the comments list. The list of links will show the number of picked comments out of the total number of comments for each link, in the comments count column. The picked and discarded comments will NOT be shown anymore in the comments list, but there will be a new link in the links table that will lead to a comments page where we show the picked comments. The same for discarded, but thru a different link. The distinction between these 2 views is based on the value of the "state" query parameter for the comments page.
- refresh the comments (scrape again) over time with a limit of X days, specified as `days_limit` in ScrapeRequest
- webapp
  - (page) list the links that have been submitted for scraping
  - deploy this webapp to k8s and serve it from an nginx server as static content
  - add an id for scraping
- delete the link together with the comments that come with it in a transaction
- deploy to k8s - docker imgs & kustomize
- separate the links repository trait from the comments repository trait
- move to postgres
- add tests for the handlers
- switch to postgresql
