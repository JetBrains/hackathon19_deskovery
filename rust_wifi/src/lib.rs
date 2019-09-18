// TODO: add some errors
pub enum PortError {
    Error
}

pub type PortResult<T> = Result<T, PortError>;

pub trait Port {
    fn write(&mut self, message: &[u8]) -> PortResult<()>;
    fn write_message(&mut self, message: &str) -> PortResult<()> {
        self.write(message.as_bytes())?;
        self.write(&[13u8, 10])?;
        Ok(())
    }
    fn read(&mut self, out: &mut [u8]) -> PortResult<usize>;
    fn command(&mut self, message: &str, out: &mut [u8]) -> PortResult<usize> {
        self.write_message(message)?;
        let mut size = 0;
        // TODO: handle `ERROR`s
        while !contains(&out[..size], b"OK") {
            size += self.read(&mut out[size..])?;
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

    let size = port.command("AT", &mut buf[..])?;
    let answer = &buf[..size];
    let answer_str = std::str::from_utf8(answer).unwrap();
//    println!("{}", answer_str);


    let size = port.command("AT+CWMODE=1", &mut buf[..])?;
    let answer = &buf[..size];
//    println!("{}", std::str::from_utf8(answer).unwrap());


    let size = port.command("AT+CWJAP=\"JetBrains-Guest\",\"wifiusers90\"", &mut buf[..])?;
    let answer = &buf[..size];
//    println!("{}", std::str::from_utf8(answer).unwrap());
    return Ok(());
}