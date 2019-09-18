extern crate serial;

use std::io;
use std::io::prelude::*;
use std::ops::Index;
use std::time::Duration;

use serial::prelude::*;
use serial::SystemPort;

use wifi::{connect_to_wifi, Port, PortResult, ip_status, establish_connection, close_connection, send_data};

fn main() {
    let mut res = serial::open("/dev/cu.usbserial-14200");
    let mut port = res.unwrap();
    let mut std_port = interact(port).unwrap();
    do_example(&mut std_port)
}

fn do_example<T: SerialPort>(std_port: &mut StdPort<T>) {
    connect_to_wifi(std_port);
    let result = ip_status(std_port);
//    close_connection(std_port);
    establish_connection(std_port);
    send_data(std_port, "GET /poll HTTP/1.1\n\n");
    close_connection(std_port);
}

fn interact<T: SerialPort>(mut port: T) -> io::Result<StdPort<T>> {
    port.reconfigure(&|settings| {
        settings.set_baud_rate(serial::Baud115200)?;
        settings.set_char_size(serial::Bits8);
        settings.set_parity(serial::ParityNone);
        settings.set_stop_bits(serial::Stop1);
        settings.set_flow_control(serial::FlowNone);
        Ok(())
    })?;

    port.set_timeout(Duration::from_millis(5000))?;
    Ok(StdPort { serial_port: port })
}

pub struct StdPort<T : SerialPort> {
    serial_port: T
}

impl<T : SerialPort> Port for StdPort<T> {
    fn write(&mut self, message: &[u8]) -> PortResult<()> {
        self.serial_port.write(message).unwrap();
        Ok(())
    }

    fn read(&mut self, buf: &mut [u8]) -> PortResult<usize> {
        let size = self.serial_port.read(buf).unwrap();
        Ok(size)
    }
}
