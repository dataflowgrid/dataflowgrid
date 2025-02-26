# GridAuthX - Authentication Exchange

## Overview

Data is precious - only authorized people should use it in most cases. Several authentication provider exist which
promise to prove that only verified people can log in.

But just because you know *WHO* a person is does not mean that the system knows which data he can access. Further restrictions
could apply like
- on a subset (by type or attribute or links) of the data may be allowed
- only if the person is at a certain location/network
- only at certain times
- certain data can only be accessed by Technical Users


Furthermore, as data should be shared, different people come from different Authentication Providers. Their tokens
will then be exchanged by *GridAuthX* into a token that can be used to access data.

## Policy storage
Policies are stored in *Enstate*. Access for technical users is always granted on a certain version of *Enstate*.
Access for personal users can be given on all versions (meaning latest + history).

This also means that once an access was given an a certain version is cannot be revoked because processings need to be able
to access the same metadata and data again.

## Policy format

