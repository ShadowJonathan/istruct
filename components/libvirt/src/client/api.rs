use std::collections::HashMap;

use async_trait::async_trait;

use istruct_common::{
    api::{
        compute::{
            devadm::v1::{DevAdmApi, DeviceType},
            machine::{
                device::v1::{CpuDevice, MachineDevApi, MemoryDevice},
                v1::{MachineAction, MachineApi, MachineState},
            },
        },
        network::device::v1::NetworkDevApi,
        storage::device::v1::{BlockDevice, StorageDevApi},
        ApiBase,
    },
    id::{DeviceId, MachineId},
};

use super::ClientPuck;

impl ApiBase for ClientPuck {}

#[async_trait]
impl MachineApi for ClientPuck {
    async fn act(&self, machine: MachineId, action: MachineAction) {
        self.with(|c| c.act_on(machine, action));
    }

    async fn status(&self, machine: MachineId) -> Option<MachineState> {
        self.with(|c| c.get_status(machine))
    }

    async fn dev_attach(&self, machine: MachineId, device: DeviceId) -> anyhow::Result<()> {
        self.with(|c| c.attach_device(machine, device))
    }

    async fn dev_detach(&self, machine: MachineId, device: DeviceId) -> anyhow::Result<()> {
        self.with(|c| c.detach_device(machine, device))
    }

    async fn dev_list(&self, machine: MachineId) -> Option<HashMap<DeviceId, String>> {
        self.with(|c| {
            let db = c.db();
            if !db.is_known_machine(&machine) {
                return None;
            }
            Some(
                db.get_dev_attached_to(machine)
                    .filter_map(|d| {
                        c.db()
                            .get_dev_type(d)
                            .map(super::DeviceType::to_string)
                            .map(|t| (d, t))
                    })
                    .collect(),
            )
        })
    }

    async fn create(&self) -> MachineId {
        self.with(|c| c.create())
    }

    async fn destroy(&self, machine: MachineId) -> anyhow::Result<()> {
        self.with(|c| c.destroy(machine))
    }

    async fn list(&self) -> Vec<MachineId> {
        self.with(|c| c.find_domains().collect())
    }
}

#[async_trait]
impl DevAdmApi for ClientPuck {
    async fn all(&self) -> HashMap<DeviceId, DeviceType> {
        self.with(|c| {
            c.db()
                .all_dev_types()
                .map(|(u, t)| (u, t.to_string()))
                .collect()
        })
    }

    async fn get_type(&self, dev: DeviceId) -> Option<DeviceType> {
        self.with(|c| c.db().get_dev_type(dev).map(|t| t.to_string()))
    }
}

#[async_trait]
impl MachineDevApi for ClientPuck {
    async fn get_memory(&self, device: DeviceId) -> Option<MemoryDevice> {
        self.with(|c| c.get_mem_bytes(device).map(|bytes| MemoryDevice { bytes }))
    }

    async fn set_memory(&self, device: DeviceId, memory: MemoryDevice) -> anyhow::Result<()> {
        self.with(|c| c.set_mem_bytes(device, memory.bytes))
    }

    async fn get_cpu(&self, device: DeviceId) -> Option<CpuDevice> {
        self.with(|c| c.get_cpu_cores(device).map(|cores| CpuDevice { cores }))
    }

    async fn set_cpu(&self, device: DeviceId, cpu: CpuDevice) -> anyhow::Result<()> {
        self.with(|c| c.set_cpu_cores(device, cpu.cores))
    }
}

#[async_trait]
impl NetworkDevApi for ClientPuck {
    async fn create_nat(&self) -> DeviceId {
        self.with(|c| c.create_nat())
    }

    async fn delete_nat(&self, device: DeviceId) -> anyhow::Result<()> {
        self.with(|c| c.delete_nat(device))
    }
}

#[async_trait]
impl StorageDevApi for ClientPuck {
    async fn get_block(&self, device: DeviceId) -> Option<BlockDevice> {
        self.with(|c| c.get_block_bytes(device).map(|bytes| BlockDevice { bytes }))
    }

    async fn create_block(&self, block: BlockDevice) -> DeviceId {
        self.with(|c| c.create_block(block.bytes))
    }

    async fn delete_block(&self, device: DeviceId) -> anyhow::Result<()> {
        self.with(|c| c.delete_block(device))
    }
}
