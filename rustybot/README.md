# rustybot

rustybot is a reimplementation of rustbot in Rust.

## Usage

rustybot commands can be invoked via a standard syntax. Commands are marked by a command identifier string (default ~) followed by the command name and arguments. Commands must either start at the beginning of an irc message (e.g. ~crate libc) or be contained withing curly braces (e.g. "this is a normal message {~crate libc}").

In addition to the standard set of commands rustybot can be taught factoids which act as pseudo commands that will cause rustybot to respond with a specified string.

## Current Status
### Commands
- learn
- forget
- rfc
- crate
- qotd
- error
- hresult
- ntstatus
- win32
- lock
- unlock

## Development
### Prerequisites
- rustc (either via rustup or your distributions package manager)
- cargo (via the same method as above)
- sqlite3

### Building
- `git clone https://github.com/ZoeyR/nestor.git`
- `cd rustybot`
- `cargo build
