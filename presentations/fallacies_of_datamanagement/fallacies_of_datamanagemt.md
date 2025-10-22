This document lists the fallacies that many current data management solution have. It is written to be similar to [Fallacies of distributed computing](https://en.wikipedia.org/wiki/Fallacies_of_distributed_computing).

# Leaky abstraction to users

Data processing combines data with compute to build results which in turn are inputs for the next data processings.

The less users need to know and understand about how and where the data is located and where computing happens
the less they have to keep in mind and consider when thinking about their business problems.

This considerably reduces the cognitive load on the developers while letting the data management system _breath_ to
optimize the workload.

# users are not aware of costs

Data storage costs money. Data movement costs time and money. Computations cost time and money (and carbon emissions).

Developers are often not aware of the costs their computing jobs create. In many the _cannot_ even get an answer - let
alone have an estimation before the processing is started.l

# One man's trash is another man's treasure

Imagine you are a drone developer and test a new sensor on it. Now during tests one of the motors stops responding.
Any recordings of those tests would be trash for your usecase. _Right?_

But wait: the team which is responsible for the motor development was desperately waiting for this case to happen
because customers were reporting it - but it could never be reproduced in the lab. Now finally they have a proper
debug reporting of this situation!

# Data retention depends on where data came from

# not managing reproducable processings

# data transport doesn't cost

# getting most while paying least

This is a sentence you hear quite often in everyday life. However, one of the first things you learn in business classes
is that this statement does not make sense: if you can spend $1 less but wait twice the time - would you do it? If you can spend $1M to make it a minute faster - would you go for it?

Mathematically you can only optimize for __one__ thing - either cost or time. But you know the saying _"time is money"_. We can put a price tag on an hour of our time, and then optimize for costs.

# priorities are simple

# not considering eh-da-costs


