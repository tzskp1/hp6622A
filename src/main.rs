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

fn main() {
    let mut port = create_port().expect("test");
    let res = port.write("++ver\n".as_bytes()).expect("test");
    let mut buf = String::new();
    let res2 = port.read_to_string(&mut buf);
    println!("{:?}", port.name());
    println!("{:?}", res);
    println!("{:?}", res2);
    println!("{:?}", buf);
}
