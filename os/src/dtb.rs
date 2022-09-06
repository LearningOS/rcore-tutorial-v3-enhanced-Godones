use crate::drivers::rtc::{init_rtc, QEMU_RTC};
use crate::sync::UPIntrFreeCell;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt::{Debug, Formatter};
use device_tree::util::SliceRead;
use device_tree::{DeviceTree, Node};
use lazy_static::lazy_static;
use log::{info, trace};

pub fn init_dtb(addr: usize) {
    trace!("init dtb at 0x{:x}", addr);
    #[repr(C)]
    struct DtbHeader {
        be_magic: u32,
        be_size: u32,
    }
    let header = unsafe { &*(addr as *const DtbHeader) };
    // from_be 是大小端序的转换（from big endian）
    let magic = u32::from_be(header.be_magic);
    const DEVICE_TREE_MAGIC: u32 = 0xd00dfeed;
    // 验证 Device Tree Magic Number
    assert_eq!(magic, DEVICE_TREE_MAGIC);
    let size = u32::from_be(header.be_size);
    let dtb_data = unsafe { core::slice::from_raw_parts(addr as *const u8, size as usize) };
    let dt = DeviceTree::load(dtb_data).expect("failed to parse device tree");
    walk_dt_node(&dt.root); //从根节点遍历
    for device in DEVICES.exclusive_access().iter() {
        println!("{:?}", device);
    }
}

fn walk_dt_node(dt: &Node) {
    if let Ok(_compatible) = dt.prop_str("compatible") {
        device_probe(dt);
    }
    for child in dt.children.iter() {
        // 遍历子节点
        walk_dt_node(child);
    }
}

#[derive(Default)]
struct DeviceBase {
    name: String,
    base_addr: usize,
    size: usize,
    irq: u32,
}

impl Debug for DeviceBase {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "DeviceBase {{ name: {:?}, base_addr: {:#x}, size: {:#x}, irq: {:?} }}",
            self.name, self.base_addr, self.size, self.irq
        )
    }
}

#[derive(Default, Debug)]
struct UartDevice {
    base: DeviceBase,
    baud_rate: u32,
}
#[derive(Default, Debug)]
struct RtcDevice {
    base: DeviceBase,
}
#[derive(Default, Debug)]
struct VirtioDevice {
    base: DeviceBase,
}
#[derive(Debug, Default)]
struct PlicDevice {
    base: DeviceBase,
}

#[derive(Debug)]
enum Device {
    Uart(UartDevice),
    Rtc(RtcDevice),
    VirtIo(VirtioDevice),
    Plic(PlicDevice),
    Unknown,
}

lazy_static! {
    static ref DEVICES: UPIntrFreeCell<Vec<Device>> = unsafe { UPIntrFreeCell::new(Vec::new()) };
}

fn register_range(node: &Node) -> (usize, usize) {
    if let Some(reg) = node.prop_raw("reg") {
        let paddr = reg.as_slice().read_be_u64(0).unwrap();
        let size = reg.as_slice().read_be_u64(8).unwrap();
        let vaddr = paddr;
        return (vaddr as usize, size as usize);
    }
    (0, 0)
}
fn irq_number(node: &Node) -> u32 {
    if let Ok(irq) = node.prop_u32("interrupts") {
        return irq;
    }
    0
}

fn device_probe(node: &Node) {
    let compatible = node.prop_str("compatible");
    if let Ok(str) = compatible {
        let mut base_info = DeviceBase::default();
        base_info.name = str.to_string();
        match str {
            "ns16550a"
            | "google,goldfish-rtc"
            | "virtio,mmio"
            | "sifive,plic-1.0.0\0riscv,plic0" => {
                (base_info.base_addr, base_info.size) = register_range(node);
                base_info.irq = irq_number(node);
            }
            _ => (),
        }
        match str {
            "ns16550a" => {
                let freq = node.prop_u32("clock-frequency").unwrap_or(0);
                assert!(freq > 0);
                let mut uart_info = UartDevice::default();
                uart_info.base = base_info;
                uart_info.baud_rate = freq;
                DEVICES.exclusive_access().push(Device::Uart(uart_info));
            }
            "google,goldfish-rtc" => {
                let mut rtc_info = RtcDevice::default();
                rtc_info.base = base_info;
                DEVICES.exclusive_access().push(Device::Rtc(rtc_info));
            }
            "virtio,mmio" => {
                let mut virtio_info = VirtioDevice::default();
                virtio_info.base = base_info;
                DEVICES.exclusive_access().push(Device::VirtIo(virtio_info));
            }
            "sifive,plic-1.0.0\0riscv,plic0" => {
                let mut plic_info = PlicDevice::default();
                plic_info.base = base_info;
                DEVICES.exclusive_access().push(Device::Plic(plic_info));
            }
            _ => (),
        }
    }
}

pub fn init_device() {
    DEVICES
        .exclusive_access()
        .iter()
        .for_each(|device| match device {
            Device::Rtc(rtc_device) => {
                init_rtc(rtc_device.base.base_addr, rtc_device.base.irq);
                let time = unsafe { QEMU_RTC.get().unwrap().read_time() };
                println!("time: {:?}", time);
                unsafe {
                    QEMU_RTC.get().unwrap().enable_irq();
                    QEMU_RTC.get().unwrap().set_alarm_with_next_s(1); //每分钟中断一次更新，时间
                }
            }
            _ => (),
        })
}

#[allow(unused)]
pub fn dtb(addr: usize) {
    use dtb_walker::{utils::indent, Dtb, DtbObj, HeaderError as E, WalkOperation as Op};

    info!("init dtb at 0x{:x}", addr);
    #[repr(C)]
    struct DtbHeader {
        be_magic: u32,
        be_size: u32,
    }
    let header = unsafe { &*(addr as *const DtbHeader) };
    // from_be 是大小端序的转换（from big endian）
    let magic = u32::from_be(header.be_magic);
    const DEVICE_TREE_MAGIC: u32 = 0xd00dfeed;
    // 验证 Device Tree Magic Number
    assert_eq!(magic, DEVICE_TREE_MAGIC);
    let size = u32::from_be(header.be_size);
    let dtb_data = unsafe { core::slice::from_raw_parts(addr as *const u8, size as usize) };
    const INDENT_WIDTH: usize = 4;

    let dtb = unsafe {
        Dtb::from_raw_parts_filtered(dtb_data.as_ptr() as _, |e| {
            matches!(e, E::Misaligned(4) | E::LastCompVersion(16))
        })
    }
    .map_err(|e| format!("verify header failed: {e:?}"))
    .unwrap();
    dtb.walk(|path, obj| match obj {
        DtbObj::SubNode { name } => {
            info!("{}{path}/{name}", indent(path.level(), INDENT_WIDTH));
            Op::StepInto
        }
        DtbObj::Property(prop) => {
            let indent = indent(path.level(), INDENT_WIDTH);
            info!("{indent}{prop:?}");
            Op::StepOver
        }
    });
}
