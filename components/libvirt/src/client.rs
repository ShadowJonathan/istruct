use std::{
    borrow::Borrow,
    marker::PhantomData,
    ops::Index,
    path::{Path, PathBuf},
    sync::Arc,
};

use diplomatic_bag::DiplomaticBag;

use istruct_common::{
    api::compute::machine::v1::{MachineAction, MachineState},
    id::{DeviceId, MachineId},
};
use persy::{ByteVec, IndexType, Persy};
use uuid::Uuid;
use virt::{
    connect::Connect,
    domain::{Domain, DomainState},
};

mod api;

#[derive(Debug, Clone)]
pub struct ClientPuck {
    pub inner: Arc<DiplomaticBag<Client>>,
}

impl ClientPuck {
    #[inline]
    fn with<R: Send>(&self, f: impl (FnOnce(&Client) -> R) + Send) -> R {
        (*self.inner).as_ref().map(|_, c| f(c)).into_inner()
    }

    pub fn create(f: impl FnOnce() -> Client + Send) -> Self {
        let bag = DiplomaticBag::new(move |_| f());

        ClientPuck {
            inner: Arc::new(bag),
        }
    }
}

pub struct Client {
    pub conn: Connect,
    pub persy: Persy,
    pub block_path_dir: PathBuf,
}

impl Client {
    pub fn new(
        uri: &str,
        persy: impl AsRef<Path>,
        block_dir: impl AsRef<Path>,
    ) -> anyhow::Result<Self> {
        let conn = virt::connect::Connect::open(uri)?;

        let block_path_dir = block_dir.as_ref().canonicalize()?;

        if !block_path_dir.is_dir() {
            anyhow::bail!("block_dir is not a directory");
        }

        let persy = Persy::open_or_create_with(persy, persy::Config::new(), |_| Ok(()))?;

        Self::create_indexes(&persy);

        Ok(Self {
            conn,
            persy,
            block_path_dir,
        })
    }

    fn create_indexes(persy: &Persy) {
        test_or_create_index::<u128, u128>(&persy, DEV_MACHINE_ATTACHED);
        test_or_create_index::<u128, ByteVec>(&persy, DEV_TYPE);

        test_or_create_index::<u128, u64>(&persy, DEV_CPU);
        test_or_create_index::<u128, u64>(&persy, DEV_MEM);

        test_or_create_index::<u128, u64>(&persy, DEV_BLOCK_CAPACITY);

        test_or_create_index::<u128, u8>(&persy, KNOWN_MACHINES);
    }

    pub fn get_domain(&self, machine: impl Borrow<Uuid>) -> Option<Domain> {
        Domain::lookup_by_uuid_string(&self.conn, &machine.borrow().to_hyphenated().to_string())
            .ok()
    }

    pub fn get_domain_xml(&self, machine: impl Borrow<Uuid>) -> Option<crate::xml::Domain> {
        let d = self.get_domain(machine)?;

        let xml = d.get_xml_desc(0).unwrap();

        Some(crate::xml::Domain::from_str(&xml).unwrap())
    }

    pub fn list(&self) -> Vec<Uuid> {
        // todo proper error management
        self.conn
            .list_all_domains(0)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|d| d.get_uuid_string().ok())
            .filter_map(|s| Uuid::parse_str(&s).ok())
            .collect()
    }

    pub fn get_status(&self, uuid: impl Borrow<MachineId>) -> Option<MachineState> {
        self.get_domain(uuid)
            .map(|d| {
                d.get_state()
                    .ok()
                    .map(|(s, _)| virt_domainstate_to_istruct(s))
            })
            .flatten()
    }

    pub fn get_name(&self, uuid: Uuid) -> Option<String> {
        self.get_domain(uuid).map(|d| d.get_name().ok()).flatten()
    }

    pub fn act_on(&self, uuid: Uuid, action: MachineAction) -> Option<()> {
        let domain = self.get_domain(uuid)?;

        // todo verify starting state before running these
        match action {
            MachineAction::ForceShutdown => domain
                .destroy_flags(virt::domain::VIR_DOMAIN_DESTROY_GRACEFUL)
                .unwrap(),
            MachineAction::ForceReset => domain.reset().unwrap(),
            MachineAction::Shutdown => domain.shutdown().unwrap(),
            MachineAction::Suspend => domain.suspend().unwrap(),
            MachineAction::Resume => domain.resume().unwrap(),
            MachineAction::Boot => domain.create().unwrap(),
        };

        Some(())
    }

    pub fn create(&self) -> Uuid {
        use crate::xml;

        let uuid = Uuid::new_v4();

        let cpu_cores = 1u64;
        let mem_bytes = 128000000; // 128 MB

        self.define_domain(xml::Domain {
            typ: "kvm".to_string(),
            id: None,
            name: uuid.to_string(),
            uuid: Some(uuid),
            genid: None,
            title: None,
            description: None,
            memory: xml::Memory {
                unit: xml::Unit::Bytes,
                dump_core: None,
                amount: mem_bytes as usize,
            },
            max_memory: None,
            current_memory: None,
            vcpu: xml::VCpu {
                placement: None,
                cpuset: None,
                amount: cpu_cores as usize,
            },
            os: xml::OperatingSystem {
                firmware: None,
                typ: xml::OSType {
                    arch: None,
                    machine: None,
                    hypervisor: "hvm".to_string(),
                },
            },
            features: None,
            devices: xml::Devices {
                emulator: "/usr/bin/qemu-system-x86_64".to_string(),
                disks: vec![xml::Disk {
                    r#type: xml::DiskType::File,
                    driver: None,
                    source: None,
                    target: Some(xml::DiskTarget {
                        dev: "sda".into(),
                        bus: Some("sata".into()),

                        tray: None,
                        removable: None,
                        rotation_rate: None,
                    }),
                    readonly: vec![xml::Empty::new()],
                    boot: Some(xml::DiskBoot {
                        order: 1,

                        loadparm: None,
                    }),

                    device: Some(xml::DiskDevice::CDROM),
                    shareable: vec![],
                    serial: None,
                    wwn: None,
                    vendor: None,
                    product: None,
                    address: None,
                }],
                controllers: vec![],
                interfaces: vec![],
                parallels: vec![],
                serials: vec![],
                consoles: vec![],
                channels: vec![],
                inputs: vec![],
                graphics: vec![xml::Graphics {
                    typ: Some("spice".into()),
                    auto_port: Some("yes".into()),
                    listeners: vec![xml::GraphicsListener {
                        typ: Some("address".into()),
                    }],

                    port: None,
                    tls_port: None,
                    image: None,
                    streaming: None,
                    gl: None,
                }],
            },
        })
        .unwrap();

        let db = self.db();

        let mem = Uuid::new_v4();

        db.set_dev_type(mem.clone(), DeviceType::Compute(ComputeDeviceType::Mem));
        db.set_dev_attached(mem.clone(), uuid.clone());
        db.set_dev_mem(mem, mem_bytes);

        let cpu = Uuid::new_v4();

        db.set_dev_type(cpu.clone(), DeviceType::Compute(ComputeDeviceType::Cpu));
        db.set_dev_attached(cpu.clone(), uuid.clone());
        db.set_dev_cpu(cpu, cpu_cores);

        db.set_known_machine(uuid.clone());

        uuid
    }

    pub fn define_domain(&self, domain: crate::xml::Domain) -> anyhow::Result<Uuid> {
        Ok(Uuid::parse_str(
            &Domain::define_xml_flags(
                &self.conn,
                &domain.to_string()?,
                virt::domain::VIR_DOMAIN_DEFINE_VALIDATE,
            )?
            .get_uuid_string()?,
        )?)
    }

    // todo: device destroy/detach flags
    pub fn destroy(&self, uuid: Uuid) -> anyhow::Result<()> {
        let status = self
            .get_status(&uuid)
            .ok_or(anyhow::anyhow!("machine does not exist"))?;

        if let MachineState::Off = status {
            // fallthrough
        } else {
            anyhow::bail!("machine is not off");
        }

        let mut mem: Option<DeviceId> = None;
        let mut cpu: Option<DeviceId> = None;

        for (d, t) in self
            .db()
            .get_dev_attached_to(&uuid)
            .filter_map(|d| self.db().get_dev_type(d).map(|t| (d, t)))
        {
            if let DeviceType::Compute(c) = t {
                match c {
                    ComputeDeviceType::Cpu => cpu.insert(d),
                    ComputeDeviceType::Mem => mem.insert(d),
                };
            } else {
                self.detach_device(uuid.clone(), d)
                    .expect("machine is off, removing stuff should be fine")
            }
        }

        let cpu = cpu.expect("cpu device always exists");
        let mem = mem.expect("memory device always exists");

        let db = self.db();

        db.del_dev_type(mem.clone());
        db.del_dev_attached(mem.clone());
        db.del_dev_mem(mem);

        db.del_dev_type(cpu.clone());
        db.del_dev_attached(cpu.clone());
        db.del_dev_cpu(cpu);

        self.undefine_domain(uuid.clone());

        db.del_known_machine(uuid.clone());

        Ok(())
    }

    pub fn undefine_domain(&self, uuid: Uuid) {
        if let Some(d) = self.get_domain(uuid) {
            d.undefine().unwrap();
        }
    }

    pub fn attach_device(&self, machine: MachineId, device: DeviceId) -> anyhow::Result<()> {
        let typ = self
            .db()
            .get_dev_type(&device)
            .ok_or(anyhow::anyhow!("could not find device"))?;

        if let DeviceType::Compute(_) = typ {
            anyhow::bail!("cannot reassign compute devices");
        }

        if let Some(attached_machine) = self.db().get_dev_attached(&device) {
            if attached_machine == machine {
                anyhow::bail!("device already attached to this machine")
            } else {
                anyhow::bail!("device already attached to other machine")
            }
        }

        use {NetworkDeviceType as N, StorageDeviceType as S};

        match typ {
            DeviceType::Storage(S::Block) => self.attach_block(machine, device),
            DeviceType::Network(N::Nat) => self.attach_nat(machine, device),

            DeviceType::Compute(_) => unreachable!(),
        }
    }

    pub fn detach_device(&self, machine: MachineId, device: DeviceId) -> anyhow::Result<()> {
        let typ = self
            .db()
            .get_dev_type(&device)
            .ok_or(anyhow::anyhow!("could not find device"))?;

        if let DeviceType::Compute(_) = typ {
            anyhow::bail!("cannot reassign compute devices");
        }

        let attached_machine = self.db().get_dev_attached(&device);

        if let Some(attached_machine) = attached_machine {
            if attached_machine != machine {
                anyhow::bail!("device is attached to different device");
            }
        } else {
            anyhow::bail!("device is not attached");
        }

        use {NetworkDeviceType as N, StorageDeviceType as S};

        match typ {
            DeviceType::Storage(S::Block) => self.detach_block(machine, device),
            DeviceType::Network(N::Nat) => self.detach_nat(machine, device),

            DeviceType::Compute(_) => unreachable!(),
        }
    }

    pub fn find_domains(&self) -> impl Iterator<Item = MachineId> + '_ {
        let domains = self.conn.list_all_domains(0).unwrap();

        let db = self.db();

        domains
            .into_iter()
            .map(|d| Uuid::parse_str(&d.get_uuid_string().unwrap()).unwrap())
            .filter(move |u| db.is_known_machine(u))
    }

    fn edit(
        &self,
        machine: impl Borrow<MachineId>,
        f: impl FnOnce(&mut crate::xml::Domain),
    ) -> anyhow::Result<()> {
        let mut dom = self
            .get_domain_xml(machine)
            .ok_or(anyhow::anyhow!("cannot fetch domain"))?;

        f(&mut dom);

        self.define_domain(dom)?;

        Ok(())
    }
}

// device functions specific to compute
impl Client {
    fn set_cpu_cores(&self, dev: DeviceId, cores: u64) -> anyhow::Result<()> {
        if cores == 0 {
            anyhow::bail!("cores have to be above 0");
        }

        self.db()
            .get_dev_type(&dev)
            .ok_or(anyhow::anyhow!("cannot find device"))
            .and_then(|t| {
                if let DeviceType::Compute(ComputeDeviceType::Cpu) = t {
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("device is not cpu"))
                }
            })?;

        let m = self
            .db()
            .get_dev_attached(dev.clone())
            .expect("device is cpu, it has to be attached");

        self.edit(m, |d| d.vcpu.amount = cores as usize)?;

        self.db().set_dev_cpu(dev, cores);

        Ok(())
    }

    fn get_cpu_cores(&self, dev: DeviceId) -> Option<u64> {
        self.db().get_dev_cpu(dev)
    }

    fn set_mem_bytes(&self, dev: DeviceId, bytes: u64) -> anyhow::Result<()> {
        if bytes == 0 {
            anyhow::bail!("memory bytes have to be above 0");
        }

        self.db()
            .get_dev_type(&dev)
            .ok_or(anyhow::anyhow!("cannot find device"))
            .and_then(|t| {
                if let DeviceType::Compute(ComputeDeviceType::Mem) = t {
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("device is not memory"))
                }
            })?;

        let m = self
            .db()
            .get_dev_attached(dev.clone())
            .expect("device is mem, it has to be attached");

        self.edit(m, |d| {
            d.memory.amount = bytes as usize;
            d.memory.unit = crate::xml::Unit::Bytes;

            d.current_memory = None;
        })?;

        self.db().set_dev_mem(dev, bytes);

        Ok(())
    }

    fn get_mem_bytes(&self, dev: DeviceId) -> Option<u64> {
        self.db().get_dev_mem(dev)
    }
}

// device functions specific to storage
impl Client {
    fn create_block(&self, bytes: u64) -> DeviceId {
        let uuid = Uuid::new_v4();

        self.create_block_file(&uuid, bytes).unwrap();

        let db = self.db();

        db.set_dev_type(uuid.clone(), DeviceType::Storage(StorageDeviceType::Block));
        db.set_dev_block_cap(uuid.clone(), bytes);

        uuid
    }

    fn delete_block(&self, device: DeviceId) -> anyhow::Result<()> {
        self.db()
            .get_dev_type(&device)
            .ok_or(anyhow::anyhow!("could not find device"))
            .and_then(|t| {
                if let DeviceType::Storage(StorageDeviceType::Block) = t {
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("device was not block storage"))
                }
            })?;

        if let Some(machine) = self.db().get_dev_attached(&device) {
            anyhow::bail!("device is attached to {}", machine);
        }

        self.delete_block_file(device)?;

        let db = self.db();

        db.del_dev_type(device.clone());
        db.del_dev_block_cap(device.clone());

        Ok(())
    }

    fn get_block_bytes(&self, device: DeviceId) -> Option<u64> {
        self.db().get_dev_block_cap(device)
    }

    fn attach_block(&self, machine: MachineId, device: DeviceId) -> anyhow::Result<()> {
        self.db()
            .get_dev_type(&device)
            .ok_or(anyhow::anyhow!("could not find device"))
            .and_then(|t| {
                if let DeviceType::Storage(StorageDeviceType::Block) = t {
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("device was not a storage block"))
                }
            })?;

        if let Some(m) = self.db().get_dev_attached(&device) {
            anyhow::bail!("device is attached to {}", m);
        }

        self.edit(&machine, |d| {
            use crate::xml::{Disk, DiskDriver, DiskTarget, DiskType, Source};

            d.devices.disks.push(Disk {
                r#type: DiskType::File,
                driver: Some(DiskDriver {
                    name: "qemu".into(),
                    r#type: Some("qcow2".into()),

                    cache: None,
                    error_policy: None,
                    read_error_policy: None,
                    io: None,
                    ioeventfd: None,
                    event_idx: None,
                    copy_on_read: None,
                    discard: None,
                    detect_zeroes: None,
                    iommu: None,
                    ats: None,
                    packed: None,
                    page_per_vq: None,
                }),
                source: Some(Source::file(
                    self.path_for_block_device(&device).to_str().unwrap(),
                )),
                target: Some(DiskTarget {
                    dev: calculate_next_vd(&d.devices.disks),
                    bus: Some("ide".into()),

                    tray: None,
                    removable: None,
                    rotation_rate: None,
                }),

                device: None,
                boot: None,
                readonly: vec![],
                shareable: vec![],
                serial: None,
                wwn: None,
                vendor: None,
                product: None,
                address: None,
            })
        })?;

        self.db().set_dev_attached(device, machine);

        Ok(())
    }

    fn detach_block(&self, machine: MachineId, device: DeviceId) -> anyhow::Result<()> {
        self.db()
            .get_dev_type(&device)
            .ok_or(anyhow::anyhow!("could not find device"))
            .and_then(|t| {
                if let DeviceType::Storage(StorageDeviceType::Block) = t {
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("device was not a storage block"))
                }
            })?;

        if let Some(m) = self.db().get_dev_attached(&device) {
            if m != machine {
                anyhow::bail!(
                    "device is not attached to {}, it is attached to {}",
                    machine,
                    m
                );
            }
        } else {
            anyhow::bail!("device is not attached");
        }

        self.edit(machine, |d| {
            d.devices.disks.retain(|disk| {
                if let Some(source) = &disk.source {
                    if let Some(file) = &source.file {
                        if file.contains(&device.to_string()) {
                            return false;
                        }
                    }
                }

                true
            })
        })?;

        self.db().del_dev_attached(device);

        Ok(())
    }
}

const NAT_NETWORK_XML: &str = r#"<network>
<name>istruct_nat</name>
<forward mode="nat"/>
<domain name="network"/>
<ip address="192.168.100.1" netmask="255.255.255.0">
  <dhcp>
    <range start="192.168.100.128" end="192.168.100.254"/>
  </dhcp>
</ip>
</network>"#;

// device functions specific to network
impl Client {
    fn create_nat(&self) -> DeviceId {
        let uuid = Uuid::new_v4();

        self.assure_nat_network();

        self.db()
            .set_dev_type(uuid.clone(), DeviceType::Network(NetworkDeviceType::Nat));

        uuid
    }

    fn assure_nat_network(&self) {
        use virt::network::Network;

        if Network::lookup_by_name(&self.conn, "istruct_nat").is_err() {
            Network::define_xml(&self.conn, NAT_NETWORK_XML).unwrap();
        }
    }

    fn delete_nat(&self, device: DeviceId) -> anyhow::Result<()> {
        self.db()
            .get_dev_type(&device)
            .ok_or(anyhow::anyhow!("could not find device"))
            .and_then(|t| {
                if let DeviceType::Network(NetworkDeviceType::Nat) = t {
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("device was not a nat interface"))
                }
            })?;

        if let Some(machine) = self.db().get_dev_attached(&device) {
            anyhow::bail!("device is attached to {}", machine);
        }

        // device is nat, is not attached

        self.db().del_dev_type(device);

        Ok(())
    }

    fn attach_nat(&self, machine: MachineId, device: DeviceId) -> anyhow::Result<()> {
        self.db()
            .get_dev_type(&device)
            .ok_or(anyhow::anyhow!("could not find device"))
            .and_then(|t| {
                if let DeviceType::Network(NetworkDeviceType::Nat) = t {
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("device was not a nat network"))
                }
            })?;

        if let Some(m) = self.db().get_dev_attached(&device) {
            anyhow::bail!("device is attached to {}", m);
        }

        self.edit(&machine, |d| {
            use crate::xml::{NetworkInterface, NetworkSource};

            d.devices.interfaces.push(NetworkInterface {
                typ: "network".to_string(),
                source: Some(NetworkSource {
                    network: Some("istruct_nat".into()),

                    bridge: None,
                }),

                mac: None,
                model: None,
                address: None,
            })
        })?;

        self.db().set_dev_attached(device, machine);

        Ok(())
    }

    fn detach_nat(&self, machine: MachineId, device: DeviceId) -> anyhow::Result<()> {
        self.db()
            .get_dev_type(&device)
            .ok_or(anyhow::anyhow!("could not find device"))
            .and_then(|t| {
                if let DeviceType::Network(NetworkDeviceType::Nat) = t {
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("device was not a nat network"))
                }
            })?;

        if let Some(m) = self.db().get_dev_attached(&device) {
            if m != machine {
                anyhow::bail!(
                    "device is not attached to {}, it is attached to {}",
                    machine,
                    m
                );
            }
        } else {
            anyhow::bail!("device is not attached");
        }

        self.edit(machine, |d| {
            d.devices.interfaces.retain(|i| {
                if let Some(source) = &i.source {
                    if let Some(network) = &source.network {
                        return network.as_str() != "istruct_nat";
                    }
                }

                true
            })
        })?;

        self.db().del_dev_attached(device);

        Ok(())
    }
}

// block device file functions
impl Client {
    fn path_for_block_device(&self, dev: impl Borrow<DeviceId>) -> std::path::PathBuf {
        self.block_path_dir
            .join(format!("block_{}.qcow2", dev.borrow()))
    }

    fn create_block_file(&self, dev: impl Borrow<DeviceId>, bytes: u64) -> anyhow::Result<()> {
        use std::process::Command;

        Command::new("qemu-img")
            .arg("create")
            .arg("-f")
            .arg("qcow2")
            .arg(self.path_for_block_device(dev))
            .arg(bytes.to_string())
            .output()?;

        Ok(())
    }

    fn delete_block_file(&self, dev: impl Borrow<DeviceId>) -> anyhow::Result<()> {
        std::fs::remove_file(self.path_for_block_device(dev)).map_err(Into::into)
    }
}

// todo: temp CD methods
// fixme: detach_block can remove a cd drive if it has the same UUID
// impl Client {
//     pub fn get_cd(&self, machine: MachineId) -> Option<String> {
//         let d = self.get_domain_xml(machine)?;

//         for disk in &d.devices.disks {
//             if let Some(crate::xml::DiskDevice::CDROM) = disk.device {
//                 return disk.source.as_ref().and_then(|s| s.file.to_owned());
//             }
//         }

//         None
//     }

//     pub fn set_cd(&self, machine: MachineId, path: String) -> anyhow::Result<()> {
//         self.edit(machine, |d| {
//             for disk in &mut d.devices.disks {
//                 if let Some(crate::xml::DiskDevice::CDROM) = disk.device {
//                     disk.source = Some(crate::xml::Source::file(path));

//                     break;
//                 }
//             }
//         })
//     }

//     pub fn rm_cd(&self, machine: MachineId) -> anyhow::Result<()> {
//         self.edit(machine, |d| {
//             for disk in &mut d.devices.disks {
//                 if let Some(crate::xml::DiskDevice::CDROM) = disk.device {
//                     disk.source = None;

//                     return;
//                 }
//             }
//         })
//     }
// }

const KNOWN_MACHINES: &str = "known_machines";
const DEV_MACHINE_ATTACHED: &str = "dev_machine_attached";
const DEV_TYPE: &str = "dev_type";
const DEV_CPU: &str = "dev_cpu";
const DEV_MEM: &str = "dev_mem";
const DEV_BLOCK_CAPACITY: &str = "dev_block_capacity";

#[derive(Debug)]
enum DeviceType {
    Compute(ComputeDeviceType),
    Storage(StorageDeviceType),
    Network(NetworkDeviceType),
}

#[derive(Debug)]
enum ComputeDeviceType {
    Cpu,
    Mem,
}

#[derive(Debug)]
enum StorageDeviceType {
    Block,
}

#[derive(Debug)]
enum NetworkDeviceType {
    Nat,
}

impl DeviceType {
    fn parse_str(s: impl Borrow<str>) -> Option<Self> {
        use {ComputeDeviceType as C, NetworkDeviceType as N, StorageDeviceType as S};

        Some(match s.borrow() {
            "is.compute.cpu" => Self::Compute(C::Cpu),
            "is.compute.mem" => Self::Compute(C::Mem),
            "is.storage.block" => Self::Storage(S::Block),
            "is.network.nat" => Self::Network(N::Nat),
            _ => return None,
        })
    }

    fn to_string(self) -> String {
        use {ComputeDeviceType as C, NetworkDeviceType as N, StorageDeviceType as S};

        match self {
            DeviceType::Compute(C::Cpu) => "is.compute.cpu",
            DeviceType::Compute(C::Mem) => "is.compute.mem",
            DeviceType::Storage(S::Block) => "is.storage.block",
            DeviceType::Network(N::Nat) => "is.network.nat",
        }
        .to_string()
    }
}

impl Client {
    fn db(&self) -> ClientDB<'_> {
        return ClientDB(self);
    }
}

struct PersyInterface<'a, K, V>
where
    K: IndexType,
    V: IndexType,
{
    persy: &'a Persy,
    name: &'static str,

    key: PhantomData<K>,
    value: PhantomData<V>,
}

impl<K, V> PersyInterface<'_, K, V>
where
    K: IndexType,
    V: IndexType,
{
    fn get<Q>(&self, key: Q) -> Option<V>
    where
        Q: Borrow<K>,
    {
        self.persy
            .get::<K, V>(self.name, key.borrow())
            .unwrap()
            .next()
    }

    fn range<R>(&self, range: R) -> impl Iterator<Item = (K, V)>
    where
        R: std::ops::RangeBounds<K>,
    {
        self.persy
            .range::<K, V, _>(&self.name, range)
            .unwrap()
            .filter_map(move |(k, mut v)| v.next().map(|v| (k, v)))
    }

    fn set(&self, key: K, val: V) {
        let mut tx = self.begin();
        tx.put::<K, V>(&self.name, key, val).unwrap();
        tx.prepare().unwrap().commit().unwrap();
    }

    fn begin(&self) -> persy::Transaction {
        self.persy.begin().unwrap()
    }

    fn del(&self, key: K) {
        let mut tx = self.begin();
        tx.remove::<K, V>(&self.name, key, None).unwrap();
        tx.prepare().unwrap().commit().unwrap();
    }
}

pub struct ClientDB<'c>(&'c Client);

// Persy Interface
impl ClientDB<'_> {
    fn interface<'s, K, V>(&'s self, name: &'static str) -> PersyInterface<'s, K, V>
    where
        K: IndexType,
        V: IndexType,
    {
        PersyInterface {
            persy: &self.0.persy,
            name,
            key: PhantomData,
            value: PhantomData,
        }
    }
}

// Device Type
impl ClientDB<'_> {
    fn dev_type(&self) -> PersyInterface<'_, u128, ByteVec> {
        self.interface(DEV_TYPE)
    }

    fn all_dev_types(&self) -> impl Iterator<Item = (DeviceId, DeviceType)> {
        self.dev_type().range(..).map(|(k, v)| {
            (
                Uuid::from_u128(k),
                DeviceType::parse_str(
                    String::from_utf8(v.into()).expect("database has value string"),
                )
                .expect("database has valid device type"),
            )
        })
    }

    fn get_dev_type(&self, dev: impl Borrow<DeviceId>) -> Option<DeviceType> {
        self.dev_type()
            .get(dev.borrow().as_u128())
            .map(|v| String::from_utf8(v.into()).unwrap())
            .map(|s| DeviceType::parse_str(s).expect("database has valid device type"))
    }

    fn set_dev_type(&self, dev: DeviceId, typ: DeviceType) {
        self.dev_type()
            .set(dev.as_u128(), typ.to_string().into_bytes().into())
    }

    fn del_dev_type(&self, dev: DeviceId) {
        self.dev_type().del(dev.as_u128())
    }
}

// Device Attach
impl ClientDB<'_> {
    fn dev_attach(&self) -> PersyInterface<'_, u128, u128> {
        self.interface(DEV_MACHINE_ATTACHED)
    }

    fn get_dev_attached(&self, dev: impl Borrow<DeviceId>) -> Option<MachineId> {
        self.dev_attach()
            .get(dev.borrow().as_u128())
            .map(|v| Uuid::from_u128(v))
    }

    fn get_dev_attached_to(
        &self,
        machine: impl Borrow<MachineId>,
    ) -> impl Iterator<Item = DeviceId> {
        let m = machine.borrow().as_u128();

        self.dev_attach().range(..).filter_map(move |(k, v)| {
            if v == m {
                Some(Uuid::from_u128(k))
            } else {
                None
            }
        })
    }

    fn set_dev_attached(&self, dev: DeviceId, machine: MachineId) {
        self.dev_attach().set(dev.as_u128(), machine.as_u128())
    }

    fn del_dev_attached(&self, dev: DeviceId) {
        self.dev_attach().del(dev.as_u128())
    }
}

// CPU Device
impl ClientDB<'_> {
    fn dev_cpu(&self) -> PersyInterface<'_, u128, u64> {
        self.interface(DEV_CPU)
    }

    fn get_dev_cpu(&self, dev: impl Borrow<DeviceId>) -> Option<u64> {
        self.dev_cpu().get(dev.borrow().as_u128())
    }

    fn set_dev_cpu(&self, dev: DeviceId, cores: u64) {
        self.dev_cpu().set(dev.as_u128(), cores)
    }

    fn del_dev_cpu(&self, dev: DeviceId) {
        self.dev_cpu().del(dev.as_u128())
    }
}

// Memory device
impl ClientDB<'_> {
    fn dev_mem(&self) -> PersyInterface<'_, u128, u64> {
        self.interface(DEV_MEM)
    }

    fn get_dev_mem(&self, dev: impl Borrow<DeviceId>) -> Option<u64> {
        self.dev_mem().get(dev.borrow().as_u128())
    }

    fn set_dev_mem(&self, dev: DeviceId, bytes: u64) {
        self.dev_mem().set(dev.as_u128(), bytes)
    }

    fn del_dev_mem(&self, dev: DeviceId) {
        self.dev_mem().del(dev.as_u128())
    }
}

// Storage device
impl ClientDB<'_> {
    fn dev_block_cap(&self) -> PersyInterface<'_, u128, u64> {
        self.interface(DEV_BLOCK_CAPACITY)
    }

    fn get_dev_block_cap(&self, dev: impl Borrow<DeviceId>) -> Option<u64> {
        self.dev_block_cap().get(dev.borrow().as_u128())
    }

    fn set_dev_block_cap(&self, dev: DeviceId, cores: u64) {
        self.dev_block_cap().set(dev.as_u128(), cores)
    }

    fn del_dev_block_cap(&self, dev: DeviceId) {
        self.dev_block_cap().del(dev.as_u128())
    }
}

// Storage device
impl ClientDB<'_> {
    fn known_machines(&self) -> PersyInterface<'_, u128, u8> {
        self.interface(KNOWN_MACHINES)
    }

    fn is_known_machine(&self, machine: impl Borrow<MachineId>) -> bool {
        self.known_machines()
            .get(machine.borrow().as_u128())
            .is_some()
    }

    fn set_known_machine(&self, machine: MachineId) {
        self.known_machines().set(machine.as_u128(), 0)
    }

    fn del_known_machine(&self, machine: MachineId) {
        self.known_machines().del(machine.as_u128())
    }
}

fn virt_domainstate_to_istruct(state: DomainState) -> MachineState {
    use virt::domain::{
        VIR_DOMAIN_BLOCKED, VIR_DOMAIN_CRASHED, VIR_DOMAIN_NOSTATE, VIR_DOMAIN_PAUSED,
        VIR_DOMAIN_PMSUSPENDED, VIR_DOMAIN_RUNNING, VIR_DOMAIN_SHUTDOWN, VIR_DOMAIN_SHUTOFF,
    };
    match state {
        VIR_DOMAIN_NOSTATE => MachineState::Off,
        VIR_DOMAIN_RUNNING => MachineState::Running,
        // todo technically not correct
        VIR_DOMAIN_BLOCKED => MachineState::Error,
        VIR_DOMAIN_PAUSED => MachineState::Suspended,
        // todo technically not correct, add "transitioning" state?
        VIR_DOMAIN_SHUTDOWN => MachineState::Running,
        VIR_DOMAIN_SHUTOFF => MachineState::Off,
        VIR_DOMAIN_CRASHED => MachineState::Error,
        // todo watch this state when resuming
        VIR_DOMAIN_PMSUSPENDED => MachineState::Suspended,
        unknown => panic!("Unknown DomainState {}", unknown),
    }
}

fn test_or_create_index<K, V>(persy: &Persy, name: &str)
where
    K: IndexType,
    V: IndexType,
{
    if !persy.exists_index(name).unwrap() {
        let mut tx = persy.begin().unwrap();
        tx.create_index::<K, V>(name, persy::ValueMode::Replace)
            .unwrap();
        tx.prepare().unwrap().commit().unwrap();
    }
}

static ASCII_LOWER: [char; 26] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z',
];

fn calculate_next_vd(disks: &[crate::xml::Disk]) -> String {
    let mut seen = vec![];

    for disk in disks {
        if let Some(t) = &disk.target {
            if t.dev.starts_with("vd") {
                seen.push(t.dev[2..].to_string())
            }
        }
    }

    seen.sort_by_key(|s| (s.len(), s.clone()));

    let mut current = "a".to_string();

    for one in seen {
        if current == one {
            let last: char = current.chars().last().unwrap();
            let replace_with: String = if last == 'z' {
                "aa".into()
            } else {
                let i = ASCII_LOWER.iter().position(|&c| c == last).unwrap();

                ASCII_LOWER.index(i + 1).to_string()
            };

            let len = current.len();
            current.replace_range(len - 1..len, &replace_with)
        }
    }

    format!("vd{}", current)
}
