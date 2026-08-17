use core::error;

use aya_ebpf::{
    bindings::xdp_action, macros::{map,xdp}, maps::{RingBuf, xdp}, programs::XdpContext,
};

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

