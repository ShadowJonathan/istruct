# `IStruct`

> Derivative of "infrastructure", only when stylized as `IStruct`, it conforms to a Java interface naming scheme; An interface to "Structure"

An architecture design aiming to provide a coherent set of APIs and definitions for Compute infrastructure,
so that different "components" can work together with common definitions.

*Note: This library is a WIP, design documents and such as scattered about at the moment.*

## Repo layout

- The core API definitions, structures, and enums, are laid out under `common`
- "In-tree" components are under `components`
  - The only available component at the moment is a Libvirt one.