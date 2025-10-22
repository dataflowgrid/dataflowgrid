Toor - Token Orchestrator
-------------------------

_Toor_ is a workflow engine which is based on the idea of Petri token. Those are used in [Petri nets](https://en.wikipedia.org/wiki/Petri_net) but _Toor_ takes a somewhat different approach.

Almost all workflow engines model workflows as DAGs which stands for Directed Acyclic Graph. While the idea to model a workflow as a graph consisting of nodes (aka vertices) and directed edges is very natural - it is not so natural to expect that every execution is acyclic. Why would we _not_ allow the workflow to have loops?
Another problem is the synchronization between different workflow executions.

Main idea
=========

A workflow consists of nodes and directed edges between them. The nodes are where some processing happens in some external system.

Workflow Executions are modeled by creating one (or more) _Tokens_ and placing them inside the workflow.

_Toor_ will manage the transition of the tokens from output connector to a predefinied input connector of some other node.

Elements of Toor
================

Every node has input and output connectors. All connectors are named in a path like syntax using / as a delimeter. The names can contain wildcards * (some string) and ** (any subpath).

The workflow defines the ordered connections between output connectors and input connectors. Every workflow can contain additional parameters in a generic object.

Execution flow is controlled by tokens. Those have a name and carry parameters as a generic object.

States
======
Several elements of the workflow can be in certain states:
- the Toor engine as a whole
- the workflow as a whole
- the components
- nodes in workflow
- input connectors
- output connectors
- tokens

ALl of those elements can be in _active_ or _stopped_ state. In _stopped_ state no automatic transitioning is performed and no callback to components are made.

Note that changes to the workflow and tokens are possible at any time. This would allow you to stop the workflow, do some migration and let it run again.


Execution
=========

Nodes of the workflow are mapped to components. Nodes in different workflows or different nodes of one workflow can map to the same component.

Whenever a new token arrives at either an input connector or at the node itself, the component is informed via a callback.

Any system (usually the component executor) can make changes to the workflow at any time. This is mostly used to move a token from an input connector to the node - and to move the token from the node to some output connector. However, changes are not limited in that way. Basically it is possible to move a token to any position in the workflow; to let a token disappear - or even to change the workflow definition. If you really want to you can even move the token to another workflow.

After every change the Toor engine will move any token in output connectors to their respective input connectors of another component if:
- the Toor engine is active
- the workflow is active
- the token is active
- the output connector is active
- at least one connection an that output connector is active.

Groupings
=========

To make management of tokens easier those can be assigned to one or more groups. Also groups can contain other groups.

In more complex scenarios with branching several tokens together form a workflow execution and this way tools
can more easily get an idea about the current progress.

Node nestings
=============

Nodes can have parent nodes. In case the a token is placed at an output connector for which no name is defined, the Toor engine will try to find a matching input connector on the parent node. If it finds one it will place the token there. As input connectors can use the wildcard name, this can be used as a catch-all for error cases.

A workflow can have an explicit outer node as a catch-all for all other nodes. If no other component handles the token it is implicitly transfered into a dead-token-queue of the workflow.


Common workflow modellings
==========================

### Branching
A component may consume one token and generate 2 or more tokens as its output. Those tokens then run independently.
There should be another component which then gathers those tokens and combines the results back into a single token.

### Fan-out

This case is similar to branching. But now the workflow does __not__ contain several different paths. Instead only one path is modelled but the fanout component would generate different tokens with different parameters.

### Semaphore
Imagine you want to protect a critical section by only letting a certain number of parallel tokens be executed within that section. A component could keep track of active tokens, park token which want to enter - and upon
leaving the section let another token enter.

