1. build internal API module which implements corresponding `ApiInterface`
2. convert into `&dyn` and pass into `api.rs` constructor
3. pass resulting `Router`-esc (with api prefix and all) into composite router
4. create server/service from it

---

is.compute.machine

POST    /act/:id/:action
    Imperactive machine action (boot, suspend, resume, force stop, etc.)
GET     /status/:id
    Current machine status

Attr objects are json-aligned?
GET     /attr/:id/:attr
PUT     /attr/:id/:attr (raw body/JSON?)
PATCH   /attr/:id (JSON)
GET     /lsattr/:id\

- device attach here
  - reason: splitting competencies, machines cant go without devices, but devices can go without machines, also big api to create should be able to also add devices immediately, so splitting here is good

devices (e.g. motherboard components)

is.compute.machine.dev

POST    /attach/:machine_id/:dev_id
POST    /detatch/:machine_id/:dev_id

memory/cpu as device? hot-plug?

abstracting it in device could make device-specific things (cpu count, mem count, graphics device) more specific

device type, mixins, mixin order is important (to apply final device profile)

attribute: pure/impure (no/yes sideeffect))

is.network.dev
is.storage.dev
(no .dev is subsystem, .dev is device creation, attributes and underlying storage)