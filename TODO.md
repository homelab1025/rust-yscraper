## TODO
1. add a web interface to this to list the comments
2. thru the web interface user should be able to select which comments to keep and which to discard
3. the discard and keep actions should be done thru keys (no mouse needed) for quick action
4. show a confirmation of whether the action was keep or discard and allow the user to change it
5. web interface should be able to list the kept comments
6. move to postgres

## Notes (To be ignored by AI)
Switch the main function to being synch and start a tokio runtime and block on it. Inside of that start an axum http server that serves a ping GET resource (response is "pong"). The scraping is done upon calling the "/scrape" resource with a POST http method by a client.

Add an axum server with a GET ping resource. This server is the first thing that starts and it's independent of the scraping. Minimal features for new crates being added. The response for the ping resource should be pong.
## DONE