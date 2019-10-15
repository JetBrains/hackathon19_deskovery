use crate::data::ServerData;
use core::fmt::{Error, Write};
use core::str::from_utf8;
use PortError::WriteError;

// TODO: add some errors
pub enum PortError {
    Error,
    HttpError,
    JsonError,
    WriteError,
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
    fn read_while(
        &mut self,
        out: &mut [u8],
        mut total_size: usize,
        expected_messages: &[&str],
    ) -> PortResult<(usize, usize)> {
        let mut index_result = None;
        let mut message_len = 0;
        for expected_message in expected_messages {
            let res = index_of(&out[..total_size], expected_message.as_bytes());
            if res.is_some() {
                index_result = res;
                message_len = expected_message.len();
                break;
            }
        }
        // TODO: handle `ERROR`s
        while index_result.is_none() {
            let size = self.read(&mut out[total_size..])?;
            #[cfg(feature = "std")]
            println!(
                "< {}",
                std::str::from_utf8(&out[total_size..total_size + size]).unwrap()
            );
            total_size += size;
            for expected_message in expected_messages {
                let res = index_of(&out[..total_size], expected_message.as_bytes());
                if res.is_some() {
                    index_result = res;
                    message_len = expected_message.len();
                    break;
                }
            }
        }
        let index = index_result.unwrap();
        Ok((total_size, index + message_len))
    }

    fn command(
        &mut self,
        message: &[u8],
        out: &mut [u8],
        expected_messages: &[&str],
    ) -> PortResult<usize> {
        #[cfg(feature = "std")]
        println!("> {}", std::str::from_utf8(message).unwrap());
        self.write_message(message)?;
        // TODO: handle `ERROR`s
        self.read_while(out, 0, expected_messages).map(|x| x.0)
    }
}

pub struct Device<T: Port> {
    pub brains: T,
    buf: [u8; 1024],
}

impl<T: Port> Device<T> {
    pub fn new(port: T) -> Device<T> {
        Device {
            brains: port,
            buf: [0; 1024],
        }
    }

    pub fn ip_status(&mut self) -> PortResult<u8> {
        let size = self
            .brains
            .command(b"AT+CIPSTATUS", &mut self.buf, &["OK"])?;
        print_response(&self.buf, size);

        let pattern = b"STATUS:";
        let status_index = index_of(&self.buf, pattern);
        //let s = from_utf8(&self.buf).unwrap();
        if let Some(index) = status_index {
            let status = self.buf[index + pattern.len()] - b'0';
            #[cfg(feature = "std")]
            println!("Status: {}", status);
            Ok(status)
        } else {
            Err(PortError::Error)
        }
    }

    pub fn connect_to_wifi_if_needed(&mut self) -> PortResult<()> {
        let status = self.ip_status()?;
        match status {
            2 => return Ok(()),
            3 => self.close_connection()?,
            _ => {}
        };

        let size = self.brains.command(
            b"AT+CWJAP=\"ELM NET\",\"tetkarumpumpel\"",
            &mut self.buf[..],
            &["OK"],
        )?;
        print_response(&self.buf, size);
        Ok(())
    }

    pub fn close_connection(&mut self) -> PortResult<()> {
        let size = self
            .brains
            .command(b"AT+CIPCLOSE", &mut self.buf, &["CLOSED"])?;
        print_response(&self.buf, size);
        Ok(())
    }

    pub fn establish_connection(&mut self, ip: &str, port: u32) -> PortResult<()> {
        let mut command = [0; 128];
        let mut write_buf = WriteBuf::new(&mut command);
        write!(write_buf, "AT+CIPSTART=\"TCP\",\"{}\",{}", ip, port).map_err(|_| WriteError)?;
        let command_size = write_buf.count;
        let command_slice = &command[..command_size];

        let size = self.brains.command(command_slice, &mut self.buf, &["OK"])?;
        print_response(&self.buf, size);
        Ok(())
    }

    pub fn make_post_request(
        &mut self,
        data_str: &str,
        ip: &str,
        port: u32,
    ) -> PortResult<ServerData> {
        self.establish_connection(ip, port)?;

        let mut command = [0; 1600];
        let mut write_buf = WriteBuf::new(&mut command);

        write!(write_buf, "POST /poll HTTP/1.1\r\nContent-Type: application/json\r\nConnection:close\r\nContent-Length: {}\r\n\r\n{}", data_str.len(), data_str).map_err(|_| WriteError)?;
        let command_size = write_buf.count;
        let command_slice = &command[..command_size];
        // Send data
        self.send_data(command_slice)?;

        let data = self.read_data()?;

        //        self.close_connection();
        Ok(data)
    }

    fn read_data(&mut self) -> PortResult<ServerData> {
        let mut buf = [0; 1024];
        //        TODO: try to invoke convert using dereference quick fix
        //        self.port.read_while(x, )
        let (total_size, message_size) = self.brains.read_while(&mut buf, 0, &["SEND OK"])?;
        print_response(&buf, message_size);
        let rest_of_buf = &mut buf[message_size..];
        let (total_size2, message_size2) =
            self.brains
                .read_while(rest_of_buf, total_size - message_size, &["CLOSED"])?;
        print_response(&rest_of_buf, message_size + total_size2);
        let str_data = from_utf8(&rest_of_buf[..message_size2]).unwrap();
        let option = str_data
            .lines()
            .find(|line| line.contains("HTTP") && line.ends_with("OK"));
        if option.is_none() {
            return Err(PortError::HttpError);
        }
        let start_index = str_data.find('{');
        let end_index = str_data.find('}');
        match (start_index, end_index) {
            (Some(start), Some(end)) => {
                Self::parse_data(&str_data[start..=end]).map_err(|e| PortError::JsonError)
            }
            _ => {
                #[cfg(feature = "std")]
                println!("Failed to find json");
                Err(PortError::HttpError)
            }
        }
    }

    fn send_data(&mut self, data: &[u8]) -> PortResult<()> {
        let mut command = [0; 1024];
        let mut write_buf = WriteBuf::new(&mut command);
        write!(write_buf, "AT+CIPSEND={}", data.len()).map_err(|_| WriteError)?;
        let command_size = write_buf.count;
        let command_slice = &command[..command_size];
        let size = self.brains.command(command_slice, &mut self.buf, &[">"])?;
        #[cfg(feature = "std")]
        println!("> {}", from_utf8(command_slice).unwrap());
        self.brains.write(data)?;
        //        self.port.read_while(&mut command, 0, "SEND OK")?;
        Ok(())
    }

    fn parse_data(data: &str) -> serde_json_core::de::Result<ServerData> {
        serde_json_core::de::from_str(data)
    }
}

// TODO: make it smarter
fn index_of(src: &[u8], pattern: &[u8]) -> Option<usize> {
    src.windows(pattern.len())
        .enumerate()
        .find(|(_, s)| *s == pattern)
        .map(|(i, _)| i)
}

fn print_response(buf: &[u8], size: usize) {
    #[cfg(feature = "std")]
    println!("{}", std::str::from_utf8(&buf[..size]).unwrap());
}

struct WriteBuf<'a> {
    buf: &'a mut [u8],
    count: usize,
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