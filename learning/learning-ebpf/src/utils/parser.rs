use core::error;

use aya_ebpf::{
    bindings::xdp_action, macros::{map,xdp}, maps::{RingBuf, xdp}, programs::XdpContext,
};
use learning_ebpf::NormalizedPacket;

use crate::utils::error::ParseError;
pub struct RequestData{
    source: u8,
    dst: u8,
    checksum: u16,
    data: u32     
}



impl RequestData {
    const default: RequestData =  RequestData{
        source: 0, dst : 0 , checksum: 0 , data:0
    };
    
}

pub fn Structure_TCP()-> RequestData{

    return RequestData::default;

}

pub fn Structure_UDP()-> RequestData{
    return RequestData::default;

}

pub fn check_access(
    RequestData { source, dst, checksum, data }: RequestData
)-> bool{

    return true;
}

// create a ring buffer for the header normalization between v4 and v6 
#[map]
static RING_BUFF: RingBuf = RingBuf::with_byte_size(256 * 4096, 0);

#[xdp]
pub fn normalized_packet(ctx: XdpContext)-> u32{
    match try_parse(&ctx) {
        Ok(Some(packet)) => {
            // Push normalized bytes safely into the ring buffer
            RING_BUFF.output(&packet, 0);
            
        }
        _ => {}
    }
    xdp_action::XDP_PASS

}

fn try_parse(ctx: &XdpContext) -> Result<Option<NormalizedPacket>, ()> {
    // 1. Parse Ethernet & IP (handling both IPv4 and IPv6 bytes uniformly)
    // 2. Extract UDP/QUIC source & destination ports
    
    let normalized = NormalizedPacket::default;
    Ok(Some(normalized))
}