# is.compute

# is.compute.machine

did = device_id
mid = machine_id

Compute Machine control and interfacing

POST    /act/:mid/:action
GET     /status/:mid

#GET     /attr/:mid/io/:attr
#PUT     /attr/:mid/io/:attr
#DELETE  /attr/:mid/io/:attr
#PATCH   /attr/:mid/io
#GET     /attr/:mid/ls

GET     /dev/:mid               -> {:did -> type}
PUT     /dev/:mid/plug/:did
DELETE  /dev/:mid/plug/:did

POST    /m                      -> :mid
DELETE  /m/:mid
GET     /m

GET     /temp/:mid/cd   ->"/path"
POST    /temp/:mid/cd   <-"/path"
DELETE  /temp/:mid/cd

# is.compute.machine.device

Devices specific to Machines (CPU & Memory)

CPU and memory devices are created per machine, can only be altered here.

GET     /mem/:did   ->{bytes: 256}
PATCH   /mem/:did   <-{bytes: 256}

GET     /cpu/:did   ->{cores: 1}
PATCH   /cpu/:did   <-{cores: 1}


# is.compute.devadm

todo: this shouldn't be compute-specific

General registry of all devices known locally to the compute node

GET     /all        ->[(:did, "type")]
GET     /type/:did  ->"type"

# is.network.device

POST    /nat        ->:did
DELETE  /nat/:did

# is.storage.device

GET     /block                      ->{bytes: 256}
POST    /block      <-{bytes: 256}  ->:did
DELETE  /block/:did