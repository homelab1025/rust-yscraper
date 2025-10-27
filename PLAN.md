# Plan

## Change flow
- start the TUI first

## TUI interface
Add a ratatui interface for listing the comments from the db. Show the text only for each comment. If the text is longer than the terminal line use soft-wrap so that i don't have to scroll lateraly to read the whole text. Also, the date and author should appear in a dialog if i press "i". For navigation, use j and k like in vi or the UP and DOWN keys. If i press ENTER the comment text should be displayed in a dialog. Upon ESC pressed any dialog will dissapear.

Add more vertical space between the comments (1 line of empty text) and make it zebra like (black and some grey on which the text is clearly visible).

The TUI should be initialized after the db inserts and upon initialization it should load the comments from the db. It should be clearly separated from the parsing of the data, into a separate file (tui.rs).