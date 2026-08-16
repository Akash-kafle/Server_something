#![no_std]

// This file exists to enable the library target.

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NormalizedPacket {
    // Normalize both IPv4 and IPv6 into a uniform 16-byte array
    pub src_ip: [u8; 16],
    pub dst_ip: [u8; 16],
    pub src_port: u16,
    pub dst_port: u16,
    pub protocol: u8,    // e.g., 17 for UDP/QUic
    pub payload_len: u32,
}

impl NormalizedPacket{
    pub const default: NormalizedPacket = NormalizedPacket {
        src_ip: [0u8; 16],
        dst_ip: [0u8; 16],
        src_port: 0,
        dst_port: 0,
        protocol: 0,
        payload_len: 0,
    };
}


#[cfg(feature = "user")]
unsafe impl aya::Pod for NormalizedPacket {}