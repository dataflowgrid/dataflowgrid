# Varint encoding
Many numbers are rather small but can be big from time to time. Instead of always reserving the same number of bits we use Varint encoding. This is the same encoding that also Parquet uses and is described at https://en.wikipedia.org/wiki/LEB128 .
The advantages are:
- small numbers take less bits
- very big numbers are possible
