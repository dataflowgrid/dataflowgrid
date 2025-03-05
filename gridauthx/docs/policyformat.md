# Policy format

Example:

```json
{ 
    "_type": "dataflowgrid/accesspolicy:1",
    "policycondition": { 
        "_type": "or:1",
        "$or": [
            {
                "_type": "usergroup:1",
                "group": "admin"
            },
            {
                "_type": "user:1",
                "user": ["alex"]
            }
        ]
    },
    "datacondition": {
        "_type": "and:1",
        "$and": [
            {
                "_type": "attribute:1",
                "name": "public",
                "values": [true]
            }
        ]
    }
}
```

An *accesspolicy* has several subfields, which in turn have nested conditions on them.

Tokens are always issued on the latest version of the data graph.

Tokens for an older versions can be refreshed with a valid user token.

Tokens can be exchanged from user1 to user2 with valid user token of user1.

## policycondition

This condition tree defines who may get an accesstoken for this *accesspolicy*.

