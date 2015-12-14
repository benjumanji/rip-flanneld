extern crate byteorder;
extern crate etcd;

use std::io::Write;
use std::net::{Ipv4Addr, UdpSocket};
use std::str::FromStr;

use byteorder::{ByteOrder, BigEndian};
use etcd::Client;

#[derive(Debug)]
struct RipEntry {
    address: Ipv4Addr,
    metric: u32,
}

impl RipEntry {
    fn new(address: Ipv4Addr, metric: u32) -> RipEntry {
        RipEntry { address: address, metric: metric }
    }

    fn to_bytes(&self) -> [u8; 20] {
        let mut buff = [0; 20];

        // always ip family.
        buff[1] = 2;

        // leave reserved gap and write out address.
        let octets = self.address.octets();
        for i in 0..4 {
            buff[i + 4] = octets[i];
        }

        // write out metric to last 4 bytes;
        { let end = &mut buff[16..20]; BigEndian::write_u32(end, self.metric); }
        buff
    }
}

#[derive(Debug)]
struct RipResponse {
    advertisements: Vec<RipEntry>
}

impl RipResponse {
    fn new(advertisements: Vec<RipEntry>) -> Option<RipResponse> {
        if advertisements.len() <= 25 {
            Some(RipResponse { advertisements: advertisements })
        } else {
            None
        }
    }

    fn to_bytes(&self) -> Box<[u8]> {
        let mut ret = Vec::<u8>::with_capacity(4 + 20 * self.advertisements.len());
        ret.extend(&[2,1,0,0]); // magic header = request.
        for a in &self.advertisements {
            let _ = ret.write_all(&a.to_bytes());
        }
        ret.into_boxed_slice()
    }
}

fn try_main() -> Result<(), String> {
    let client = try!(Client::new("foo").map_err(|_e| "bad url"));
        let idx: Option<u64> = None;
    loop {
        let res = try!(client.watch("/coreos.com/network/config", idx, true)
                       .map_err(|e| -> String { format!("{:?}", e) }));
        let idx = res.node.modifiedIndex;
    }
}


fn main() {
    let addr = Ipv4Addr::from_str("172.17.0.2").unwrap();
    let e = RipEntry::new(addr, 1);
    let mut v = Vec::new();
    v.push(e);
    let r = RipResponse::new(v).unwrap();
    let b = r.to_bytes();

    let brd_ip = Ipv4Addr::new(255, 255, 255, 255);
    let sock = UdpSocket::bind("0.0.0.0:520").unwrap();

    let _ = sock.send_to(&b, (brd_ip, 520));
}
