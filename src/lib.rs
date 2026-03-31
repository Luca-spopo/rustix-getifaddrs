#![cfg(unix)]

pub use nix::net::if_::InterfaceFlags;
pub use nix::sys::socket::SockaddrStorage;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct LinkStats64 {
    pub tx_packets: u64,
    pub rx_packets: u64,
    pub tx_bytes: u64,
    pub rx_bytes: u64,
    pub tx_errors: u64,
    pub rx_errors: u64,
    pub tx_dropped: u64,
    pub rx_dropped: u64,
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct InterfaceAddress {
    /// Name of the network interface
    pub interface_name: String,
    /// Flags as from `SIOCGIFFLAGS` ioctl
    pub flags: InterfaceFlags,
    /// Network address of this interface
    pub address: Option<SockaddrStorage>,
    /// Netmask of this interface
    pub netmask: Option<SockaddrStorage>,
    /// Broadcast address of this interface, if applicable
    pub broadcast: Option<SockaddrStorage>,
    /// Point-to-point destination address
    pub destination: Option<SockaddrStorage>,
    /// RTNL Link Stats
    pub link_stats_64: Option<LinkStats64>,
}

#[derive(Debug, Hash, PartialEq, Eq /* TODO: StructuralPartialEq? */)]
pub struct InterfaceAddressIterator {
    //TODO: Don't use nix, and use syscalls directly
    nix_iter: nix::ifaddrs::InterfaceAddressIterator,
}

fn read_sysfs_stat(interface_name: &str, stat_name: &str) -> Option<u64> {
    let path = format!("/sys/class/net/{interface_name}/statistics/{stat_name}");
    std::fs::read_to_string(path)
        .ok()?
        .trim()
        .parse::<u64>()
        .ok()
}

fn read_link_stats_64(interface_name: &str) -> Option<LinkStats64> {
    Some(LinkStats64 {
        tx_packets: read_sysfs_stat(interface_name, "tx_packets")?,
        rx_packets: read_sysfs_stat(interface_name, "rx_packets")?,
        tx_bytes: read_sysfs_stat(interface_name, "tx_bytes")?,
        rx_bytes: read_sysfs_stat(interface_name, "rx_bytes")?,
        tx_errors: read_sysfs_stat(interface_name, "tx_errors")?,
        rx_errors: read_sysfs_stat(interface_name, "rx_errors")?,
        tx_dropped: read_sysfs_stat(interface_name, "tx_dropped")?,
        rx_dropped: read_sysfs_stat(interface_name, "rx_dropped")?,
    })
}

impl core::iter::Iterator for InterfaceAddressIterator {
    type Item = InterfaceAddress;

    fn next(&mut self) -> Option<Self::Item> {
        self.nix_iter.next().map(|addr| {
            let interface_name = addr.interface_name;
            let link_stats_64 = read_link_stats_64(&interface_name);
            InterfaceAddress {
                interface_name,
                flags: addr.flags,
                address: addr.address,
                netmask: addr.netmask,
                broadcast: addr.broadcast,
                destination: addr.destination,
                link_stats_64,
            }
        })
    }
}

pub fn getifaddrs() -> Result<InterfaceAddressIterator, Box<dyn core::error::Error>> {
    nix::ifaddrs::getifaddrs()
        .map(|iter| InterfaceAddressIterator { nix_iter: iter })
        .map_err(|errno| Box::new(errno) as Box<dyn core::error::Error>)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_test() {
        assert_eq!(4, 4);
    }
}
