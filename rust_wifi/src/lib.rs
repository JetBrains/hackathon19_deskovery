use std::fmt::{Write, Error};

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

// TODO: make it smarter
fn contains(src: &[u8], pattern: &[u8]) -> bool {
    src.windows(pattern.len()).any(|s| s == pattern)
}

pub fn connect_to_wifi<T : Port>(port: &mut T) -> PortResult<()> {
    let mut buf = [0; 256];

    let size = port.command(b"AT", &mut buf[..], "OK")?;
    print_response(&buf, size);


    let size = port.command(b"AT+CWMODE=1", &mut buf[..], "OK")?;
    print_response(&buf, size);

    let size = port.command(b"AT+CWJAP=\"JetBrains-Guest\",\"wifiusers90\"", &mut buf[..], "OK")?;
    print_response(&buf, size);
    return Ok(());
}

pub fn ip_status<T : Port>(port: &mut T) -> PortResult<u8> {
    let mut buf = [0; 256];
    let size = port.command(b"AT+CIPSTATUS", &mut buf, "OK")?;
    print_response(&buf, size);

    if buf.starts_with(b"STATUS:") {
        println!("Status: {}", buf[7]);
        Ok(buf[7])
    } else {
        Err(PortError::Error)
    }
}

pub fn close_connection<T : Port>(port: &mut T) -> PortResult<()> {
    let mut buf = [0; 256];
    let size = port.command(b"AT+CIPCLOSE", &mut buf, "CLOSED")?;
    print_response(&buf, size);
    Ok(())
}

pub fn establish_connection<T : Port>(port: &mut T) -> PortResult<()> {
    let mut buf = [0; 256];
    let size = port.command(b"AT+CIPSTART=\"TCP\",\"104.236.228.23\",8000", &mut buf, "OK")?;
    print_response(&buf, size);
    Ok(())
}

pub fn send_data<T : Port>(port: &mut T, message: &str) -> PortResult<()> {
    let mut command = [0; 256];

    let mut write_buf = WriteBuf::new(&mut command);
    let mut buf = [0; 256];

    write!(&mut write_buf, "AT+CIPSEND={}", message.len());
    let command_size = write_buf.count;
    let size = port.command(&command[..command_size], &mut buf, ">")?;

    port.write(message.as_bytes());
    Ok(())
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
