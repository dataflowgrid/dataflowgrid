# AgentQ

## Background

Imagine how files weere used in offices before computers came into the picture. All files were stored in big cabinets and when somebody wanted to work on a file the clerk took it out of the cabinet and went to his desk.
As long as the file was in his posession nobody else should work on that file.

At the same time one clerk could work on several files at the same time but would be overworked if too many files are on his desk. Instead the boss would assign other workers to those files. Of course it is not necessary that the same cleark always works on the same files if others are as competent as him.

# Overview

In AgentQ an *Agent* has an ID and is basically the same as a file in the background story. The clerks are represented by *Processors*. Work tasks are represented as *Messages*.

AgentQ acts like a department head. It forwards *Messages* to proper *Processors* and waits until it was processed.

## Advanced use cases

### Conditions
Not everything always works as planned. Sometimes the *Processor* is told *interrupt* his work and hand back the file. Such changes in the environment are called *Conditions*. The *Processor* is informed that a *Condition* changed and should handle this accordingly.

### Priorities
*Messages* arrive with different priorities and should be handled accordingly - of course only if a proper *Processor* is available. If not it is still better to continue lower priority messages instead of doing nothing. Priorities effect the *Messages* for one *Agent* but also between *Agents*.

### Cancellation

The *Agent* can see all *Messages* currently scheduled for it. Consequently it can also cancel all of them. Cancellation is also possible from the outside though it might happen that the processing already started.

### Channels
*Messages* arrive at an *Agent* on a specific *Channel*. Messages in *Channels* or processed in order. But only one *Message* on one *Channel* is selected as the current task using *Priorities*. 

### Dependencies
*Messages* may also have a dependency on *Messages* in other *Channels*.
