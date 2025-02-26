# elbow - Client-side Loadbalancing Overlay Communication Library

> **NOTE:** The name *elbow* references the *lb* for loadbalancing and *Ov* from overlay.

Communication between client and server in a dynamic cloud environment can be challenging. One of the promised advantages of cloud computing is that servers can appear and disappear at any time. This is even more important when servers are started as spot-instances (on Azure also called *low-priority VMs*). Those can be stopped by the cloud platform at any time - while having a considerable price advantage over standard VMs.

Another case is migrating workload from one server to another for example if the load on one gets too high.

Cloud platforms offer Load Balancers out of the box. The disadvantage is that all the traffic must go through them and backend servers are selected round-robin or session-stickiness. Especially for large data transfers this is not cost efficient. Client-side loadbalancing will directly choose and connect the proper server.

Communication is done using Websockets. The latter having the advantage of being able to traverse HTTP proxies. Security is implemented using TLS certificates. Another advantage here is that secondary servers can be contacted via IP addresses. Budget TLS certificate providers (like *let's encrypt*) will not allow certificates for IP addresses but rather only registered DNS names. But this means those servers (potentially short-lived in nature) have to be registered and deregistered from DNS, just to get a proper certificate. 

*elbow* provides a communication channel between client and server that handles those and other cases without the client even noticing:
- reconnects in case of connection loss
- dynamic loadbalancing the client based on round-robin, function or request content
- handles PKI for certificates for servers

