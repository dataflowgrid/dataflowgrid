# Examples
This page contains examples from simple to more complex to display the features of DOSS serialization.

Every section contains a JSON example and how this *could* be serialized into DOSS format. Note that the serialization can be done in different forms depending on the realization of the serializer. 

## Hello World
```json
{ "hello": "world"}
```
In the simples form this could be serialized without any use of the dictionary:
```cpp #seems to work fine for bytes but it is of course no cpp code
10 start object 
07 utf8 string
05 length of string - varint
68 65 6c 6c 6f  'hello'
07 utf8 string
05 length of string - varint
77 6f 72 6c 64 'world'
11 end object
```

## Hello World with Dict

The whole point of using DOSS is of course to make use of the dictionary.
```json
{ 
  "hello": "world",
  "say": "hello"
}
```
This can be serialized as
```cpp 
10 start object 
21 store next item in dict : 0 ==> hello
07 utf8 string
05 length of string - varint
68 65 6c 6c 6f  'hello'
07 utf8 string
05 length of string - varint
77 6f 72 6c 64 'world'
07 utf8 string
03 length of string - varint
73 61 79 'say'
09 dict reference
00 dict entry 0
11 end object
```

## Hello World with bool and null
Bool and null value don't need to be put into the dictionary. They have their special opcodes:
```json
{ 
  "hello": true,
  "say": null
}
```

```cpp 
10 start object 
07 utf8 string
05 length of string - varint
68 65 6c 6c 6f  'hello'
01 constant true
07 utf8 string
03 length of string - varint
73 61 79 'say'
02 constant null
11 end object
```

## Nested objects
Bool and null value don't need to be put into the dictionary. They have their special opcodes:
```json
{ 
  "hello": {
      "say": "hello"
  }
}
```

```cpp 
10 start object
21 store next item in dict: 0 ==> hello
07 utf8 string
05 length of string - varint
68 65 6c 6c 6f  'hello'
10 start object
07 utf8 string
03 length of string - varint
73 61 79 'say'
09 dict reference
00 dict entry 0
11 end object
11 end object
```

## Arrays
```json
{ 
  "hello": ["say", "hello"]
}
```

```cpp 
10 start object
21 store next item in dict: 0 ==> hello
07 utf8 string
05 length of string - varint
68 65 6c 6c 6f  'hello'
12 start array
07 utf8 string
03 length of string - varint
73 61 79 'say'
09 dict reference
00 dict entry 0
13 end array
11 end object
```

## Settings and hints
Hints are like settings but can be ignored if the reader does not understand them. An unrecognized settings must lead to an error.

They can be inserted at any time into the stream but should be inserted at the beginning. The same setting can be inserted several times and is valid until changed.

The example sets the minimum version with an empty object:

```cpp 
20 set config
03 varint value
00 setting "minimum version"
03 varint value
01 version 1

10 start object
11 end object
```

## Change string encoding
DOSS deserializer can be configured to use another string encoding like ASCII or UTF16. This setting is value for alle string decoding from that time on.

```json
{ "hello": "world"}
```

```cpp 
20 set config
03 varint value
01 setting "string encoding"
07 string value - still utf8
06 length of string
41 53 43 49 49 "ASCII"

10 start object
07 ASCII string
05 length of string - varint
68 65 6c 6c 6f  'hello'
07 ASCII string
05 length of string - varint
77 6f 72 6c 64 'world'
11 end object
```

## Importing predefined dictionaries
```json
{ "hello": "world"}
```
Assume that we have a predefinied dictionary that contains the words *hello* (0) and *world* (1). Then this dictionary can be loaded and entries referenced directly. We define that this dictionary has the name *hello_world*. The deserializer must know about this in advance and it must make sure that serializer and deserializer use the same version. Enforcing this is out of scope for DOSS. 

```cpp 
40 import into dict
07 string value
06 length of string
68 65 6c 6c 6f 5f 77 6f 72 6c 64 "hello_world"
10 start object
09 dict reference
00 reference 0 --> "hello"
09 dict reference
01 reference 1 --> "world"
11 end object
```

## Multiple files
You can store several files into one serialization stream quite similar to the delimiter --- in YAML
Let's assume you want to write the hello world example to two files. Only the second file has a name. It can still use the same dictionary.

```cpp 
10 start object 
21 store next item in dict : 0 ==> hello
07 utf8 string
05 length of string - varint
68 65 6c 6c 6f  'hello'
21 store next item in dict : 1 ==> world
07 utf8 string
05 length of string - varint
77 6f 72 6c 64 'world'
11 end object

# next file
50 start file
07 utf8 string
08 length of string - varint
65 78 61 6d 70 6c 65 32 'example2'
09 dict reference
00 dict entry 0
09 dict reference
00 dict entry 1
11 end object
```

## Skipping data

```json
{ 
  "hello": "world",
  "say": "hello"
}
```
This can be serialized as
```cpp 
10 start object 
30 skip bytes to go the end of the object
00 16 bytes till end of object

21 store next item in dict : 0 ==> hello
07 utf8 string
05 length of string - varint
68 65 6c 6c 6f  'hello'
07 utf8 string
05 length of string - varint
77 6f 72 6c 64 'world'

07 utf8 string
03 length of string - varint
73 61 79 'say'
09 dict reference
00 dict entry 0
11 end object
```
As the deserializer now knows where the end of the object is it could skip from any pointer after to the end.
But the skipping does not need to go to the end of the object. It can as well go to any other point within the current object/array as long as the parsing in between does not change the parsing of the rest of the stream. It could find another skipping opcode at that point.