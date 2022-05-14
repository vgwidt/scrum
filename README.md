# scrum

Ticket tracking software using a terminal user interface (tui-rs).



# to-do
High Priority

Add length limit to fields and more robust string checking for adding/editing tickets

Allow left right scroll on inputs

Create new input form with all required entries that can be tabbed through

Convert priority input to list item

More intuitive key inputs

Serious refactoring, removal of inefficiencies and excess checks

Checks for whether a ticket is selected or not isn't working like I imagined (if let Some(selected) = app.ticket_list_state.selected() {).  Currently checking length of open or closed instead (if not empty)


Medium Priority:

Database backup feature

Proper database path

How selection between open and closed works is probably terrible and should be redone.  It currently relies on way too many checks to keep system afloat, there is definitely a way cleaner way to do it.



Low Priority

Database selection and management tools

Client-Server integration

HTML display

E-mail integration for making tickets
