use iced::{
    text_input, button, executor, time, Align, Application, Button, Clipboard, Column,
    Command, Container, Element, HorizontalAlignment, Length, Row, Settings,
    Subscription, Text, TextInput,
};
use std::{time::{Duration, Instant}, sync::{Arc, Mutex}};

pub fn create_port() -> Option<Box<dyn serialport::SerialPort>> {
    let ports = serialport::available_ports().ok()?;
    let port = ports.into_iter().find(|p| match p.port_type {
        serialport::SerialPortType::UsbPort(serialport::UsbPortInfo {vid: 0x1b5f, pid: 0x9207, ..}) => {
            true
        },
        _ => {
            false
        }
    })?;
    serialport::new(port.port_name, 2_000_000).timeout(Duration::from_millis(500)).open().ok()
}

pub fn query(port: &mut Box<dyn serialport::SerialPort>, q: &str) -> Option<String> {
    let s = port.write((q.to_owned() + "\n").as_bytes()).ok()?;
    assert_eq!(s, q.len() + 1);
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
    let resp = resp.replace(&[' ', '\r', '\n'][..], "");
    Some(resp)
}

pub fn command(port: &mut Box<dyn serialport::SerialPort>, q: &str) -> Option<()> {
    let s = port.write((q.to_owned() + "\n").as_bytes()).ok()?;
    assert_eq!(s, q.len() + 1);
    Some(())
}

#[derive(Debug, Clone)]
pub struct Status {
    pub vsets: (f64, f64),
    pub vouts: (f64, f64),
    pub isets: (f64, f64),
    pub iouts: (f64, f64),
    pub outs: (u64, u64),
    pub ovsets: (f64, f64),
    pub ocps: (u64, u64),
}

/*
pub fn read_status(port: &mut Box<dyn serialport::SerialPort>) -> Option<Status> {
    let vsets = (query(port, "VSET? 1")?.parse::<f64>().ok()?, query(port, "VSET? 2")?.parse::<f64>().ok()?);
    let vouts = (query(port, "VOUT? 1")?.parse::<f64>().ok()?, query(port, "VOUT? 2")?.parse::<f64>().ok()?);
    let isets = (query(port, "ISET? 1")?.parse::<f64>().ok()?, query(port, "ISET? 2")?.parse::<f64>().ok()?);
    let iouts = (query(port, "IOUT? 1")?.parse::<f64>().ok()?, query(port, "IOUT? 2")?.parse::<f64>().ok()?);
    let outs = (query(port, "OUT? 1")?.parse::<u64>().ok()?, query(port, "OUT? 2")?.parse::<u64>().ok()?);
    let ovsets = (query(port, "OVSET? 1")?.parse::<f64>().ok()?, query(port, "OVSET? 2")?.parse::<f64>().ok()?);
    let ocps = (query(port, "OCP? 1")?.parse::<f64>().ok()?, query(port, "OCP? 2")?.parse::<f64>().ok()?);
    Some(Status {vsets, vouts, isets, iouts, outs, ovsets, ocps})
}
*/

fn vec2tuple<T>(xs: Vec<T>) -> (T, T)
where T: Copy
{
    (xs[0].clone(), xs[1].clone())
}

fn oiter2vec<T, U>(xs: U) -> Vec<T>
where U : Iterator<Item = Option<T>>
{
    xs.fold(Vec::new(), |mut x, y| {
        match y {
            Some(yv) => {
                x.push(yv);
                x
            },
            None => {
                x
            }
        }
    })
}

pub fn read_status(port: &mut Box<dyn serialport::SerialPort>) -> Option<Status> {
    let s = port.write("++macro 1\n".as_bytes()).ok()?;
    assert_eq!(s, 10);
    let mut resp = String::new();
    let mut buf = vec![0 as u8; 1000];
    loop {
        match port.read(&mut buf) {
            Ok(size) => {
                resp += &String::from_utf8(buf[0..size].to_vec())
                    .unwrap_or("".to_string());
                if resp.chars().filter(|x| *x == '\n').count() == 14 {
                    break;
                }
            },
            _ => {
                break;
            }
        }
    }
    let resp = resp.replace(&[' ', '\r'][..], "");
    let xs = resp.split(|c| c == '\n').collect::<Vec<_>>().into_iter().filter(|x| x.len() != 0).collect::<Vec<_>>();
    if xs.len() != 14 {
        None
    } else {
        let vsets = vec2tuple(oiter2vec(xs[0..2].into_iter().map(|x| x.parse::<f64>().ok())));
        let vouts = vec2tuple(oiter2vec(xs[2..4].into_iter().map(|x| x.parse::<f64>().ok())));
        let isets = vec2tuple(oiter2vec(xs[4..6].into_iter().map(|x| x.parse::<f64>().ok())));
        let iouts = vec2tuple(oiter2vec(xs[6..8].into_iter().map(|x| x.parse::<f64>().ok())));
        let outs = vec2tuple(oiter2vec(xs[8..10].into_iter().map(|x| x.parse::<u64>().ok())));
        let ovsets = vec2tuple(oiter2vec(xs[10..12].into_iter().map(|x| x.parse::<f64>().ok())));
        let ocps = vec2tuple(oiter2vec(xs[12..14].into_iter().map(|x| x.parse::<u64>().ok())));
        Some(Status {vsets, vouts, isets, iouts, outs, ovsets, ocps})
    }
}

// fn main() {
//     let mut port = create_port().expect("test");
//     println!("{:?}", port.name());
//     let st = Instant::now();
//     // println!("{:?}", query(&mut port, "++macro 1"));
//     // println!("{:?}", st.elapsed());
//     println!("{:?}", read_status(&mut port));
//     println!("{:?}", st.elapsed());
//     println!("{:?}", read_status(&mut port));
//     println!("{:?}", st.elapsed());
// }
//
pub fn main() -> iced::Result {
    Panel::run(Settings::default())
}

fn tick(x: Instant) -> Message {
    Message::Tick(x, None)
}

struct PanelBlock<T> {
    button: button::State,
    text_input: text_input::State,
    ti_content: T,
}

impl<T> PanelBlock<T> {
    pub fn new(val: T) -> Self {
        PanelBlock { button: button::State::new(), text_input: text_input::State::new(), ti_content: val }
    }
}

struct Panel {
    port: Option<Arc<Mutex<Box<dyn serialport::SerialPort>>>>,
    status: Option<Status>,
    vset1: PanelBlock<String>,
    vset2: PanelBlock<String>,
    iset1: PanelBlock<String>,
    iset2: PanelBlock<String>,
}

#[derive(Debug, Clone)]
enum Target {
    Volt(f64),
    Ampere(f64),
    Out(u64),
    OV(f64),
    OCP(u64),
}

#[derive(Debug, Clone)]
enum Message {
    Set(u64, Target),
    SetInput(u64, Option<Target>, String),
    Tick(Instant, Option<Status>),
}

impl Application for Panel {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Panel, Command<Message>) {
        (
            Panel {
                port: create_port().map(|x| Arc::new(Mutex::new(x))),
                status: None,
                vset1: PanelBlock::new("".to_string()),
                vset2: PanelBlock::new("".to_string()),
                iset1: PanelBlock::new("".to_string()),
                iset2: PanelBlock::new("".to_string()),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Panel")
    }

    fn update(
        &mut self,
        message: Message,
        _clipboard: &mut Clipboard,
    ) -> Command<Message> {
        match message {
            Message::Tick(now, None) => match &self.port {
                Some(port) => {
                    let port = Arc::clone(port);
                    Command::perform(async {port}, move |port| {
                        let status = port.lock().ok().and_then(|mut port| {
                            read_status(&mut port)
                        });
                        Message::Tick(now, status)
                    })
                },
                None => Command::none()
            },
            Message::Tick(_, Some(status)) => {
                self.status = Some(status);
                Command::none()
            },
            Message::SetInput(1, Some(Target::Volt(_)), val) => {
                self.vset1.ti_content = val;
                Command::none()
            },
            Message::SetInput(2, Some(Target::Volt(_)), val) => {
                self.vset2.ti_content = val;
                Command::none()
            },
            Message::SetInput(1, Some(Target::Ampere(_)), val) => {
                self.iset1.ti_content = val;
                Command::none()
            },
            Message::SetInput(2, Some(Target::Ampere(_)), val) => {
                self.iset2.ti_content = val;
                Command::none()
            },
            Message::SetInput(_, _, _) => {
                Command::none()
            },
            Message::Set(ch, Target::Volt(val)) => {
                match &self.port {
                    Some(port) => {
                        let port = Arc::clone(&port);
                        Command::perform(async {port}, move |port| {
                            port.lock().ok().and_then(|mut port| {
                                query(&mut port, format!("VSET {},{}", ch, val).as_str())
                            });
                            Message::Tick(Instant::now(), None)
                        })
                    },
                    None => Command::none()
                }
            },
            Message::Set(ch, Target::Ampere(val)) => {
                match &self.port {
                    Some(port) => {
                        let port = Arc::clone(&port);
                        Command::perform(async {port}, move |port| {
                            port.lock().ok().and_then(|mut port| {
                                query(&mut port, format!("ISET {},{}", ch, val).as_str())
                            });
                            Message::Tick(Instant::now(), None)
                        })
                    },
                    None => Command::none()
                }
            },
            Message::Set(ch, Target::OCP(val)) => {
                match &self.port {
                    Some(port) => {
                        let port = Arc::clone(&port);
                        Command::perform(async {port}, move |port| {
                            port.lock().ok().and_then(|mut port| {
                                query(&mut port, format!("OCP {},{}", ch, val).as_str())
                            });
                            Message::Tick(Instant::now(), None)
                        })
                    },
                    None => Command::none()
                }
            },
            Message::Set(ch, Target::Out(val)) => {
                match &self.port {
                    Some(port) => {
                        let port = Arc::clone(&port);
                        Command::perform(async {port}, move |port| {
                            port.lock().ok().and_then(|mut port| {
                                query(&mut port, format!("OUT {},{}", ch, val).as_str())
                            });
                            Message::Tick(Instant::now(), None)
                        })
                    },
                    None => Command::none()
                }
            },
            Message::Set(ch, Target::OV(val)) => {
                match &self.port {
                    Some(port) => {
                        let port = Arc::clone(&port);
                        Command::perform(async {port}, move |port| {
                            port.lock().ok().and_then(|mut port| {
                                query(&mut port, format!("OVSET {},{}", ch, val).as_str())
                            });
                            Message::Tick(Instant::now(), None)
                        })
                    },
                    None => Command::none()
                }
            },
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        time::every(Duration::from_millis(500)).map(tick)
    }

    fn view(&mut self) -> Element<Message> {
        let button = |state, label, style| {
            Button::new(
                state,
                Text::new(label)
                    .horizontal_alignment(HorizontalAlignment::Center),
            )
            .min_width(80)
            .padding(10)
            .style(style)
        };

        let ch1v = Text::new(self.status.as_ref().map(|x| format!("{} V", x.vouts.0)).unwrap_or("".to_string())).size(40).width(iced::Length::Units(200));
        let ch1vi = TextInput::new(&mut self.vset1.text_input, "", &self.vset1.ti_content, |x| Message::SetInput(1, x.parse::<f64>().ok().map(Target::Volt), x)).size(40).width(iced::Length::Units(150));
        let vset1 =
            button(&mut self.vset1.button, "VSET", style::Button::Primary)
                .on_press(Message::Set(1, Target::Volt(self.vset1.ti_content.parse::<f64>().unwrap_or(0.0))));
        let ch1v = Row::new()
            .align_items(Align::Center)
            .spacing(20)
            .push(ch1v)
            .push(ch1vi)
            .push(vset1);

        let ch1i = Text::new(self.status.as_ref().map(|x| format!("{} A", x.iouts.0)).unwrap_or("".to_string())).size(40).width(iced::Length::Units(200));
        let ch1ii = TextInput::new(&mut self.iset1.text_input, "", &self.iset1.ti_content, |x| Message::SetInput(1, x.parse::<f64>().ok().map(Target::Ampere), x)).size(40).width(iced::Length::Units(150));
        let iset1 =
            button(&mut self.iset1.button, "ISET", style::Button::Primary)
                .on_press(Message::Set(1, Target::Volt(self.iset1.ti_content.parse::<f64>().unwrap_or(0.0))));
        let ch1i = Row::new()
            .align_items(Align::Center)
            .spacing(20)
            .push(ch1i)
            .push(ch1ii)
            .push(iset1);

        let ch2v = Text::new(self.status.as_ref().map(|x| format!("{} V", x.vouts.1)).unwrap_or("".to_string())).size(40).width(iced::Length::Units(200));
        let ch2vi = TextInput::new(&mut self.vset2.text_input, "", &self.vset2.ti_content, |x| Message::SetInput(2, x.parse::<f64>().ok().map(Target::Volt), x)).size(40).width(iced::Length::Units(150));
        let vset2 =
            button(&mut self.vset2.button, "VSET", style::Button::Primary)
                .on_press(Message::Set(2, Target::Volt(self.vset2.ti_content.parse::<f64>().unwrap_or(0.0))));
        let ch2v = Row::new()
            .align_items(Align::Center)
            .spacing(20)
            .push(ch2v)
            .push(ch2vi)
            .push(vset2);

        let ch2i = Text::new(self.status.as_ref().map(|x| format!("{} A", x.iouts.1)).unwrap_or("".to_string())).size(40).width(iced::Length::Units(200));
        let ch2ii = TextInput::new(&mut self.iset2.text_input, "", &self.iset2.ti_content, |x| Message::SetInput(2, x.parse::<f64>().ok().map(Target::Ampere), x)).size(40).width(iced::Length::Units(150));
        let iset2 =
            button(&mut self.iset2.button, "ISET", style::Button::Primary)
                .on_press(Message::Set(2, Target::Volt(self.iset2.ti_content.parse::<f64>().unwrap_or(0.0))));
        let ch2i = Row::new()
            .align_items(Align::Center)
            .spacing(20)
            .push(ch2i)
            .push(ch2ii)
            .push(iset2);

        let ch1 = Column::new()
            .push(ch1v)
            .push(ch1i);

        let ch2 = Column::new()
            .push(ch2v)
            .push(ch2i);

        Container::new(Row::new().spacing(20).push(ch1).push(ch2))
            .width(Length::FillPortion(2))
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

mod style {
    use iced::{button, Background, Color, Vector};

    pub enum Button {
        Primary,
        Secondary,
        Destructive,
    }

    impl button::StyleSheet for Button {
        fn active(&self) -> button::Style {
            button::Style {
                background: Some(Background::Color(match self {
                    Button::Primary => Color::from_rgb(0.11, 0.42, 0.87),
                    Button::Secondary => Color::from_rgb(0.5, 0.5, 0.5),
                    Button::Destructive => Color::from_rgb(0.8, 0.2, 0.2),
                })),
                border_radius: 12.0,
                shadow_offset: Vector::new(1.0, 1.0),
                text_color: Color::WHITE,
                ..button::Style::default()
            }
        }
    }
}
