# jsonStream

*jsonStream* is a library to process JSON structures (nested dictionaries)
as an event stream. It allows input streaming of large data (e.g. via TCP)
and also allows skipping of parts of the input.

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

