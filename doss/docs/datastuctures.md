# DOSS Datastructures

## OrderedMultiDict
When DOSS is not working an streaming mode it treats objects as Dictionaries having keys and assigned values.
But these dicts have some special properties:
- keys are ordered, similar to the order they are written down in JSON
- keys can be duplicated, so a key can appear again later in the order

Keeping the order is important for efficient access. If the reader can figure out that it is not interested in the rest of the object be looking at e.g. the key *type* it is best if this key is serialized at the beginning.
Being able to serialize the same key several times is important e.g. for sending state to the deserializing app.

When a message is deserialized into a OrderedMultiDict the deserializer needs to know what to do with duplicate keys. Options are:
- just insert the new key at the end of the list
- overwrite the old value but keep the key where it is
- delete the old key and insert the key at the end
- merge the values into an array at under the old key

