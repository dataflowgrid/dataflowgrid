# Directory Optimized Streaming and Storage - DOSS

## Introduction 
Processing data can be done in two different ways: dynamic schema and static schema. With a static schema the schema is known at compile time and translated into static data structures.
With dynamic processing the schema is not known, sometimes does not exist. In this case the data is commonly represented as nested dictionaries.

The most known format to save this nested dictionaries to a file is JSON. But JSON is rather *chatty* meaning it includes a lot of characters that include no information (like whitespaces) and duplicates characters (like field names in every instance of an array). Basically the same is true for YAML.

DOSS is optimized for storing such dictionaries by keeping a dictionary of elements (strings, numbers, objects) in memory and reusing references to those entries while deserializing the data stream.

DOSS allows streaming of the data. This means it can be consumed even before the whole file was completely generated. On the other hand it can be optimized to allow skipping of big chunks of the file when those chunks are not needed.


# Getting Started
TODO: Guide users through getting your code up and running on their own system. In this section you can talk about:
1.	Installation process
2.	Software dependencies
3.	Latest releases
4.	API references

# Build and Test
TODO: Describe and show how to build your code and run the tests. 

# Contribute
TODO: Explain how other users and developers can contribute to make your code better. 

If you want to learn more about creating good readme files then refer the following [guidelines](https://docs.microsoft.com/en-us/azure/devops/repos/git/create-a-readme?view=azure-devops). You can also seek inspiration from the below readme files:
- [ASP.NET Core](https://github.com/aspnet/Home)
- [Visual Studio Code](https://github.com/Microsoft/vscode)
- [Chakra Core](https://github.com/Microsoft/ChakraCore)