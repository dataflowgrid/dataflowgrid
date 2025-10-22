# System context

_dataflowgrid_ is a data management system for large files.


# Requirements

- we manage large files in the GB to TB range
- users can interact with us easily
- users do not need to know or handle any internals
- users can only access files which they are allowed to by their current usecase
- every usage of a file (or part thereof) is tracked
- non-interactive processes can only access those
  files which were marked for input
    - this includes containers and program libraries
- processes can mark themselfes to be _pure_ functions, meaning there output will be exactly the same if executed again
- the system will decide what, when and where to execute processes

# Non-Requirements

Certain tradeoffs are explicitly made:

- execution time is important but whether we don't care of the processing runs a few seconds more or not
- we handle millions of large files - but not billions of small files
- we expect a maximum of 10k parallel users
