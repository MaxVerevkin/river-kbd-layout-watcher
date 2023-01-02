mod protocol {
    use wayrs_client;
    pub use wayrs_client::protocol::*;
    wayrs_scanner::generate!("river-status-unstable-v1.xml");
}
use protocol::*;

use wayrs_client::connection::Connection;
use wayrs_client::global::GlobalsExt;
use wayrs_client::proxy::{Dispatch, Dispatcher};
use wayrs_client::socket::IoMode;

use std::convert::Infallible;

fn main() {
    let mut conn = Connection::connect().unwrap();
    let globals = conn.blocking_collect_initial_globals().unwrap();

    let seat: WlSeat = globals.bind(&mut conn, 1..=1).unwrap();
    let status_mgr: ZriverStatusManagerV1 = globals.bind(&mut conn, 5..=5).unwrap();
    let _seat_status = status_mgr.get_river_seat_status(&mut conn, seat);
    status_mgr.destroy(&mut conn);

    let mut state = State;

    loop {
        conn.flush(IoMode::Blocking).unwrap();
        conn.recv_events(IoMode::Blocking).unwrap();
        conn.dispatch_events(&mut state).unwrap();
    }
}

struct State;

impl Dispatcher for State {
    type Error = Infallible;
}

impl Dispatch<ZriverSeatStatusV1> for State {
    fn event(
        &mut self,
        _: &mut Connection<Self>,
        _: ZriverSeatStatusV1,
        event: zriver_seat_status_v1::Event,
    ) {
        use zriver_seat_status_v1::Event;
        match event {
            Event::KeyboardLayout(event) => {
                let device = String::from_utf8_lossy(event.device.as_bytes());
                let layout = String::from_utf8_lossy(event.layout.as_bytes());
                println!("{device}: {layout}");
            }
            Event::KeyboardLayoutClear(device) => {
                let device = String::from_utf8_lossy(device.as_bytes());
                println!("{device}: <none>");
            }
            _ => (),
        }
    }
}

// Don't care
impl Dispatch<WlRegistry> for State {}
impl Dispatch<WlSeat> for State {}

// No events
impl Dispatch<ZriverStatusManagerV1> for State {}
