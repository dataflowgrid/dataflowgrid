# dataFlowGrid Commons

Common datastructures that are used in several projects
and could be useful outside of this project

## CursedBuffer

A buffer with read and write cursor marks and dynamic capacity.

The name is referencing *Cursor* but with special complex use cases.

## Read and write guards

Access to buffer content is using *Guards*. A guard can be used to get a reference to the underlying buffer.




## The several buffer positions

*CursedBuffer* uses 4 different positions to track the state of the buffer:

- low read position

