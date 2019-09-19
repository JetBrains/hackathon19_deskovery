use std::fmt::{Write, Error};
use std::cmp::min;
use data::ServerData;

// TODO: add some errors
pub enum PortError {
    Error,
    HttpError
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
    // returns (total_size, size_of_expected_message)
    fn read_while(&mut self, out: &mut [u8], mut total_size: usize, expected_message: &str) -> PortResult<(usize, usize)> {
        let mut index_result = index_of(&out[..total_size], expected_message.as_bytes());
        // TODO: handle `ERROR`s
        while index_result.is_none() {
            let size = self.read(&mut out[total_size..])?;
            println!("< {}", std::str::from_utf8(&out[total_size..total_size + size]).unwrap());
            total_size += size;
            index_result = index_of(&out[..total_size], expected_message.as_bytes())
        }
        let index = index_result.unwrap();
        Ok((total_size, index + expected_message.len()))
    }

    fn command(&mut self, message: &[u8], out: &mut [u8], expected_message: &str) -> PortResult<usize> {
        println!("> {}", std::str::from_utf8(message).unwrap());
        self.write_message(message)?;
        // TODO: handle `ERROR`s
        self.read_while(out, 0, expected_message).map(|x| x.0)
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

    pub fn connect_to_wifi_if_needed(&mut self) -> PortResult<()> {
        let status = self.ip_status()?;
        if status != 2 {
            let size = self.port.command(b"AT+CWJAP=\"JetBrains-Guest\",\"deskoverynet\"", &mut self.buf[..], "OK")?;
            print_response(&self.buf, size);
        }
        return Ok(());
    }

    pub fn close_connection(&mut self) -> PortResult<()> {
        let size = self.port.command(b"AT+CIPCLOSE", &mut self.buf, "CLOSED")?;
        print_response(&self.buf, size);
        Ok(())
    }

    pub fn establish_connection(&mut self, ip: &str, port: u32) -> PortResult<()> {
        let mut command = [0; 128];
        let mut write_buf = WriteBuf::new(&mut command);
        write!(write_buf, "AT+CIPSTART=\"TCP\",\"{}\",{}", ip, port);
        let command_size = write_buf.count;
        let command_slice = &command[..command_size];

        let size = self.port.command(command_slice, &mut self.buf, "OK")?;
        print_response(&self.buf, size);
        Ok(())
    }

    pub fn make_post_request(&mut self, message: &str, ip: &str, port: u32) -> PortResult<ServerData> {
        self.establish_connection(ip, port)?;

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

        let data = self.read_data()?;
        self.close_connection();
        Ok(data)
    }

    fn read_data(&mut self) -> PortResult<ServerData> {
        let mut buf = [0; 1024];
//        TODO: try to invoke convert using dereference quick fix
//        self.port.read_while(x, )
        let (total_size, message_size) = self.port.read_while(&mut buf, 0, "SEND OK")?;
        print_response(&buf, message_size);
        let rest_of_buf = &mut buf[message_size..];
        let (total_size2, message_size2) = self.port.read_while(rest_of_buf, total_size - message_size, "CLOSED")?;
        print_response(&rest_of_buf, message_size + total_size2);
        let str_data = std::str::from_utf8(&rest_of_buf[..message_size2]).unwrap();
        let option = str_data.lines().find(|line| line.contains("HTTP") && line.ends_with("OK"));
        if option.is_none() {
            return Err(PortError::HttpError)
        }
        let start_index = str_data.find("{");
        let end_index = str_data.find("}");
        match (start_index, end_index) {
            (Some(start), Some(end)) => Ok(Self::parse_data(&str_data[start..end])),
            _ => {
                println!("Failed to find json");
                Err(PortError::HttpError)
            }
        }
    }

    fn send_data(&mut self, data: &[u8]) -> PortResult<()> {
        let mut command = [0; 32];
        let mut write_buf = WriteBuf::new(&mut command);
        write!(write_buf, "AT+CIPSEND={}", data.len());
        let command_size = write_buf.count;
        let command_slice = &command[..command_size];
        let size = self.port.command(command_slice, &mut self.buf, ">")?;
        self.port.write(data)
    }

    fn parse_data(data: &str) -> ServerData {
        serde_json_core::de::from_str(data).unwrap()
    }
}

// TODO: make it smarter
fn index_of(src: &[u8], pattern: &[u8]) -> Option<usize> {
    src
        .windows(pattern.len())
        .enumerate()
        .find(|(_, s)| *s == pattern)
        .map(|(i, _)| i)
}

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
