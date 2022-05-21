# Scrum

Ticket tracking software using a terminal user interface (tui-rs). 

This is my first attempt at making a useful application, particularly for myself because I sorely miss having a ticketing system at my current job.  I'm trying to balance doing things right with getting a working, safe, and usable prototype.  I then plan to go back and try to make the code as clean and efficient as possible, which I believe will be very helpful as a learner.

## To-do / Issues

### High Priority
- Allow notes to be edited, removed, or moved from one ticket to another.  To be done in the Edit ticket form, but may also add a notes only view.
 
- Move delete function to ticket edit screen (currently only available on closed ticket view)

- Allow scroll on ticket description

- Add length limit to fields and more robust string checking for adding/editing tickets

- Allow left right cursor movement on inputs

- More intuitive key inputs

- Serious refactoring, removal of inefficiencies and excess checks

- Cursor location does not work when text is wrapped

- Enable cursor for edit form

- Closing/Opening ticket puts it at the bottom of the respective list

- Need to use proper enums for things like priority.  Way too much manual code that will break if something is added.


### Medium Priority:

- Enable Sorting (will require revisit of indexing)

- Database backup feature

- Proper database pathing (and possible user settings)

- How selection between open and closed works is probably terrible and should be redone.  It currently relies on way too many checks to keep system afloat, there is definitely a way cleaner way to do it.  This will go in tandem with adding filters/incorporating a notes view.

- Revisit how indexing works, particularly with open/closed tickets

- Add assignee and possible contact field

### Low Priority

- Database selection and management tools

- Client-Server integration

- HTML display

- E-mail integration for making tickets

:jp:
