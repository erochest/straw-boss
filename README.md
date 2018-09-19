
# straw boss

This parses Procfiles and dumps out information about them, the service names and command-lines that they define.

## Some notes on architecture

### Task

`yamlize` is the current example of this.

This contains data as read from and written to disk.

### Server

This is the `start` command. It starts up the server and passes options to it.

This contains a server, which owns a listener, and types for running and reporting on jobs.

### Client

This is pretty much everything else.

This contains a stream and consumes reporting types.

## Domain

### Service

Has information about a command to run (name and the command).

### ServiceProcess

A service with a handle to the executing process.

### Server

Has a manager and handles communication with it.

### Client

Communicates with a server to get information about a manager.
