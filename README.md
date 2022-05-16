# Scrum

Ticket tracking software using a terminal user interface (tui-rs).  


## To-do / Issues

### High Priority
- Fix crash bug (indexing issue).  Can reproduce with fresh start, create ticket, close ticket 1 then 0 then open 1 then 0 then close 0 and it crashes, likely due to order/indexing.  Might crash when trying to re-open 0 before 1 as well if it ends up lower in the list.

- Add length limit to fields and more robust string checking for adding/editing tickets

- Allow left right cursor movement on inputs

- Create new input form with all required entries that can be tabbed through

- Convert priority input to list item

- More intuitive key inputs

- Serious refactoring, removal of inefficiencies and excess checks

- Checks for whether a ticket is selected or not isn't working like I imagined (if let Some(selected) = app.ticket_list_state.selected() {).  Currently checking length of open or closed instead (if not empty)

- Cursor location does not work when text is wrapped

- Add notes, which can be posted to a ticket (rather than having to update the description with changes)


### Medium Priority:

- Database backup feature

- Proper database pathing (and possible user settings)

- How selection between open and closed works is probably terrible and should be redone.  It currently relies on way too many checks to keep system afloat, there is definitely a way cleaner way to do it.



### Low Priority

- Database selection and management tools

- Client-Server integration

- HTML display

- E-mail integration for making tickets

:jp:
