# refer

A foundational library for building graphs out of structs.

## Goal
The main goal of this library is to provide foundation for programs
that are modeling graph like problems.

Primary attribute of the library is composability for achieving code reuse,
flexibility, and zero-cost abstractions or at least zero-overhead principle.

Secondary attribute is performance of memory and computation. This is achieved
by enabling optimizations, and then optionally providing implementations
that exploit them.

## Features
- Reference management, through Item and Shell family of traits.
- Memory management, through Container family of traits.
- Access management, through Collection family of traits.

## Architecture
There are several pieces that interact/are composable with one another:
- Model - the what's being build using the library.
- Items - the building blocks of the model.
- Shells - associated to each item and used to record references to its item.
- Collection - provides access to contained items and shells of a model.
- Containers - stores/contains items and shells.
- References - a reference to an (item,shell) that is supposed to be tracked and valid.
- Ids - provide identity management which ties all of the other pieces together.

Ids and references are concrete types that are not intended to be extended and are provided
by the library.

Collections, containers, and shells, are traits that are intended to be implemented
for the model if some of the provided generic implementations are not sufficient.

Items are traits that are intended to be implemented for the model.

Models aren't represented in any way by the library besides providing some examples/generic implementations.

## Unsafe
Internally the library uses unsafe code to allow type level separation of access to separate collections of
objects that are intertwined in memory. With that it's possible to have mutable access to all items and all shells
separately and at the same time even though in memory an item and its shell can be one by the other.


## Nightly
Some of the nightly features are crucial for the library to work, so until they are stabilized, nightly is required.

## Examples
TODO

## Development
The library is a work in progress and not yet fully verified.