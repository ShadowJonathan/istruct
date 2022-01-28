use super::{OnOff, YesNo};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "domain")]
pub struct Domain {
    #[serde(rename = "$attr:type")]
    pub typ: String,
    #[serde(rename = "$attr:id", skip_serializing_if = "Option::is_none")]
    pub id: Option<isize>,

    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genid: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    // metadata: (),
    pub memory: Memory,

    #[serde(rename = "maxMemory", skip_serializing_if = "Option::is_none")]
    pub max_memory: Option<MaxMemory>,

    #[serde(rename = "currentMemory", skip_serializing_if = "Option::is_none")]
    pub current_memory: Option<CurrentMemory>,

    pub vcpu: VCpu,

    pub os: OperatingSystem,

    pub features: Option<Features>,

    pub devices: Devices,
}

#[derive(Debug, Serialize, Deserialize)]
struct DomainDoc {
    domain: Domain,
}

impl Domain {
    pub fn to_string(self) -> Result<String, xml_serde::Error> {
        xml_serde::to_string(&DomainDoc { domain: self })
    }

    pub fn from_str(s: &str) -> Result<Self, xml_serde::Error> {
        xml_serde::from_str::<DomainDoc>(s).map(|dd| dd.domain)
    }
}

// TODO: maybe change to enum between bios, kernel and container
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "os")]
pub struct OperatingSystem {
    #[serde(rename = "$attr:firmware", skip_serializing_if = "Option::is_none")]
    pub firmware: Option<Firmware>,

    #[serde(rename = "type")]
    pub typ: OSType,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Firmware {
    BIOS,
    EFI,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OSType {
    #[serde(rename = "$attr:arch", skip_serializing_if = "Option::is_none")]
    pub arch: Option<String>,

    #[serde(rename = "$attr:machine", skip_serializing_if = "Option::is_none")]
    pub machine: Option<String>,

    #[serde(rename = "$value")]
    pub hypervisor: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Features {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub pae: Vec<Empty>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub acpi: Vec<Empty>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub apic: Vec<Empty>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub hap: Vec<Empty>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub viridian: Vec<Empty>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub privnet: Vec<Empty>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub hyperv: Option<FeaturesHyperV>,

    // todo: pvspinlock
    // todo: kvm
    // todo: xen
    // todo: pmu
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vmport: Option<WithOnOff>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeaturesHyperV {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relaxed: Option<WithOnOff>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vapic: Option<WithOnOff>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spinlocks: Option<hyperv::SpinLocks>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vpindex: Option<WithOnOff>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime: Option<WithOnOff>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub synic: Option<WithOnOff>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stimer: Option<hyperv::STimer>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reset: Option<WithOnOff>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor_id: Option<hyperv::VendorId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequencies: Option<WithOnOff>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reenlightenment: Option<WithOnOff>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tlbflush: Option<WithOnOff>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipi: Option<WithOnOff>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evmcs: Option<WithOnOff>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WithOnOff {
    #[serde(rename = "$attr:state")]
    pub status: OnOff,
}

pub mod hyperv {
    use super::*;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct SpinLocks {
        #[serde(rename = "$attr:state")]
        pub status: OnOff,

        #[serde(rename = "$attr:retries")]
        pub retries: usize,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct STimer {
        #[serde(rename = "$attr:state")]
        pub status: OnOff,

        #[serde(rename = "$attr:direct")]
        pub direct: OnOff,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct VendorId {
        #[serde(rename = "$attr:state")]
        pub status: OnOff,

        #[serde(rename = "$attr:value")]
        pub value: String,
    }
}

// TODO: Add "vcpus"
#[derive(Debug, Serialize, Deserialize)]
pub struct VCpu {
    #[serde(rename = "$attr:placement", skip_serializing_if = "Option::is_none")]
    pub placement: Option<String>,

    #[serde(rename = "$attr:cpuset", skip_serializing_if = "Option::is_none")]
    pub cpuset: Option<String>,

    #[serde(rename = "$value")]
    pub amount: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Memory {
    #[serde(rename = "$attr:unit")]
    pub unit: Unit,

    #[serde(rename = "$attr:dumpCore", skip_serializing_if = "Option::is_none")]
    pub dump_core: Option<OnOff>,

    #[serde(rename = "$value")]
    pub amount: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MaxMemory {
    #[serde(rename = "$attr:unit")]
    pub unit: Unit,

    #[serde(rename = "$attr:slots", skip_serializing_if = "Option::is_none")]
    pub slots: Option<usize>,

    #[serde(rename = "$value")]
    pub amount: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CurrentMemory {
    #[serde(rename = "$attr:unit")]
    pub unit: Unit,

    #[serde(rename = "$value")]
    pub amount: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Unit {
    #[serde(rename = "bytes")]
    Bytes,
    #[serde(rename = "b")]
    B, // bytes

    KB,
    KiB,
    #[serde(rename = "k")]
    K, // KiB

    MB,
    MiB,
    M, // MiB

    GB,
    GiB,
    G, // GiB

    TB,
    TiB,
    T, // GiB
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Devices {
    pub emulator: String,

    #[serde(rename = "disk", default, skip_serializing_if = "Vec::is_empty")]
    pub disks: Vec<Disk>,

    // todo: filesystem
    #[serde(rename = "controller", default, skip_serializing_if = "Vec::is_empty")]
    pub controllers: Vec<Controller>,

    // todo: lease
    // todo: hostdev
    // todo: redirdev
    // todo: smartcard
    #[serde(rename = "interface", default, skip_serializing_if = "Vec::is_empty")]
    pub interfaces: Vec<NetworkInterface>,

    #[serde(rename = "parallel", default, skip_serializing_if = "Vec::is_empty")]
    pub parallels: Vec<FromToDevice>,

    #[serde(rename = "serial", default, skip_serializing_if = "Vec::is_empty")]
    pub serials: Vec<FromToDevice>,

    #[serde(rename = "console", default, skip_serializing_if = "Vec::is_empty")]
    pub consoles: Vec<FromToDevice>,

    #[serde(rename = "channel", default, skip_serializing_if = "Vec::is_empty")]
    pub channels: Vec<FromToDevice>,

    #[serde(rename = "input", default, skip_serializing_if = "Vec::is_empty")]
    pub inputs: Vec<Input>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub graphics: Vec<Graphics>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Disk {
    #[serde(rename = "$attr:type")]
    pub r#type: DiskType,

    #[serde(rename = "$attr:device", skip_serializing_if = "Option::is_none")]
    pub device: Option<DiskDevice>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub driver: Option<DiskDriver>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<Source>,

    // todo: mirror
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<DiskTarget>,
    // todo: iotune

    // todo: backingStore

    // todo: backenddomain
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boot: Option<DiskBoot>,

    // todo: encryption
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub readonly: Vec<Empty>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub shareable: Vec<Empty>,

    // todo: transient
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serial: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub wwn: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub product: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<Address>,
    // todo: auth
    // todo: geometry
    // todo: blockio
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiskType {
    File,
    Block,
    Dir,
    Network,
    Volume,
    NVME,
    VHostUser,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiskDevice {
    Disk,
    CDROM,
    LUN,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Source {
    #[serde(rename = "$attr:file", skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,

    #[serde(rename = "$attr:dev", skip_serializing_if = "Option::is_none")]
    pub dev: Option<String>,

    #[serde(rename = "$attr:dir", skip_serializing_if = "Option::is_none")]
    pub dir: Option<String>,

    #[serde(rename = "$attr:protocol", skip_serializing_if = "Option::is_none")]
    pub protocol: Option<DiskSourceProtocol>,

    #[serde(rename = "$attr:name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(rename = "$attr:tls", skip_serializing_if = "Option::is_none")]
    pub tls: Option<YesNo>,

    #[serde(rename = "$attr:query", skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,

    #[serde(rename = "$attr:pool", skip_serializing_if = "Option::is_none")]
    pub pool: Option<String>,

    #[serde(rename = "$attr:volume", skip_serializing_if = "Option::is_none")]
    pub volume: Option<String>,

    #[serde(rename = "$attr:mode", skip_serializing_if = "Option::is_none")]
    pub mode: Option<DiskMode>,

    #[serde(rename = "$attr:type", skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,

    #[serde(rename = "$attr:managed", skip_serializing_if = "Option::is_none")]
    pub nvme_managed: Option<YesNo>,

    #[serde(rename = "$attr:namespace", skip_serializing_if = "Option::is_none")]
    pub nvme_namespace: Option<usize>,

    #[serde(rename = "$attr:index", skip_serializing_if = "Option::is_none")]
    pub index: Option<usize>,

    #[serde(rename = "host", default, skip_serializing_if = "Vec::is_empty")]
    pub hosts: Vec<DiskSourceHost>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot: Option<DiskSourceSnapshot>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<DiskSourceConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<DiskSourceAuth>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub encryption: Option<DiskSourceEncryption>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reservations: Option<DiskSourceReservations>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<DiskSourceAddress>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub slices: Option<DiskSourceSlices>,
    // todo: ssl
    // todo: cookies
    // todo: readahead
    // todo: timeout
    // todo: identity
    // todo: reconnect
}

impl Source {
    pub fn file(f: impl AsRef<str>) -> Self {
        Self {
            file: Some(f.as_ref().into()),

            dev: None,
            dir: None,
            protocol: None,
            name: None,
            tls: None,
            query: None,
            pool: None,
            volume: None,
            mode: None,
            r#type: None,
            nvme_managed: None,
            nvme_namespace: None,
            index: None,
            hosts: vec![],
            snapshot: None,
            config: None,
            auth: None,
            encryption: None,
            reservations: None,
            address: None,
            slices: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiskSourceProtocol {
    NBD,
    iSCSI,
    RBD,
    SheepDog,
    Gluster,
    vXHS,
    NFS,
    HTTP,
    HTTPS,
    FTP,
    FTPS,
    TFTP,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiskMode {
    Direct,
    Host,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiskSourceHost {
    #[serde(rename = "$attr:name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(rename = "$attr:port", skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,

    #[serde(rename = "$attr:transport", skip_serializing_if = "Option::is_none")]
    pub transport: Option<String>,

    #[serde(rename = "$attr:socket", skip_serializing_if = "Option::is_none")]
    pub socket: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiskSourceSnapshot {
    #[serde(rename = "$attr:name")]
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiskSourceConfig {
    #[serde(rename = "$attr:file")]
    pub file: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiskSourceAuth {
    #[serde(rename = "$attr:username")]
    pub username: String,

    pub secret: DiskSourceAuthSecret,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiskSourceAuthSecret {
    #[serde(rename = "$attr:type")]
    pub r#type: String,

    #[serde(rename = "$attr:uuid", skip_serializing_if = "Option::is_none")]
    pub uuid: Option<Uuid>,

    #[serde(rename = "$attr:usage", skip_serializing_if = "Option::is_none")]
    pub usage: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiskSourceEncryption {
    // TODO
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiskSourceReservations {
    #[serde(rename = "$attr:managed")]
    pub managed: YesNo,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<DiskSourceReservationsSource>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiskSourceReservationsSource {
    #[serde(rename = "$attr:type")]
    pub r#type: String,

    #[serde(rename = "$attr:path")]
    pub path: String,

    #[serde(rename = "$attr:mode")]
    pub mode: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiskSourceInitiator {
    #[serde(rename = "$attr:name")]
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiskSourceAddress {
    #[serde(rename = "$value")]
    pub pci: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiskSourceSlices {
    // TODO
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiskTarget {
    #[serde(rename = "$attr:dev")]
    pub dev: String,

    #[serde(rename = "$attr:bus", skip_serializing_if = "Option::is_none")]
    pub bus: Option<String>,

    #[serde(rename = "$attr:tray", skip_serializing_if = "Option::is_none")]
    pub tray: Option<DiskTargetTray>,

    #[serde(rename = "$attr:removable", skip_serializing_if = "Option::is_none")]
    pub removable: Option<OnOff>,

    #[serde(
        rename = "$attr:rotation_rate",
        skip_serializing_if = "Option::is_none"
    )]
    pub rotation_rate: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiskTargetTray {
    Open,
    Closed,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiskDriver {
    #[serde(rename = "$attr:name")]
    pub name: String,

    #[serde(rename = "$attr:type", skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,

    #[serde(rename = "$attr:cache", skip_serializing_if = "Option::is_none")]
    pub cache: Option<DiskDriverCache>,

    #[serde(rename = "$attr:error_policy", skip_serializing_if = "Option::is_none")]
    pub error_policy: Option<DiskDriverErrorPolicy>,

    #[serde(
        rename = "$attr:rerror_policy",
        skip_serializing_if = "Option::is_none"
    )]
    pub read_error_policy: Option<DiskDriverErrorPolicy>,

    #[serde(rename = "$attr:io", skip_serializing_if = "Option::is_none")]
    pub io: Option<String>,

    #[serde(rename = "$attr:ioeventfd", skip_serializing_if = "Option::is_none")]
    pub ioeventfd: Option<OnOff>,

    #[serde(rename = "$attr:event_idx", skip_serializing_if = "Option::is_none")]
    pub event_idx: Option<OnOff>,

    #[serde(rename = "$attr:copy_on_read", skip_serializing_if = "Option::is_none")]
    pub copy_on_read: Option<OnOff>,

    #[serde(rename = "$attr:discard", skip_serializing_if = "Option::is_none")]
    pub discard: Option<DiskDriverTargetDiscard>,

    #[serde(
        rename = "$attr:detect_zeroes",
        skip_serializing_if = "Option::is_none"
    )]
    pub detect_zeroes: Option<DiskDriverDetectZeroes>,

    // todo: iothread
    // todo: queues
    // todo: queue_size
    #[serde(rename = "$attr:iommu", skip_serializing_if = "Option::is_none")]
    pub iommu: Option<OnOff>,

    #[serde(rename = "$attr:ats", skip_serializing_if = "Option::is_none")]
    pub ats: Option<OnOff>,

    #[serde(rename = "$attr:packed", skip_serializing_if = "Option::is_none")]
    pub packed: Option<OnOff>,

    #[serde(rename = "$attr:page_per_vq", skip_serializing_if = "Option::is_none")]
    pub page_per_vq: Option<OnOff>,
    // todo: metadata_cache
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiskDriverCache {
    Default,
    None,
    WriteThrough,
    WriteBack,
    DirectSync,
    Unsafe,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiskDriverErrorPolicy {
    Stop,
    Report,
    Ignore,
    ENOSPACE,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiskDriverTargetDiscard {
    Unmap,
    Ignore,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiskDriverDetectZeroes {
    On,
    Off,
    Unmap,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiskBoot {
    #[serde(rename = "$attr:order")]
    pub order: usize,

    #[serde(rename = "$attr:loadparm", skip_serializing_if = "Option::is_none")]
    pub loadparm: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Empty {
    #[serde(rename = "$value", skip)]
    // workaround to make empty element work
    pub mark: (),
}

impl Empty {
    pub fn new() -> Self {
        Self {
            mark: ()
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Address {
    #[serde(rename = "$attr:type")]
    pub typ: String,

    #[serde(rename = "$attr:controller", skip_serializing_if = "Option::is_none")]
    pub controller: Option<String>,

    #[serde(rename = "$attr:domain", skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,

    #[serde(rename = "$attr:bus", skip_serializing_if = "Option::is_none")]
    pub bus: Option<String>,

    #[serde(rename = "$attr:target", skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,

    #[serde(rename = "$attr:unit", skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,

    #[serde(rename = "$attr:slot", skip_serializing_if = "Option::is_none")]
    pub slot: Option<String>,

    #[serde(rename = "$attr:function", skip_serializing_if = "Option::is_none")]
    pub function: Option<String>,

    #[serde(
        rename = "$attr:multifunction",
        skip_serializing_if = "Option::is_none"
    )]
    pub multifunction: Option<String>,

    #[serde(rename = "$attr:port", skip_serializing_if = "Option::is_none")]
    pub port: Option<String>,
    // todo: zpci

    // todo: reg

    // todo: machine
    // todo: cssid
    // todo: ssid
    // todo: devno

    // todo: iobase
    // todo: irq
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Controller {
    #[serde(rename = "$attr:type")]
    pub typ: String,

    #[serde(rename = "$attr:index", skip_serializing_if = "Option::is_none")]
    pub index: Option<u64>,

    #[serde(rename = "$attr:ports", skip_serializing_if = "Option::is_none")]
    pub ports: Option<String>,

    #[serde(rename = "$attr:vectors", skip_serializing_if = "Option::is_none")]
    pub vectors: Option<String>,

    #[serde(rename = "$attr:model", skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    #[serde(
        rename = "$attr:maxGrantFrames",
        skip_serializing_if = "Option::is_none"
    )]
    pub max_grant_frames: Option<String>,

    #[serde(
        rename = "$attr:maxEventChannels",
        skip_serializing_if = "Option::is_none"
    )]
    pub max_event_channels: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub driver: Option<ControllerDriver>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub master: Option<ControllerMaster>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<Address>,
    // todo: target
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ControllerDriver {
    #[serde(rename = "$attr:queues", skip_serializing_if = "Option::is_none")]
    pub queues: Option<String>,
    #[serde(rename = "$attr:cmd_per_lun", skip_serializing_if = "Option::is_none")]
    pub cmd_per_lun: Option<String>,
    #[serde(rename = "$attr:max_sectors", skip_serializing_if = "Option::is_none")]
    pub max_sectors: Option<String>,
    #[serde(rename = "$attr:ioeventfd", skip_serializing_if = "Option::is_none")]
    pub ioeventfd: Option<String>,
    #[serde(rename = "$attr:iothread", skip_serializing_if = "Option::is_none")]
    pub iothread: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ControllerMaster {
    #[serde(rename = "$attr:startport", skip_serializing_if = "Option::is_none")]
    pub startport: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkInterface {
    #[serde(rename = "$attr:type")]
    pub typ: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub mac: Option<NetworkMac>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<NetworkSource>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<NetworkModel>,

    // todo: boot
    // todo: vlan
    // todo: virtualport
    // todo: ip vec
    // todo: route vec
    // todo: script
    // todo: downscript
    // todo: backenddomain
    // todo: target
    // todo: guest
    // todo: driver
    // todo: backend
    // todo: filterref
    // todo: tune
    // todo: teaming
    // todo: link
    // todo: mtu
    // todo: bandwidth
    // todo: port
    // todo: coalesce
    // todo: rom
    // todo: acpi
    // todo: alias
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<Address>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkMac {
    #[serde(rename = "$attr:address")]
    pub address: String,

    #[serde(rename = "$attr:type", skip_serializing_if = "Option::is_none")]
    pub typ: Option<String>,

    #[serde(rename = "$attr:check", skip_serializing_if = "Option::is_none")]
    pub check: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkSource {
    #[serde(rename = "$attr:bridge", skip_serializing_if = "Option::is_none")]
    pub bridge: Option<String>,

    #[serde(rename = "$attr:network", skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
    // todo much more
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkModel {
    #[serde(rename = "$attr:type", skip_serializing_if = "Option::is_none")]
    pub typ: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FromToDevice {
    #[serde(rename = "$attr:type", skip_serializing_if = "Option::is_none")]
    pub typ: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<Source>,
    pub target: FromToTarget,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<Address>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FromToSource {
    #[serde(rename = "$attr:path", skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FromToTarget {
    #[serde(rename = "$attr:type", skip_serializing_if = "Option::is_none")]
    pub typ: Option<String>,

    #[serde(rename = "$attr:path", skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    #[serde(rename = "$attr:port", skip_serializing_if = "Option::is_none")]
    pub port: Option<String>,

    #[serde(rename = "$attr:name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<FromToTargetModel>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FromToTargetModel {
    #[serde(rename = "$attr:name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Input {
    #[serde(rename = "$attr:type")]
    pub typ: String,

    #[serde(rename = "$attr:bus", skip_serializing_if = "Option::is_none")]
    pub bus: Option<String>,

    #[serde(rename = "$attr:model", skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub driver: Option<InputDriver>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<InputSource>,

    // todo: acpi
    // todo: alias
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<Address>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InputDriver {
    #[serde(rename = "$attr:iommu", skip_serializing_if = "Option::is_none")]
    pub iommu: Option<String>,

    #[serde(rename = "$attr:ats", skip_serializing_if = "Option::is_none")]
    pub ats: Option<String>,

    #[serde(rename = "$attr:packed", skip_serializing_if = "Option::is_none")]
    pub packed: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InputSource {
    #[serde(rename = "$attr:evdev", skip_serializing_if = "Option::is_none")]
    pub evdev: Option<String>,

    #[serde(rename = "$attr:grab", skip_serializing_if = "Option::is_none")]
    pub dev: Option<String>,

    #[serde(rename = "$attr:grab", skip_serializing_if = "Option::is_none")]
    pub grab: Option<String>,

    #[serde(rename = "$attr:repeat", skip_serializing_if = "Option::is_none")]
    pub repeat: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Graphics {
    #[serde(rename = "$attr:type", skip_serializing_if = "Option::is_none")]
    pub typ: Option<String>,

    #[serde(rename = "$attr:port", skip_serializing_if = "Option::is_none")]
    pub port: Option<usize>,

    #[serde(rename = "$attr:tlsPort", skip_serializing_if = "Option::is_none")]
    pub tls_port: Option<usize>,

    #[serde(rename = "$attr:autoport", skip_serializing_if = "Option::is_none")]
    pub auto_port: Option<String>,

    #[serde(rename = "listen", default, skip_serializing_if = "Vec::is_empty")]
    pub listeners: Vec<GraphicsListener>,

    // todo: add channel vec
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<GraphicsImage>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub streaming: Option<GraphicsStreaming>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub gl: Option<GraphicsGL>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphicsListener {
    #[serde(rename = "$attr:type", skip_serializing_if = "Option::is_none")]
    pub typ: Option<String>,
    // todo address + network + socket listener type
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphicsImage {
    #[serde(rename = "$attr:compression", skip_serializing_if = "Option::is_none")]
    pub compression: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphicsStreaming {
    #[serde(rename = "$attr:mode")]
    pub mode: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphicsGL {
    #[serde(rename = "$attr:enable", skip_serializing_if = "Option::is_none")]
    pub enable: Option<String>,

    #[serde(rename = "$attr:rendernode", skip_serializing_if = "Option::is_none")]
    pub rendernode: Option<String>,
}
