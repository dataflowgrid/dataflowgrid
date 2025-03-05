# Token format

*GridAuthX* issues several different tokens which connect a user/machine/job/... to an *accesspolicy*.

The returned token follows the JWT specification regarding
header, payload and signature.

*GridAuthX* offers an endpoint to query for public keys in the JWKS format.

## User token

A *usertoken* is supposed to identify a real user of the system. Input is usually an OAuth2 token from Azure EntraID or something similar.

```json
{
    "sub": <string, subscriber id, internal technical userid>,
    "user": <string, technical id, reference to external system>,
    "exp": <timestamp, usually valid for not too long>,
    "aud": <string, accesspolicy id>,
    "_dfg": <object, allowed systems with state>
}
```

The *usertoken* does not contain too much useful information for applications (like user's name) - which is intentional.

A user token can be refreshed with the old systemstate or with a new systemstate.

## Job token

Every defined job also needs to access data. It does so via an accesspolicy too. But there is one important difference between users and jobs: jobs will always access the same state of the system at any time. This is important to guarantee that the job returns the same results when reexecuted.

