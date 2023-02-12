# Plandustry
Plandustry is a command-line tool for editing [Mindustry](https://github.com/Anuken/Mindustry) schematics.

## Command-Line usage
The program must be run with a command line first determining the operation to perform, followed by arguments that depend on the operation.  
In general, arguments can either be literal (not starting with a dash) short form (starting with a single dash) and long form (starting with a double dash). In
all cases, arguments are space-delimited or can be wrapped in quotes (but internal quites are considered literal). Both short and long form arguments may be
passed a value (py), but differ in their syntax and interpretation. Short form arguments are only a single character long, but multiple (even duplicates) can
be used in the same group (following a single dash). This has the advantage that the value following it is passed to all arguments. For example:
- `-a -b -c=value` is 2 arguments with no value (`a` and `b`) and another argument (`c`) with value `value`
- `-ab -xyx` is 5 arguments with no value: `a`, `b`, 2 times `x` and `z`
- `-hello=world` is 5 arguments: `h`, `e`, 2 times `l` and `o` each (even the double `l`) with the value `world`
Long form arguments are simpler: they start with a double dash and may be arbitrarily long. The syntax for passing values is the same but there can only be one
argument per token, and the value only applies to it (such as `--long-form-arg=its-value`).  
Note that arguments can forbid, require or permit the inclusion of a value, and some may be used multiple times (as noted below).

### Print
The print command takes in schematics from the command line, from files or interactively and prints the name, tags, build cost and blocks contained in it.

| Argument | Description | Appears | Value |
| --- | --- | --- | --- |
| `literal` | A base-64 encoded Schematic to print | Optional, Repeatable | N/A |
| `-f`, `--file` | A path to a `.msch` file (binary schematic) to print | Optional, Repeatable | Required |
| `-i`, `--interactive` | Run interactively where base-64 encoded schematics are read from stdin and printed | Optional | Forbidden |

Note that interactive mode is the default if no literals or files are given, but to include it anyway is not an error.

### Edit
The edit command is an interactive prompt capable of loading, editing, printing and saving schematics.

| Argument | Description | Appears | Value |
| --- | --- | --- | --- |
| `literal` | A base-64 encoded Schematic to load | Optional | N/A |
| `-f`, `--file` | A path to a `.msch` file (binary schematic) to load | Optional | Required |

If the file argument is present, literals are ignored. After loading the given schematic (if any), the program enters interactive mode. Use "help" for a list
of available commands in interactive mode.
