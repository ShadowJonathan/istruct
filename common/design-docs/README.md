# IStruct Design

From a high-level perspective, IStruct would look like a bunch of "lego bricks", of which you can assemble a coherent compute cluster with different components, which'd then talk to eachother over a generic protocol.

## Layering

IStruct is made up of 3 "layers";
- Component
- Cluster
- Zone

The "Component" is the smallest part, the smallest "brick", it is a piece of software that - at the very least;
- Exposes an API endpoint
  - Exposes one or more API methods or domains
- Can be assigned a component ID by the cluster manager
- Optionally also has endpoints or interconnect points where protocol-specific traffic can flow to/from

The next layer is the "cluster", this refers to an environment where components can freely share resources.

The last layer is a "zone", where the above doesn't apply, and components sharing resources is not a given.

All three layers can be addressed, but a cluster manager, and a zone manager, has to identify each sub-layer item uniquely.