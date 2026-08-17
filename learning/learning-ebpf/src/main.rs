#![no_std]
#![no_main]

use aya_ebpf::{
    bindings::xdp_action, 
    macros::{map,xdp}, 
    maps::HashMap,
    programs::XdpContext};


use core::mem;
use network_types::{
    eth::{EthHdr, EtherType},
    ip::{IpError, IpProto, Ipv4Hdr, Ipv6Hdr},
    tcp::TcpHdr,
    udp::UdpHdr,
};
use aya_log_ebpf::info;
use learning_common::{NormalizedPacket, RING_BUFF};

use crate::utils::{error::ParseError, parser::{Structure_TCP, Structure_UDP}};

mod utils;

#[xdp]
pub fn learning(ctx: XdpContext) -> u32 {
    match try_learning(ctx) {
        Ok(Ok(Some(packet))) => {
            // Push normalized bytes safely into the ring buffer
            let _ = RING_BUFF.output::<NormalizedPacket>(&packet, 0);
        },
        Ok(Err(ret)) => return ret,
        Ok(_) => {},
        Err(_) => return xdp_action::XDP_ABORTED,
    }
    return  xdp_action::XDP_PASS;
}


#[inline(always)] 
fn ptr_at<T>(ctx: &XdpContext, offset: usize) -> Result<*const T, ()> {
    let start = ctx.data();
    let end = ctx.data_end();
    let len = mem::size_of::<T>();

    if start + offset + len > end {
        return Err(());
    }

    Ok((start + offset) as *const T)
}

fn try_learning(ctx: XdpContext) -> Result<Result<Option<NormalizedPacket>,u32>, ParseError> {
    let ethhdr: *const EthHdr = ptr_at(&ctx, 0).unwrap(); 

    let iphdr: *const Ipv4Hdr = ptr_at(&ctx, EthHdr::LEN).unwrap();
    // let source_addr = u32::from_be_bytes(unsafe { (*ipv4hdr).src_addr });


    // Need to make this Proto normalized_hdr dependent not direct v4 v6 access header
    let proto = unsafe { (*iphdr).proto() }
        .map_err(|IpError::InvalidProto(_proto)| ()).unwrap();

    let iphdr = match unsafe { (*ethhdr).ether_type() }{
        Ok(EtherType::Ipv4) => {
            
        },
        Ok(EtherType::Ipv6) => {},
        _ => {},
    };

    let data: Option<utils::parser::RequestData> = match proto  {
        IpProto::Tcp => Some(Structure_TCP()),
        IpProto::Udp => Some(Structure_UDP()),
        _ => return Err(ParseError::UnsupportedProtocol),
    };    
    if !utils::parser::check_access(data.unwrap()){
        return Ok(Err(xdp_action::XDP_DROP));
    }
    let ipv4hdr: *const Ipv4Hdr = unsafe { ptr_at(&ctx, EthHdr::LEN).unwrap() };
    let source = u32::from_be_bytes(unsafe { (*ipv4hdr).src_addr });
    let action =
    match (system_ip(source), block_ip(source)) {
        (true, _) => xdp_action::XDP_PASS,   // system IP always wins, regardless of blocklist
        (false, true) => xdp_action::XDP_DROP,
        (false, false) => xdp_action::XDP_PASS,
    };

    if !system_ip(source) {
        info!(&ctx, "SRC IP: {:i}, ACTION: {}", source , action);
    }

    Ok(Err(action))
}
fn block_ip(address: u32) -> bool {
    unsafe { BLOCKLIST.get(&address).is_some() }
}

fn system_ip(address: u32) -> bool {
    unsafe { SYSTEM_LIST.get(&address).is_some() }
}

#[map]
static BLOCKLIST: HashMap<u32, u32> =
    HashMap::<u32, u32>::with_max_entries(1024, 0);

#[map]
static SYSTEM_LIST: HashMap<u32,u32> =  
    HashMap::<u32, u32>::with_max_entries(1024, 0);


#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[unsafe(link_section = "license")]
#[unsafe(no_mangle)]
static LICENSE: [u8; 13] = *b"Dual MIT/GPL\0";
