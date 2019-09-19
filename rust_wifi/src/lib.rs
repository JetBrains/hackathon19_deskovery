use std::fmt::{Write, Error};
use std::cmp::min;

// TODO: add some errors
pub enum PortError {
    Error
}

pub type PortResult<T> = Result<T, PortError>;

pub trait Port {
    fn write(&mut self, message: &[u8]) -> PortResult<()>;
    fn write_message(&mut self, message: &[u8]) -> PortResult<()> {
        self.write(message)?;
        self.write(&[13u8, 10])?;
        Ok(())
    }
    fn read(&mut self, out: &mut [u8]) -> PortResult<usize>;
    fn command(&mut self, message: &[u8], out: &mut [u8], expected_message: &str) -> PortResult<usize> {
        println!("> {}", std::str::from_utf8(message).unwrap());
        self.write_message(message)?;
        let mut size = 0;
        // TODO: handle `ERROR`s
        while !contains(&out[..size], expected_message.as_bytes()) {
            let new_size = self.read(&mut out[size..])?;
            println!("< {}", std::str::from_utf8(&out[size..size + new_size]).unwrap());
            size += new_size;

        }
        Ok(size)
    }
}

pub struct Device<T : Port> {
    port: T,
    buf: [u8; 1024]
}

impl<T: Port> Device<T> {
    pub fn new(port: T) -> Device<T> {
        Device { port, buf: [0; 1024] }
    }

    pub fn ip_status(&mut self) -> PortResult<u8> {
        let size = self.port.command(b"AT+CIPSTATUS", &mut self.buf, "OK")?;
        print_response(&self.buf, size);

        if self.buf.starts_with(b"STATUS:") {
            println!("Status: {}", self.buf[7]);
            Ok(self.buf[7])
        } else {
            Err(PortError::Error)
        }
    }

    pub fn connect_to_wifi(&mut self) -> PortResult<()> {
        let size = self.port.command(b"AT", &mut self.buf[..], "OK")?;
        print_response(&self.buf, size);

        let size = self.port.command(b"AT+CWMODE=1", &mut self.buf[..], "OK")?;
        print_response(&self.buf, size);

        let size = self.port.command(b"AT+CWJAP=\"JetBrains-Guest\",\"deskoverynet\"", &mut self.buf[..], "OK")?;
        print_response(&self.buf, size);
        return Ok(());
    }

    pub fn close_connection(&mut self) -> PortResult<()> {
        let size = self.port.command(b"AT+CIPCLOSE", &mut self.buf, "CLOSED")?;
        print_response(&self.buf, size);
        Ok(())
    }

    pub fn establish_connection(&mut self) -> PortResult<()> {
        let size = self.port.command(b"AT+CIPSTART=\"TCP\",\"104.236.228.23\",8000", &mut self.buf, "OK")?;
        print_response(&self.buf, size);
        Ok(())
    }

    pub fn make_post_request(&mut self, message: &str) -> PortResult<Data> {
        self.establish_connection()?;

        // Send header
        self.send_data(b"GET /poll HTTP/1.1\n\n");

        // Send data
        let message_bytes = message.as_bytes();
        let chunk_size = 1024;
        // TODO: create issue with `let`
        let chunks_count = message_bytes.len() / chunk_size + if message_bytes.len() % chunk_size == 0 { 0 } else { 1 };
        for i in 0..chunks_count {
            self.send_data(&message_bytes[chunk_size * i..min(chunk_size * (i + 1), message_bytes.len())]);
        }

        self.close_connection();
        Ok(Data)
    }

    fn send_data(&mut self, data: &[u8]) -> PortResult<()> {
        let mut command = [0; 32];
        let mut write_buf = WriteBuf::new(&mut command);
        write!(write_buf, "AT+CIPSEND={}", data.len());
        let command_size = write_buf.count;
        let size = self.port.command(&command[..command_size], &mut self.buf, ">")?;
        self.port.write(data)
    }
}

// TODO: make it smarter
fn contains(src: &[u8], pattern: &[u8]) -> bool {
    src.windows(pattern.len()).any(|s| s == pattern)
}

pub struct Data;

fn print_response(buf: &[u8], size: usize) {
//    println!("{}", std::str::from_utf8(&buf[..size]).unwrap());
}

struct WriteBuf<'a> {
    buf: &'a mut [u8],
    count: usize
}

impl<'a> WriteBuf<'a> {
    fn new(buf: &mut [u8]) -> WriteBuf {
        WriteBuf { buf, count: 0 }
    }
}

impl<'a> Write for WriteBuf<'a> {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        for x in s.as_bytes().iter() {
            if self.count >= self.buf.len() {
                return Err(Error);
            }
            self.buf[self.count] = *x;
            self.count += 1;
        }
        Ok(())
    }
}
