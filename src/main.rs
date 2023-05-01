mod protocol {
    use wayrs_client;
    use wayrs_client::protocol::*;
    wayrs_client::scanner::generate!("river-status-unstable-v1.xml");
}

use protocol::*;
use wayrs_client::protocol::*;

use wayrs_client::connection::Connection;
use wayrs_client::global::GlobalsExt;
use wayrs_client::IoMode;

use std::collections::HashMap;

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let mappings = args
        .chunks_exact(2)
        .map(|mapping| (mapping[0].to_owned(), mapping[1].to_owned()))
        .collect();

    let mut conn = Connection::connect().unwrap();
    let globals = conn.blocking_collect_initial_globals().unwrap();

    let seat: WlSeat = globals.bind(&mut conn, 1..=1).unwrap();
    let status_mgr: ZriverStatusManagerV1 = globals.bind(&mut conn, 5..=5).unwrap();
    let _seat_status = status_mgr.get_river_seat_status_with_cb(&mut conn, seat, seat_status_cb);
    status_mgr.destroy(&mut conn);

    let mut state = State {
        mappings,
        kbd_layouts: HashMap::new(),
    };

    loop {
        conn.flush(IoMode::Blocking).unwrap();
        conn.recv_events(IoMode::Blocking).unwrap();
        conn.dispatch_events(&mut state);
    }
}

struct State {
    mappings: HashMap<String, String>,
    kbd_layouts: HashMap<String, String>,
}

fn seat_status_cb(
    _: &mut Connection<State>,
    state: &mut State,
    _: ZriverSeatStatusV1,
    event: zriver_seat_status_v1::Event,
) {
    use zriver_seat_status_v1::Event;
    match event {
        Event::KeyboardLayout(event) => {
            let device = String::from_utf8_lossy(event.device.as_bytes());
            let layout = String::from_utf8_lossy(event.layout.as_bytes());
            let mapped = state
                .mappings
                .get(&*layout)
                .map(|s| s.as_str())
                .unwrap_or(&*layout);
            println!("{mapped}");
            state
                .kbd_layouts
                .insert(device.into_owned(), layout.into_owned());
        }
        Event::KeyboardLayoutClear(device) => {
            let device = String::from_utf8_lossy(device.as_bytes());
            state.kbd_layouts.remove(&*device);
            let layout = state
                .kbd_layouts
                .iter()
                .next()
                .map(|(_, layout)| layout.as_str())
                .unwrap_or("N/A");
            println!("{layout}");
        }
        _ => (),
    }
}
