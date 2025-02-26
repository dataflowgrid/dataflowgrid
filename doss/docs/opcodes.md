# Opcodes

This page contains all the defined opcodes.

| Opcode  | description |
| :------: | :----------- |
| 0        | noop (this is just ignored) |
| 1        | constant *true*  |
| 2        | constant *false*  |
| 3        | signed varint value    |
| 4        | decimal value    |
| 5*       | float value      |
| 6*       | datetime value   |
| 7        | string value (default utf8) |
| 8*       | binary string value     |
| 9        | dict reference   | 
| 10        | start Block      |
| 11        | end Block        |
| 12        | start array      |
| 13        | end array        |
| 14        | constant *null*  |
| 15        | unsigned varint value |
| 20       | set config   |
| 21       | store next item in dict |
| 22       | store next thing in dict but don't use as item |
| 23       | set dict pointer |
| 24       | clear dict entries |
| 25       | start stack      |
| 26       | leave stack      |
| 27       | set hint (like config but can be ignored) |
| 30       | skip bytes (16 bits LE)      |
| 31       | skip bytes (32 bits LE)      |
| 40       | import into dictionary |
| 50       | start file |

entries marked with * are currently not supported