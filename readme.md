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



## Syntax

_Note that most of the following is yet unimplemented_, **and wouldn't actually work**.
This is currently just a (bad) draft.

### Commands

Rush, as you might know, is a shell. You interact with it by entering commands.
A command consists of the name of the command followed by arguments.

Arguments can be:

 * [strings](#strings)
 * [streams](#piping)
 * [blocks](#blocks)

This is what a command looks like:

```Rush
echo hello world
```


### Strings

Strings can be written between single quotes, which helps if there is whitespace.

```Rush
rm 'cumbersome filename.txt'
```


### Variables

Variables are lists, which can contain arguments, ie: [strings)](#strings), [blocks](#blocks) and [stream handles](#piping).

They can be set using the `set` command.

```Rush
set my_variable 'I am a string'
set another_variable this is a list containing 7 strings
set mixed_types 'Here is a string' {echo this is a block}
```

When used in a command, they are substituted in-place.

```Rush
mkdir $my_variable # creates a directory 'I am a string'
mkdir $another_variable # creates 7 directories: "this", "is", "a", "list", "containing", "7" and "strings"
```


### Blocks

In Rush, blocks and anonymous functions are the same thing. They are used do define a scope, and, when passed to another command, to delegate the execution of a set of commands to another command.

They can be passed to commands.
A great example is in a `if`.

```Rush
if [true] {
	echo it works !
}
```

They can be set to a variable with `set`, so it is possible to reuse them (which is equivalent to a function declaration in most languages).

```Rush
set bar {
	set a foo
	echo $a # foo
}

$bar # foo

echo $a # Error !
```


### Piping

Commands return two things: an exit status and an ostream (we will discuss the exit status later).
They also accept input via the stdin.

The program's stdin and stdout is merged to form the **ostream**.
By default, the ostream is written to the terminal, and the stdin is user input from the terminal.

However, it is possible to redirect a command's ostream to another command's stdin by using a pipe `|`.

```Rush
printf foo\nbar | wc -l # 2
```

To prevent a command's stdin and the stdout to merge, you can use the command `strm`.

Its options are:

 * `-d`: discard
 * `-s`: set ostream

In the following example, `ls non-existent_file` is being run with only stderr as its ostream.
That ostream is then given to `dd` as its stdin, which is subsequently written to the file `error.txt`.

```Rush
strm -d stdin -s stderr {ls non-existent_file} | dd of=error.txt status=none
```

`strm` also returns a list containing handles to the block's stderr and stdin, which might be used to set the stream handles to a variable.

```Rush
set $stdstrm {strm {
	ls non_existent_file # error -> this goes to the stderr
	echo that goes to the stdin
}}

echo This is the stderr: $stdstrm[0]
echo This is the stdin: $stdstrm[1]
```


### Subcommands

Subcommands are used to obtain a command's ostream and pass it as an argument.

#### Stream

Except when passed to the special commands `echostrm` and `set`, when a stream is passed to a command, it is converted into a list of string by collecting it whole, then separating it at every encountered newlines.

```Rush
mkdir (echo foo) # creates a folder foo
```

When the output contains a literal newlines, it is split into a list of strings.

```Rush
mkdir (echo foo bar) # creates a folder "foo bar"
mkdir (printf foo\nbar) # creates two folders: "foo" and "bar"
```

#### Exit status

The exit status can be collected in a string with `[` and `]`.

```Rush
echo [true] # 1
```

You will see that this is especially useful for conditions.

#### Both

This emits a variable which contains the exit status as the fist item and the stream at the second item.

```Rush
echo ([echo foo])
```


### `join`

The first argument is the separator.

```Rush
echo (join \t one two three)

set foo bar baz
set yay (join \v $foo)
```


### `split`

The first argument is the separator.

```Rush
set foo (split ' ' 'wow this command is awesome!')
```


### `echostrm`

Except with `echostrm` (and `set`), when a stream is passed as a parameter to a command, the shell waits for it to end, then collect the list of strings it emitted, and pass it as arguments.
With `echostrm`, the stream is emitted as its ostream in real-time.

```Rush
set $stdstrm {strm {
	ls non_existent_file # this goes to the stderr
	echo this goes to the stdin
}}

echostrm $stdstrm[1] | cat
```

When more than one handle to a stream is given as a parameter, `echostrm` merges them.

```Rush
set $stdstrm {strm {
	ls non_existent_file # this goes to the stderr
	echo this goes to the stdin
}}

echostrm $stdstrm | cat
```

### `parallel`

This is useful when two commands can be parallelized.
```Rush
parallel
	{
		process_one
	}
	{
		process_two
	}
```

But the main use is for using a process's stdout and stderr in two different commands, at the same time.

```Rush
set $stdstrm {strm {
	ls non_existent_file # this goes to the stderr
	echo this goes to the stdin
}}


parallel
	{
		foo $stdstrm[0]
	}
	{
		bar $stdstrm[1]
	}
```
