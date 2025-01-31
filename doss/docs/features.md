# Features
## Skipping
Skipping allows leaving out a number of bytes from parsing if you are not interested in in the content of the rest of THIS object/array anymore. In cases when you don't need certain parts of the structure -> why parse it?

Skipping can never leave the current nesting level (file, object, or array). But it does not need to go to the end of the level: it could just go anywhere in between and will continue parsing from there on. Possibly at that position is another skipping opcode.

Skipping does **not** use Varint encoding. This is due to the fact that the serializer needs to know how many bytes are reserved for the forward jump. So it reserves 2 or 4 bytes little endian encoded bytes. During serialization these bytes are 0 and can later be overwritten. 

## Streaming
Streaming refers to the idea that the file can be processed on the receiver side while it is not yet transferred completely. The special case is when the file is already processed while it is still being generated on producer side. 


# Edge cases
This section provides some features that are intended use cases but might come unexpected.

## Setting the same key twice
Dictionaries are key=>value assignments. DOSS deserializes the stream of opcodes. There is nothing that would prevent the serializer to write the same opcode several times into one object.
This is no problem for the streaming approach as the handler would be called twice and can decide what to do with it.
If the object is deserialized into a dictionary the deserializer should simply overwrite the old value with the new value. This behavior could be changed by the program.

## File names could be anything
The *start file* opcode has an attribute for the filename. But this is in no way required to be a string. It could be a number or even null. It is also not required to be unique in the file.
The recommended way is to treat file names of *null* as a new file with a newly assigned name.
For files with the same name these should overwrite any older file with the same name (except null).

# Opportunities
This section provides idea where DOSS can help to make data processing and storage efficient.

## Set a hint for memory requirements
The dictionary can grow as large as needed. Though DOSS also has the possibility to clear or overwrite entries, sometimes preallocating the necessary amount of RAM is more efficient.
On the writer side a hint could be given to the serializer to use only a maximum of dict size so that readers are still able to deserialize it properly.

## 0 is a valid jump target for skipping
It is fine to use the skipping opcode with a 0 skip bytes value. This is needed during streaming serialization where the nesting end might not be known.