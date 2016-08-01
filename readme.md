# Rush

A simple shell written in Rust.
You will need the nightly compiler in order to build it.

## Design rationale

Even though most of what I plan for rush is not yet implemented, I want it to go in a clear direction.

The following are the three qualities Rush aims to have, which are intricated.
They are listed in order, from the most important to the least. When compromises have to be made, this hierarchy is respected.

### Usability
Rush is a shell. It must be easy to use for shell-related use, including shell scripting.
Many shells based on an existing language commit the error of not being adapted for shell use, for example by requiring parentheses in command calls.

### Clarity
Useless verbosity must be avoided. For example, there is no reason there should be a `do` after a `for` statement, as with the POSIX shell.
But terseness can also make things unclear. For example the variable `$IFS` (in POSIX shell scripting, too).
As a sidenote, Rush does not attempt to be backward compatible with the POSIX shell.

Rush aims to have a simple syntax. It prefers new commands over new syntax, except when it makes it less usable.

### Concision
Humans are lazy, the less the user has to type to get the work done, the best.
If it doesn't make it hard for the user to reason about what they are doing (*cough* GolfScript *cough*), Rush should be as concise as possible, especially for features used very often.
