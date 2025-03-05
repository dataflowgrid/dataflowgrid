# ShoutOut - PubSub for status updates

*ShoutOut* is a way for a client to post updates and those be received by other users. The goal is **not** for the user to depend on the messages but rather to just use them as information.

## Usecase

Imagine that a client is uploading a big file. At the same time another user is waiting for that file to arrive. It would be nice for the user to have a feeling, how much progress the upload already did. While knowing this does not make a difference for the system overall it is nice-to-know for the user waiting and looking onto his webapp.

## Basic workflow

The sender connects to *ShoutOut* and generates a *stateobject*. The *stateobject* can have additional parameters like time-to-live and security.

The sender can new update the *stateobject*'s data or send messages to it.

At the same time any user client can register at the same *stateobject* by id as a receiver. Now every time the data was updated a message will be sent to the receiver. Also messages published onto the stateobject are forwarded to the receiver.

## Non-guarantees

As the system is just there for convenience, the guarantees are limited:

- the sender should not depend on the server and work just fine without it
- any workflow MUST NOT depend on messages or state in *ShoutOut*
- any receiver should not depend on the server and work fine without it
- the server may loose state at any time
- the server may loose messages at any time
- the server may throttle status update messages to clients (meaning a client only receives an update message every 10 seconds instead of every time)
- the server may throttle published messages (meaning the client will receive only some of the published messages)
- any client with permissions on the *stateobject*'s id can send messages at any time
- a status-updated message may arrive at the receiver after the *objectstate* was updated again, there is no history of the data

## Guarantees

*ShoutOut* gives some guarantees:

- every message and every status update contain an update id, messages are only processed if the update id is newer than the current state
- messages are delivered in order to the client (though the might get lost completely)

## Development

*ShoutOut* is a server written in Rust using the hyper library. It does not use any special web application framework.
Reason being that we only have a few functions and we offer a low level service.