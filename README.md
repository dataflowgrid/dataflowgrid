# DataFlowGrid - Rethinking data management

The current way for handling data (especially big data) is fundamentally on the wrong track
because it puts spotlight on the provider side. Once the focus
switches to the consumer side many opportunities for better
data handling appear.

This project is planned as a reference implementation of the 2 patents:
- [Big data file management](https://register.dpma.de/DPMAregister/pat/register?AKZ=1020242021423)
- [distributed scheduling for data management](https://register.dpma.de/DPMAregister/pat/register?AKZ=1020242021563)

Several supplementary (non-patented) technology parts are developed as well.

## Main components

- JsonStream - stream processing for JSON structures

- [DOSS - Directory optimized streaming and storage](doss/README.md)

A library to efficiently (de)serialize data which is represented as a dictionary.

- pg-doss - a library to decode DOSS binaries directly in Postgresql

- elbow - client side loadbalancing overlay connection manager

- SeqServe - provides unique sequence numbers in predefined order

- [AgentQ](agentq/README.md) - a message server that distributes to several Process Server but only one per Agent at the same time

-  [Toor](toor/README.md) - a Token Orchestrator

- Enstate - a document database keeping track of changes per entity

- GridProxy - local proxy to optimize data access

- DataGate - local data transformer

- Connex - keeps track of links between data

- FlowGraph - find your way through the graph of data

- upload-controller

- uploader - A library which connects to *upload-controller* and uploads data into the grid.

- uploader-cli

- GridAuthX - an app for token exchange (like Azure EntraID token) to a token which can be used in dataFlowGrid

- GridGateway - a gateway to distribute backend connections to several backends over one connection

- ShoutOut - Transient data store to update other client with realtime information

# Parsers

- rosbag-flow