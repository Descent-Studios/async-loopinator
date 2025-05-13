use std::{error::Error, net::UdpSocket, sync::mpsc, time::Instant};

use jack::{contrib, Client, ClientOptions, ClientStatus, Control, ProcessScope, RingBuffer};
use loop_line::{LoopLine, Param, StereoLine};
use serde_derive::{Deserialize, Serialize};
use serde_osc::{de, ser, Framing};

mod loop_line;

#[derive(Debug, Serialize, Deserialize)]
struct OscData {
    address: String,
    args: (f32, f32),
}

fn main() -> Result<(), Box<dyn Error>> {
    let (tx, rx) = mpsc::channel();
    println!("Hello, world!");
    let (client, _status) = Client::new("yeah", ClientOptions::default())?;
    println!(
        "name: {}, sample rate: {}, buffer size: {}",
        client.name(),
        client.sample_rate(),
        client.buffer_size()
    );
    let mut lines = [
        StereoLine::new(client.sample_rate() * 30),
        StereoLine::new(client.sample_rate() * 30),
        StereoLine::new(client.sample_rate() * 30),
        StereoLine::new(client.sample_rate() * 30),
    ];

    let in_a = client.register_port("r_in_1", jack::AudioIn::default())?;
    let in_b = client.register_port("r_in_2", jack::AudioIn::default())?;
    let mut out_a = client.register_port("r_out_1", jack::AudioOut::default())?;
    let mut out_b = client.register_port("r_out_2", jack::AudioOut::default())?;

    let process_callback = move |_: &Client, ps: &ProcessScope| -> Control {
        let message: Result<(usize, Param), mpsc::TryRecvError> = rx.try_recv();
        match message {
            Ok((idx, param)) => {
                lines[idx].send_param(param);
            }
            Err(_error) => {}
        }
        let out_a_p = out_a.as_mut_slice(ps);
        let out_b_p = out_b.as_mut_slice(ps);
        out_a_p.fill(0.0);
        out_b_p.fill(0.0);
        let in_a_p = in_a.as_slice(ps);
        let in_b_p = in_b.as_slice(ps);
        for line in lines.iter_mut() {
            line.write_slice(in_a_p, in_b_p);
            for sample in out_a_p.iter_mut() {
                *sample += line.l_line.read_advance(1);
            }
            for sample in out_b_p.iter_mut() {
                *sample += line.r_line.read_advance(1);
            }
        }
        Control::Continue
    };

    let process = contrib::ClosureProcessHandler::new(process_callback);

    let active_client = client.activate_async((), process)?;

    let mut record_time = Instant::now();
    {
        let socket = UdpSocket::bind("0.0.0.0:2322")?;

        loop {
            let mut buf = [0; 128];
            let (amt, _src) = socket.recv_from(&mut buf)?;

            let message: Result<OscData, serde_osc::error::Error> =
                de::from_slice(&buf[..amt], Framing::Unframed);

            if let Ok(data) = message {
                let mut addr = data.address;
                addr.remove(0);
                let (idx, addr) = addr.split_once("/").unwrap();
                //print!("{idx} ");

                let idx: usize = match idx {
                    "1" => 0,
                    "2" => 1,
                    "3" => 2,
                    "4" => 3,
                    "5" => 4,
                    "6" => 5,
                    _ => 0,
                };
                //println!("-> {idx}");

                match addr {
                    "feedback" => {
                        tx.send((idx, Param::Feedback(data.args.1)))?;
                    }
                    "record" => {
                        if (data.args.1 == 1.0) {
                            record_time = Instant::now();
                            tx.send((idx, Param::Clear))?;
                        } else {
                            let time_dur = record_time.elapsed();
                            let time_dur = time_dur.as_secs_f32() * 48000.0;
                            let time_dur = time_dur as usize;
                            tx.send((idx, Param::Time(time_dur)))?;
                        }
                    }
                    "input" => {
                        tx.send((idx, Param::Input(data.args.1)))?;
                    }
                    "output" => {
                        tx.send((idx, Param::Output(data.args.1)))?;
                    }
                    "clear" => {
                        tx.send((idx, Param::Clear))?;
                    }
                    "pan" => {
                        tx.send((idx, Param::Pan(data.args.1)));
                    }
                    _ => {}
                }
            }
        }
    }
}
