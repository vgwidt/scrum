# Scrum

Ticket tracking software using a terminal user interface (tui-rs).  


## To-do / Issues

### High Priority
- Add length limit to fields and more robust string checking for adding/editing tickets

- Allow left right cursor movement on inputs

- Create new input form with all required entries that can be tabbed through

- Convert priority input to list item

- More intuitive key inputs

- Serious refactoring, removal of inefficiencies and excess checks

- Checks for whether a ticket is selected or not isn't working like I imagined (if let Some(selected) = app.ticket_list_state.selected() {).  Currently checking length of open or closed instead (if not empty)

- Cursor location does not work when text is wrapped

- Add notes, which can be posted to a ticket (rather than having to update the description with changes)

- Sorting


### Medium Priority:

- Database backup feature

- Proper database pathing (and possible user settings)

- How selection between open and closed works is probably terrible and should be redone.  It currently relies on way too many checks to keep system afloat, there is definitely a way cleaner way to do it.

- Better indexing to determine selected item

- If menus exactly the same, I could set an AppState variable that sets the amount of expected messages to save from having to create different forms


### Low Priority

- Database selection and management tools

- Client-Server integration

- HTML display

- E-mail integration for making tickets

:jp:
