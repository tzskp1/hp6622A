use std::time::Duration;

fn create_port() -> Option<Box<dyn serialport::SerialPort>> {
    let ports = serialport::available_ports().ok()?;
    let port = ports.into_iter().find(|p| match p.port_type {
        serialport::SerialPortType::UsbPort(serialport::UsbPortInfo {vid: 0x1b5f, pid: 0x9207, ..}) => {
            true
        },
        _ => {
            false
        }
    })?;
    serialport::new(port.port_name, 115_200).timeout(Duration::from_millis(500)).open().ok()
}

fn query(port: &mut Box<dyn serialport::SerialPort>, q: &str) -> Option<String> {
    let res = port.write((q.to_owned() + "\n").as_bytes()).ok()?;
    let mut resp = String::new();
    let mut buf = vec![0 as u8; 1000];
    loop {
        match port.read(&mut buf) {
            Ok(size) => {
                resp += &String::from_utf8(buf[0..size].to_vec())
                    .unwrap_or("".to_string());
                match resp.find(|x| x == '\n') {
                    Some(_) => {
                        break;
                    },
                    _ => ()
                }
            },
            _ => {
                break;
            }
        }
    }
    Some(resp)
}

fn main() {
    let mut port = create_port().expect("test");
    println!("{:?}", port.name());
    println!("{:?}", query(&mut port, "++ver"));
    // println!("{:?}", res2);
}
