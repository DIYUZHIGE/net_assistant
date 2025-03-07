use lazy_static::lazy_static;
use serde::Serialize;
use serial2::SerialPort;
use std::io::{BufReader, Read, Write};
use std::thread::sleep;
use std::time::Duration;
use std::{
    net::TcpStream,
    sync::{Arc, Mutex},
};
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UdpSocket;

// TCP Server

#[derive(Default)]
struct TcpServerForm {
    ip: String,
    port: String,
}
lazy_static! {
    static ref TCP_SERVER_STREAM_READ: tokio::sync::Mutex<Option<tokio::io::ReadHalf<tokio::net::TcpStream>>> =
        tokio::sync::Mutex::new(None);
    static ref TCP_SERVER_STREAM_WRITE: tokio::sync::Mutex<Option<tokio::io::WriteHalf<tokio::net::TcpStream>>> =
        tokio::sync::Mutex::new(None);
    static ref TCP_SERVER_FORM: Arc<Mutex<TcpServerForm>> =
        Arc::new(Mutex::new(TcpServerForm::default()));
    static ref TCP_SERVER_HISTORY: Arc<Mutex<TcpServerHistory>> =
        Arc::new(Mutex::new(TcpServerHistory::default()));
    static ref TCP_SERVER_STATUS: Arc<Mutex<TcpServerStatus>> =
        Arc::new(Mutex::new(TcpServerStatus::default()));
    static ref TCP_SERVER_LISTENING_TASK: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>> =
        Arc::new(Mutex::new(None));
    static ref TCP_SERVER_RECEIVING_TASK: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>> =
        Arc::new(Mutex::new(None));
}

impl TcpServerForm {
    fn set_ip(&mut self, ip: String) {
        self.ip = ip;
    }

    fn set_port(&mut self, port: String) {
        self.port = port;
    }
}

async fn handle_tcp_server_stream() {
    println!("message receiving");
    let mut buf = vec![0u8; 512];
    loop {
        println!("message receiving");
        let length = TCP_SERVER_STREAM_READ
            .lock()
            .await
            .as_mut()
            .unwrap()
            .read(&mut buf)
            .await
            .unwrap();
        if length == 0 {
            break;
        }
        let message = format!(
            "接收: {}\n",
            String::from_utf8_lossy(&buf.as_slice()[..length])
        );
        println!("{}", message);
        TCP_SERVER_HISTORY
            .lock()
            .unwrap()
            .history
            .push_str(&message);
        TCP_SERVER_HISTORY.lock().unwrap().update();
        println!("received one message");
    }
}

#[tauri::command(async)]
async fn tcp_server_establish() {
    TCP_SERVER_LISTENING_TASK
        .lock()
        .unwrap()
        .replace(tokio::spawn(tcp_server_listening()));
    TCP_SERVER_STATUS.lock().unwrap().set("established".into());
    TCP_SERVER_STATUS.lock().unwrap().update();
    println!("established");
}

#[tauri::command(async)]
async fn tcp_server_listening() {
    let ip = TCP_SERVER_FORM.lock().unwrap().ip.clone();
    let port = TCP_SERVER_FORM.lock().unwrap().port.clone();
    let addr = format!("{}:{}", ip, port);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    loop {
        let (socket, _) = listener.accept().await.unwrap();

        let (read_socket, write_socket) = tokio::io::split(socket);

        TCP_SERVER_STREAM_READ.lock().await.replace(read_socket);
        TCP_SERVER_STREAM_WRITE.lock().await.replace(write_socket);
        println!("A CONNECTION!");
        TCP_SERVER_RECEIVING_TASK
            .lock()
            .unwrap()
            .replace(tokio::spawn(handle_tcp_server_stream()));

        println!("NEXT!");
    }
}

#[tauri::command(async)]
async fn tcp_server_destroy() {
    if TCP_SERVER_LISTENING_TASK.lock().unwrap().is_some() {
        TCP_SERVER_LISTENING_TASK
            .lock()
            .unwrap()
            .as_mut()
            .unwrap()
            .abort();
    }
    if TCP_SERVER_RECEIVING_TASK.lock().unwrap().is_some() {
        TCP_SERVER_RECEIVING_TASK
            .lock()
            .unwrap()
            .as_mut()
            .unwrap()
            .abort();
    }

    TCP_SERVER_LISTENING_TASK.clear_poison();
    TCP_SERVER_RECEIVING_TASK.clear_poison();

    TCP_SERVER_STATUS.lock().unwrap().set("destroyed".into());
    TCP_SERVER_STATUS.lock().unwrap().update();
}

#[tauri::command]
async fn update_tcp_server_form(ip: String, port: String) {
    TCP_SERVER_FORM.lock().unwrap().set_ip(ip.clone());
    TCP_SERVER_FORM.lock().unwrap().set_port(port.clone());
    println!("updated address : {}:{}", ip, port);
}

#[tauri::command(async)]
async fn tcp_server_send(message: String) {
    println!("{}", message);
    TCP_SERVER_STREAM_WRITE
        .lock()
        .await
        .as_mut()
        .unwrap()
        .write(message.as_bytes())
        .await
        .unwrap();
    let message = format!("发送: {}\n", message);
    TCP_SERVER_HISTORY
        .lock()
        .unwrap()
        .history
        .push_str(&message);
    TCP_SERVER_HISTORY.lock().unwrap().update();
}

// TCP Client

#[derive(Default)]
struct TcpClient {
    ip: String,
    port: String,
}

#[derive(Serialize, Default)]
struct TcpServerStatus {
    status: String,
}

impl TcpServerStatus {
    fn set(&mut self, status: String) {
        self.status = status;
    }

    fn update(&self) {
        let app_handle = get_apphandle();
        app_handle.emit("update_tcp_server_status", self).unwrap();
    }
}

#[derive(Serialize, Default)]
struct TcpServerHistory {
    history: String,
}

impl TcpServerHistory {
    fn update(&self) {
        let app_handle = get_apphandle();
        app_handle.emit("update_tcp_server_history", &self).unwrap();
    }
}
#[derive(Serialize)]
struct TcpClientHistory {
    history: String,
}

impl TcpClient {
    fn set_ip(&mut self, ip: String) {
        self.ip = ip;
    }
    fn set_port(&mut self, port: String) {
        self.port = port;
    }
    fn connect(&mut self) {
        let address = format!("{}:{}", self.ip, self.port);
        // TODO 错误处理
        let open_stream_result = TcpStream::connect(address);
        match open_stream_result {
            Ok(mut stream) => {
                stream.set_nodelay(true).unwrap();
                tokio::spawn(async move {
                    handle_tcp_client_stream(&mut stream);
                });
            }
            Err(e) => {
                println!("Failed to open stream: {:?}", e);
            }
        }
    }
}

fn handle_tcp_client_stream(stream: &mut TcpStream) {
    TCP_CLIENT_STREAM
        .lock()
        .unwrap()
        .replace(stream.try_clone().unwrap());
    println!("handling stream");
    let mut reader = BufReader::new(stream);
    let mut buffer = [0; 512];

    loop {
        sleep(Duration::from_millis(100));
        println!("waiting for message");
        match reader.read(&mut buffer) {
            Ok(0) => {
                println!("Client disconnected");
                break;
            }

            Ok(n) => {
                let message = String::from_utf8_lossy(&buffer[..n]);
                println!("message received: {}", message);
                let message = format!("接收: {}\n", message);
                TCP_CLIENT_HISTORY.lock().unwrap().push_str(&message);
                let tcp_client_history = TcpClientHistory {
                    history: TCP_CLIENT_HISTORY.lock().unwrap().clone(),
                };
                let app_handle = get_apphandle();
                update_tcp_client_history(&app_handle, &tcp_client_history);
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                println!("Error reading from stream: {:?}", e);
                break;
            }
        }
    }
}

#[derive(Serialize, Clone)]
struct TcpClientStatus {
    status: String,
}

lazy_static! {
    static ref TCP_CLIENT: Arc<Mutex<TcpClient>> = Arc::new(Mutex::new(TcpClient::default()));
    static ref TCP_CLIENT_STREAM: Arc<Mutex<Option<TcpStream>>> = Arc::new(Mutex::new(None));
}

lazy_static! {
    static ref TCP_CLIENT_HISTORY: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
}

lazy_static! {
    static ref APP_HANDLE: Arc<Mutex<Option<AppHandle>>> = Arc::new(Mutex::new(None));
}

#[tauri::command(async)]
async fn update_tcp_client_form(ip: String, port: String) {
    println!("{}:{}", ip, port);
    TCP_CLIENT.lock().unwrap().set_ip(ip);
    TCP_CLIENT.lock().unwrap().set_port(port);
}

#[tauri::command(async)]
async fn tcp_client_connect() {
    println!("connecting");
    TCP_CLIENT_STREAM.clear_poison();
    tokio::spawn(async move {
        TCP_CLIENT.lock().unwrap().connect();
    });
    let tcp_status = TcpClientStatus {
        status: "connected".to_string(),
    };
    let app_handle = get_apphandle();
    update_tcp_client_status(&app_handle, &tcp_status);
}

#[tauri::command(async)]
async fn tcp_client_disconnect() {
    TCP_CLIENT_STREAM.lock().unwrap().take();
    let tcp_status = TcpClientStatus {
        status: "disconnected".to_string(),
    };
    let app_handle = get_apphandle();
    update_tcp_client_status(&app_handle, &tcp_status);
}

fn update_tcp_client_status(app: &AppHandle, status: &TcpClientStatus) {
    app.emit("update_tcp_client_status", status).unwrap();
}

#[tauri::command(async)]
async fn tcp_client_send(message: String) {
    TCP_CLIENT_STREAM.clear_poison();
    TCP_CLIENT_STREAM
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .write(message.as_bytes())
        .unwrap();
    TCP_CLIENT_STREAM
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .flush()
        .unwrap();

    let message = format!("发送: {}\n", message);
    TCP_CLIENT_HISTORY.lock().unwrap().push_str(&message);
    let tcp_client_history = TcpClientHistory {
        history: TCP_CLIENT_HISTORY.lock().unwrap().clone(),
    };
    let app_handle = get_apphandle();
    update_tcp_client_history(&app_handle, &tcp_client_history);
}

fn update_tcp_client_history(app: &AppHandle, history: &TcpClientHistory) {
    app.emit("update_tcp_client_history", history).unwrap();
}

fn get_apphandle() -> AppHandle {
    let app_handle = APP_HANDLE.clone();
    let mut app_handle = app_handle.lock().unwrap();
    let app_handle = app_handle.as_mut().unwrap();
    app_handle.clone()
}

// UDP

struct UdpClient {
    ip: String,
    port: String,
    to_ip: String,
    to_port: String,
}

#[derive(Serialize, Default)]
struct UdpClientHistory {
    history: String,
}

impl UdpClientHistory {
    fn update(&self) {
        let app_handle = get_apphandle();
        app_handle.emit("update_udp_history", self).unwrap();
    }

    fn append(&mut self, message: &str) {
        self.history.push_str(message);
    }
}

#[derive(Serialize, Default)]
struct UdpClientStatus {
    status: String,
}

impl UdpClientStatus {
    fn update(&self) {
        let app_handle = get_apphandle();
        app_handle.emit("update_udp_status", self).unwrap();
    }

    fn set_status(&mut self, status: &str) {
        self.status = status.to_string();
    }
}

lazy_static! {
    static ref UDP_CLIENT: Arc<tokio::sync::RwLock<UdpClient>> =
        Arc::new(tokio::sync::RwLock::new(UdpClient {
            ip: String::new(),
            port: String::new(),
            to_ip: String::new(),
            to_port: String::new(),
        }));
    static ref UDP_CLIENT_HISTORY: Arc<Mutex<UdpClientHistory>> =
        Arc::new(Mutex::new(UdpClientHistory::default()));
    static ref UDP_SOCKET: Arc<tokio::sync::RwLock<Option<UdpSocket>>> =
        Arc::new(tokio::sync::RwLock::new(None));
    static ref UDP_CLIENT_STATUS: Arc<Mutex<UdpClientStatus>> =
        Arc::new(Mutex::new(UdpClientStatus::default()));
    static ref UDP_TASK: Arc<tokio::sync::Mutex<Option<tokio::task::JoinHandle<()>>>> =
        Arc::new(tokio::sync::Mutex::new(None));
    static ref UDP_RECEIVE_TASK: Arc<tokio::sync::Mutex<Option<tokio::task::JoinHandle<()>>>> =
        Arc::new(tokio::sync::Mutex::new(None));
}

impl UdpClient {
    fn set_ip(&mut self, ip: String) {
        self.ip = ip;
    }
    fn set_port(&mut self, port: String) {
        self.port = port;
    }
    fn set_to_ip(&mut self, ip: String) {
        self.to_ip = ip;
    }
    fn set_to_port(&mut self, port: String) {
        self.to_port = port;
    }
    async fn connect(&self) {
        let address = format!("{}:{}", self.ip, self.port);
        // TODO 错误处理
        let udp_socket_result = UdpSocket::bind(address).await;
        match udp_socket_result {
            Ok(udp_socket) => {
                let udp_receive_task = tokio::spawn(async move {
                    udp_receive(udp_socket).await;
                });
                UDP_RECEIVE_TASK.lock().await.replace(udp_receive_task);
            }
            Err(e) => {
                println!("Failed to open stream: {:?}", e);
            }
        }
    }
}

#[tauri::command(async)]
async fn update_udp_form(ip: String, port: String, to_ip: String, to_port: String) {
    println!("{}:{}", ip, port);
    UDP_CLIENT.write().await.set_ip(ip);
    UDP_CLIENT.write().await.set_port(port);
    UDP_CLIENT.write().await.set_to_ip(to_ip);
    UDP_CLIENT.write().await.set_to_port(to_port);
}

#[tauri::command(async)]
async fn udp_bind() {
    println!("binding");
    let bind_task = tokio::spawn(async move {
        UDP_CLIENT.read().await.connect().await;
    });
    UDP_TASK.lock().await.replace(bind_task);
    println!("binded");
    UDP_CLIENT_STATUS.lock().unwrap().set_status("binded");
    UDP_CLIENT_STATUS.lock().unwrap().update();
}

#[tauri::command(async)]
async fn udp_unbind() {
    println!("unbinding");
    UDP_RECEIVE_TASK.lock().await.take().unwrap().abort();
    UDP_TASK.lock().await.take().unwrap().abort();
    *UDP_SOCKET.write().await = None;
    println!("unbinded");
    UDP_CLIENT_STATUS.lock().unwrap().set_status("unbinded");
    UDP_CLIENT_STATUS.lock().unwrap().update();
}

#[tauri::command(async)]
async fn udp_send(message: String) {
    println!("sending: {}", message);
    let to_address = format!(
        "{}:{}",
        UDP_CLIENT.read().await.to_ip,
        UDP_CLIENT.read().await.to_port
    );
    println!("sending to {}", to_address);

    UDP_SOCKET
        .read()
        .await
        .as_ref()
        .unwrap()
        .send_to(message.as_bytes(), to_address)
        .await
        .unwrap();
    let message = format!("发送: {}\n", message);
    UDP_CLIENT_HISTORY.lock().unwrap().append(&message);
    UDP_CLIENT_HISTORY.lock().unwrap().update();
    println!("sented: {}", message);
}

async fn udp_receive(udp_socket: UdpSocket) {
    let mut buf = [0u8; 2048];
    *UDP_SOCKET.write().await = Some(udp_socket);
    loop {
        println!("udp waiting for message");
        let (length, _) = UDP_SOCKET
            .read()
            .await
            .as_ref()
            .unwrap()
            .recv_from(&mut buf)
            .await
            .unwrap();
        let message = format!("接收: {}\n", String::from_utf8_lossy(&buf[..length]));
        println!("{}", message);
        UDP_CLIENT_HISTORY.lock().unwrap().append(&message);
        UDP_CLIENT_HISTORY.lock().unwrap().update();
    }
}

// SERIALPORT
#[derive(Debug)]
struct SerialPortForm {
    port: String,
    baud_rate: u32,
    data_bits: u32,
    stop_bits: u32,
    parity: String,
    flow_control: String,
}

impl SerialPortForm {
    fn set_port(&mut self, port: String) {
        self.port = port;
    }

    fn set_baud_rate(&mut self, baud_rate: u32) {
        self.baud_rate = baud_rate;
    }

    fn set_data_bits(&mut self, data_bits: u32) {
        self.data_bits = data_bits;
    }

    fn set_stop_bits(&mut self, stop_bits: u32) {
        self.stop_bits = stop_bits;
    }

    fn set_parity(&mut self, parity: String) {
        self.parity = parity;
    }
    fn set_flow_control(&mut self, flow_control: String) {
        self.flow_control = flow_control;
    }
}

#[derive(Serialize, Default)]
struct SerialPortStatus {
    status: String,
}

impl SerialPortStatus {
    fn set_status(&mut self, status: String) {
        self.status = status;
    }

    fn update(&self) {
        println!("serial port status: {}", self.status);
        let app_handle = get_apphandle();
        app_handle.emit("update_serial_port_status", self).unwrap();
    }
}

#[derive(Serialize, Default)]
struct SerialPortHistory {
    history: String,
}

impl SerialPortHistory {
    fn append(&mut self, message: &str) {
        self.history.push_str(message);
    }

    fn update(&self) {
        println!("serial port history: {}", self.history);
        let app_handle = get_apphandle();
        app_handle.emit("update_serial_port_history", self).unwrap();
    }
}

#[derive(Serialize, Default)]
struct SerialPortNames {
    names: Vec<String>,
}

lazy_static! {
    static ref SERIAL_PORT_FORM: Mutex<Option<SerialPortForm>> = Mutex::new(Some(SerialPortForm {
        port: String::new(),
        baud_rate: 115200,
        data_bits: 8,
        stop_bits: 1,
        parity: String::new(),
        flow_control: String::new(),
    }));
    static ref SERIAL_PORT: tokio::sync::RwLock<Option<SerialPort>> =
        tokio::sync::RwLock::new(None);
    static ref SERIAL_PORT_HISTORY: Mutex<SerialPortHistory> = Mutex::new(SerialPortHistory {
        history: String::new(),
    });
    static ref SERIAL_OPEN_TASK: Mutex<Option<tokio::task::JoinHandle<()>>> = Mutex::new(None);
    static ref SERIAL_PORT_STATUS: Arc<Mutex<SerialPortStatus>> =
        Arc::new(Mutex::new(SerialPortStatus::default()));
    static ref SERIAL_PORT_RECEIVE_TASK: Mutex<Option<tokio::task::JoinHandle<()>>> =
        Mutex::new(None);
}

#[tauri::command(async)]
async fn serial_ports_refresh() {
    let port_names = serialport::available_ports().unwrap();
    let port_names: Vec<String> = port_names.iter().map(|p| p.port_name.clone()).collect();
    let serial_port_names = SerialPortNames { names: port_names };
    update_serial_port_names(&serial_port_names);
    println!("port names {:?}", serial_port_names.names);
}

fn update_serial_port_names(names: &SerialPortNames) {
    let app_handle = get_apphandle();
    app_handle.emit("update_serial_port_names", names).unwrap();
}

#[tauri::command(async)]
async fn update_serial_port_parameters(
    port: String,
    baud_rate: String,
    data_bits: usize,
    stop_bits: usize,
    parity: String,
    flow_control: String,
) {
    println!(
        "updated serial port parameters : {:?}",
        SERIAL_PORT_FORM.lock().unwrap().as_mut().unwrap()
    );
    SERIAL_PORT_FORM
        .lock()
        .unwrap()
        .as_mut()
        .unwrap()
        .set_port(port);
    SERIAL_PORT_FORM
        .lock()
        .unwrap()
        .as_mut()
        .unwrap()
        .set_baud_rate(u32::from_str_radix(&baud_rate, 10).unwrap());
    SERIAL_PORT_FORM
        .lock()
        .unwrap()
        .as_mut()
        .unwrap()
        .set_data_bits(data_bits as u32);
    SERIAL_PORT_FORM
        .lock()
        .unwrap()
        .as_mut()
        .unwrap()
        .set_stop_bits(stop_bits as u32);
    SERIAL_PORT_FORM
        .lock()
        .unwrap()
        .as_mut()
        .unwrap()
        .set_parity(parity);
    SERIAL_PORT_FORM
        .lock()
        .unwrap()
        .as_mut()
        .unwrap()
        .set_flow_control(flow_control);
    println!(
        "updated serial port parameters : {:?}",
        SERIAL_PORT_FORM.lock().unwrap().as_mut().unwrap()
    );
}

#[tauri::command(async)]
async fn serial_port_open() {
    println!("opening serial port");
    let port_name = SERIAL_PORT_FORM
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .port
        .clone();
    let baud_rate = SERIAL_PORT_FORM.lock().unwrap().as_ref().unwrap().baud_rate;
    let data_bits = match SERIAL_PORT_FORM.lock().unwrap().as_ref().unwrap().data_bits {
        8 => serial2::CharSize::Bits8,
        7 => serial2::CharSize::Bits7,
        6 => serial2::CharSize::Bits6,
        5 => serial2::CharSize::Bits5,
        _ => serial2::CharSize::Bits8,
    };
    let stop_bits = match SERIAL_PORT_FORM.lock().unwrap().as_ref().unwrap().stop_bits {
        1 => serial2::StopBits::One,
        2 => serial2::StopBits::Two,
        _ => serial2::StopBits::One,
    };

    println!("port name: {}", port_name);
    println!("baud rate: {}", baud_rate);

    let setting_fn = |mut s: serial2::Settings| -> std::io::Result<serial2::Settings> {
        s.set_baud_rate(baud_rate).unwrap();
        s.set_char_size(data_bits);
        s.set_stop_bits(stop_bits);
        Ok(s)
    };
    *SERIAL_PORT.write().await = Some(SerialPort::open(&port_name, setting_fn).unwrap());
    SERIAL_PORT_STATUS
        .lock()
        .unwrap()
        .set_status("opened".into());

    SERIAL_PORT_RECEIVE_TASK
        .lock()
        .unwrap()
        .replace(tokio::spawn(serial_port_receive()));
    SERIAL_PORT_STATUS.lock().unwrap().update();
}

async fn serial_port_receive() {
    let mut buf = [0u8; 256];
    loop {
        match SERIAL_PORT.read().await.as_ref().unwrap().read(&mut buf) {
            Ok(length) => {
                println!("received: {}", String::from_utf8_lossy(&buf[..length]));
                let message = format!("接收: {}\n", String::from_utf8_lossy(&buf[..length]));
                SERIAL_PORT_HISTORY.lock().unwrap().append(&message);
                SERIAL_PORT_HISTORY.lock().unwrap().update();
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

#[tauri::command(async)]
async fn serial_port_close() {
    SERIAL_PORT_RECEIVE_TASK
        .lock()
        .unwrap()
        .take()
        .unwrap()
        .abort();
    SERIAL_PORT_RECEIVE_TASK.clear_poison();
    drop(SERIAL_PORT.write().await);
    *SERIAL_PORT.write().await = None;
    SERIAL_PORT_STATUS
        .lock()
        .unwrap()
        .set_status("closed".into());
    SERIAL_PORT_STATUS.lock().unwrap().update();
}

#[tauri::command(async)]
async fn serial_port_write(message: String) {
    SERIAL_PORT
        .read()
        .await
        .as_ref()
        .unwrap()
        .write(message.as_bytes())
        .unwrap();
    let message = format!("发送: {}\n", message);
    SERIAL_PORT_HISTORY.lock().unwrap().append(&message);
    SERIAL_PORT_HISTORY.lock().unwrap().update();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .setup(|app| {
            APP_HANDLE.lock().unwrap().replace(app.handle().clone());
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            tcp_client_connect,
            tcp_client_disconnect,
            update_tcp_client_form,
            tcp_client_send,
            tcp_server_establish,
            tcp_server_destroy,
            tcp_server_send,
            update_tcp_server_form,
            update_udp_form,
            udp_bind,
            udp_unbind,
            udp_send,
            serial_ports_refresh,
            update_serial_port_parameters,
            serial_port_open,
            serial_port_close,
            serial_port_write
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
