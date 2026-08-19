#![no_std]
#![no_main]

use aya_ebpf::{
    bindings::xdp_action, macros::xdp, programs::XdpContext,
};

use core::mem;
use network_types::{
    eth::{EthHdr, EtherType}, ip::{IpProto, Ipv4Hdr, Ipv6Hdr}, tcp::TcpHdr, udp::UdpHdr,
};
use aya_log_ebpf::info;
use learning_common::{NormalizedPacket, RING_BUFF, get_or_assign_id};

use crate::utils::error::ParseError;

mod utils;

#[xdp]
pub fn learning(ctx: XdpContext) -> u32 {
    match try_learning(&ctx) {
        Ok(Some(packet)) => {
            // Push normalized bytes safely into the ring buffer
            let _ = RING_BUFF.output::<NormalizedPacket>(&packet, 0);
        }
        Ok(None) => {}
        Err(_) => {}
    }
    xdp_action::XDP_PASS
}

#[inline(always)]
fn ptr_at<T>(ctx: &XdpContext, offset: usize) -> Result<*const T, ParseError> {
    let start = ctx.data();
    let end = ctx.data_end();
    let len = mem::size_of::<T>();

    if start + offset + len > end {
        return Err(ParseError::InvalidEthernet);
    }

    Ok((start + offset) as *const T)
}

fn try_learning(ctx: &XdpContext) -> Result<Option<NormalizedPacket>, ParseError> {
    let ethhdr: *const EthHdr = ptr_at(ctx, 0)?;

    match unsafe { (*ethhdr).ether_type() } {
        Ok(EtherType::Ipv4) => {
            let ipv4: *const Ipv4Hdr = ptr_at(ctx, EthHdr::LEN)?;
            let proto = unsafe { (*ipv4).proto() }.map_err(|_| ParseError::Truncated)?;
            let src = unsafe { (*ipv4).src_addr };
            let dst = unsafe { (*ipv4).dst_addr };

            let mut pkt = NormalizedPacket::default();
            pkt.src_ip = u32::from_ne_bytes(src);
            pkt.dst_ip = u32::from_ne_bytes(dst);
            pkt.protocol = proto as u8;

            let tot_len = u16::from_be_bytes(unsafe { (*ipv4).tot_len });
            let ihl_bytes = (unsafe { (*ipv4).ihl() } as u32) * 4;
            pkt.payload_len = (tot_len as u32).saturating_sub(ihl_bytes);

            match proto {
                IpProto::Udp => {
                    let udphdr: *const UdpHdr = ptr_at(ctx, EthHdr::LEN + Ipv4Hdr::LEN)?;
                    pkt.src_port = unsafe { (*udphdr).src_port() };
                    pkt.dst_port = unsafe { (*udphdr).dst_port() };
                }
                IpProto::Tcp => {
                    let tcphdr: *const TcpHdr = ptr_at(ctx, EthHdr::LEN + Ipv4Hdr::LEN)?;
                    pkt.src_port = u16::from_be_bytes(unsafe { (*tcphdr).source });
                    pkt.dst_port = u16::from_be_bytes(unsafe { (*tcphdr).dest });

                    if pkt.src_port == 22 || pkt.dst_port == 22 {
                        return Ok(None);
                    }
                }
                _ => {}
            }

            info!(ctx, "This is ipv4");
            info!(ctx, "SRC IP: {:i}, SRC PORT: {}", pkt.src_ip, pkt.src_port);
            Ok(Some(pkt))
        }
        Ok(EtherType::Ipv6) => {
            let ipv6: *const Ipv6Hdr = ptr_at(ctx, EthHdr::LEN)?;
            let proto = unsafe { (*ipv6).next_hdr() }.map_err(|_| ParseError::Truncated)?;
            let src = unsafe { (*ipv6).src_addr };
            let dst = unsafe { (*ipv6).dst_addr };

            let mut pkt = NormalizedPacket::default();
            pkt.src_ip = get_or_assign_id(src);
            pkt.dst_ip = get_or_assign_id(dst);
            pkt.protocol = proto as u8;
            pkt.payload_len = u16::from_be_bytes(unsafe { (*ipv6).payload_len }) as u32;

            match proto {
                IpProto::Udp => {
                    let udphdr: *const UdpHdr = ptr_at(ctx, EthHdr::LEN + Ipv6Hdr::LEN)?;
                    pkt.src_port = unsafe { (*udphdr).src_port() };
                    pkt.dst_port = unsafe { (*udphdr).dst_port() };
                }
                IpProto::Tcp => {
                    let tcphdr: *const TcpHdr = ptr_at(ctx, EthHdr::LEN + Ipv6Hdr::LEN)?;
                    pkt.src_port = u16::from_be_bytes(unsafe { (*tcphdr).source });
                    pkt.dst_port = u16::from_be_bytes(unsafe { (*tcphdr).dest });

                    if pkt.src_port == 22 || pkt.dst_port == 22 {
                        return Ok(None);
                    }
                }
                _ => {}
            }

            info!(ctx, "This is ipv6");
            info!(ctx, "SRC IP ID: {}, SRC PORT: {}", pkt.src_ip, pkt.src_port);
            Ok(Some(pkt))
        }
        _ => {
            info!(ctx, "Got non-IP packet");
            Ok(None)
        }
    }
}


#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[unsafe(link_section = "license")]
#[unsafe(no_mangle)]
static LICENSE: [u8; 13] = *b"Dual MIT/GPL\0";

