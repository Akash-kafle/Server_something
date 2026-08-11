#![no_std]
#![no_main]

use aya_ebpf::{
    bindings::xdp_action, 
    macros::{map,xdp}, 
    maps::HashMap,
    programs::XdpContext};
use aya_log_ebpf::info;

use core::mem;
use network_types::{
    eth::{EthHdr, EtherType},
    ip::{IpError, IpProto, Ipv4Hdr},
    tcp::TcpHdr,
    udp::UdpHdr,
};

#[xdp]
pub fn learning(ctx: XdpContext) -> u32 {
    match try_learning(ctx) {
        Ok(ret) => ret,
        Err(_) => xdp_action::XDP_ABORTED,
    }
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

fn try_learning(ctx: XdpContext) -> Result<u32, ()> {
    let ethhdr: *const EthHdr = ptr_at(&ctx, 0)?; 
    match unsafe { (*ethhdr).ether_type() } {
        Ok(EtherType::Ipv4) => {}
        _ => return Ok(xdp_action::XDP_PASS),
    }

    let ipv4hdr: *const Ipv4Hdr = ptr_at(&ctx, EthHdr::LEN)?;
    // let source_addr = u32::from_be_bytes(unsafe { (*ipv4hdr).src_addr });

    let proto = unsafe { (*ipv4hdr).proto() }
        .map_err(|IpError::InvalidProto(_proto)| ())?;

    let (tcphdr, udphdr): (Option<TcpHdr>, Option<UdpHdr>) = match proto  {
        IpProto::Tcp => {
            let tcphdr: *const TcpHdr =
                ptr_at(&ctx, EthHdr::LEN + Ipv4Hdr::LEN)?;
            unsafe {(Some(*tcphdr),None)}
        }
        IpProto::Udp => {
            let udphdr: *const UdpHdr =
                ptr_at(&ctx, EthHdr::LEN + Ipv4Hdr::LEN)?;
            unsafe { (None,Some(*udphdr)) }
        }
        _ => return Err(()),
    };    
    
    let ipv4hdr: *const Ipv4Hdr = unsafe { ptr_at(&ctx, EthHdr::LEN)? };
    let source = u32::from_be_bytes(unsafe { (*ipv4hdr).src_addr });
    let action = 
    match (system_ip(source), block_ip(source)) {
        (true, _) => xdp_action::XDP_PASS,   // system IP always wins, regardless of blocklist
        (false, true) => xdp_action::XDP_DROP,
        (false, false) => xdp_action::XDP_PASS,
    };

    if !system_ip(source) {
        // info!(&ctx, " Tcp : {}, UDP: {} ", tcphdr.unwrap(),udphdr.unwrap());

        info!(&ctx, "SRC IP: {:i}, ACTION: {}", source , action);
    }

    Ok(action)
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
