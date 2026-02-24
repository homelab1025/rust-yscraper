## TODO
- ideas could be grouped in clusters based on functional areas/domains
- create a web interface using reactjs with the following features:
  - (page) list the comments that have been scraped and provide the possibility to filter them.
    - user should be able to select which comments to keep and which to discard
    - the discard and keep actions should be done using the keyboard (no mouse needed) for quick action
    - show a confirmation of whether the action was KEEP or DISCARD and allow the user to change it
  - (page) list comments that are selected as kept. These are potential project ideas to try out new technologies.
## TEMP
the comment gets a new attribute named "state" which reflects whether the comment was picked or discarded. This can be encoded in a number in the application code. The user can pick or discard a comment based on the 

## DONE
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