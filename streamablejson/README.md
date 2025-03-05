# Streamable Json

*streamablejson* is a library to process JSON structures (nested dictionaries)
as an event stream. It allows input streaming of large data (e.g. via TCP)
and also allows skipping of parts of the input.

## Format

Being able to stream JSON is important when the size of the file is big,
should be processed during load - or simple does not fully exist while transferring.

We define a new format *streamablejson* (postfix *.sjson*) to differentiate it from JSON.
Every valid JSON file is also a valid streamablejson file. There are main differences:
- allow an object to contain the same field several times
- allow typed objects
- allow comments like // and /*..*/

### Typed objects
JSON allows some primitive types. But those are not sufficient in many cases.
Common example are DateTime values (commonly encoded as an object with type and 
milliseconds since epoch) or ObjectID (used in MongoDB and Azure CosmosDB).

One example in Azure CosmosDB would be

{
_id" : ObjectId("5ac205a4115db114402c7257"),
}

[BSON](https://bsonspec.org/spec.html) (binary JSON) itself offers an ObjectID type and
UTC timestamp type and some other types but no generic solution.

JSONS allows a type identifier followed by parentheses and another element in between,
which could be a string, a number, array or an object. 

The object can be used anywhere, even as keys in objects. Type identifiers can be nested. 


## Event processing
The library takes the idea of [StAX](https://en.wikipedia.org/wiki/StAX) (a Java
API for stream processing of XML files) and applies it to JSON.

The user defines a handler function which gets called for events like:
- start of an element
- closing of an element
- key/value assignment

The handler processes the event and handles internal state. It returns
a value to the reader which tells the reader:
- continue
- skip


## Skipping
In some cases you can figure out that a certain element in JSON does not bother
you anymore. At the same time parsing the element and forwarding events puts
load on the processor. *jsonStream* allows the processor to skip further
processing of an element. The processing will than continue at the closing
event of the current element without parsing further child elements.

