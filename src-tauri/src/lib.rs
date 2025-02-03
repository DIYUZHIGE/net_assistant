use lazy_static::lazy_static;
use serde::Serialize;
use std::io::{BufReader, Read, Write};
use std::{
    cell::RefCell,
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};
use tauri::{AppHandle, Emitter};

// TCP Server
#[derive(Default)]
struct TcpServer {
    ip: String,
    port: String,
    listener: Mutex<Option<TcpListener>>,

    history: RefCell<String>,
}

lazy_static! {
    static ref TCP_SERVER_STREAM: Mutex<Option<TcpStream>> = Mutex::new(None);
    static ref TCP_SERVER: Arc<Mutex<TcpServer>> = Arc::new(Mutex::new(TcpServer::default()));
}

impl TcpServer {
    fn set_ip(&mut self, ip: String) {
        self.ip = ip;
    }

    fn set_port(&mut self, port: String) {
        self.port = port;
    }

    fn listen(&mut self) {
        let address = format!("{}:{}", self.ip, self.port);
        println!(" address : {}", address);
        let listen_result = TcpListener::bind(address);
        match listen_result {
            Ok(listener) => {
                self.listener.lock().unwrap().replace(listener);
                self.wait_for_stream();
            }
            Err(e) => {
                println!("Failed to open stream: {:?}", e);
            }
        }
    }

    fn wait_for_stream(&mut self) {
        println!("waiting for stream");

        let tcp_server_status = TcpServerStatus {
            status: "listening".to_string(),
        };
        let app_handle = get_apphandle();
        update_tcp_server_status(&app_handle, &tcp_server_status);
        let (stream, _) = self
            .listener
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .accept()
            .unwrap();
        TCP_SERVER_STREAM.lock().unwrap().replace(stream);
        thread::spawn(move || {
            handle_tcp_server_stream(&mut TCP_SERVER_STREAM.lock().unwrap().as_mut().unwrap());
        });
    }

    fn destroy(&mut self) {
        self.listener.lock().unwrap().take();
    }
}



fn handle_tcp_server_stream(stream: &TcpStream) {
    println!("handling stream");
    let mut reader = BufReader::new(stream);
    let mut buffer = [0; 512];

    loop {
        println!("waiting for message");
        match reader.read(&mut buffer) {
            Ok(0) => {
                println!("Client disconnected");
                break;
            }
            Ok(n) => {
                let message = String::from_utf8_lossy(&buffer[..n]);
                println!("message received: {}", message);
                let message = format!("received: {}\n", message);
                TCP_SERVER
                    .lock()
                    .unwrap()
                    .history
                    .borrow_mut()
                    .push_str(&message);
                let tcp_server_history = TcpServerHistory {
                    history: TCP_SERVER.lock().unwrap().history.borrow().clone(),
                };
                let app_handle = get_apphandle();
                update_tcp_server_history(&app_handle, &tcp_server_history);
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

#[tauri::command(async)]
async fn tcp_server_establish() {
    println!("establishing");
    TCP_SERVER.lock().unwrap().listen();
    println!("listening");

    let tcp_server_status = TcpServerStatus {
        status: "established".to_string(),
    };
    let app_handle = get_apphandle();
    update_tcp_server_status(&app_handle, &tcp_server_status);
}

#[tauri::command(async)]
async fn tcp_server_destroy() {
    println!("destroying"); 
    TCP_SERVER.lock().unwrap().destroy();
    println!("destroyed");
    let tcp_server_status = TcpServerStatus {
        status: "destroyed".to_string(),
    };
    let app_handle = get_apphandle();
    update_tcp_server_status(&app_handle, &tcp_server_status);
}

#[tauri::command]
async fn update_tcp_server_form(ip: String, port: String) {
    TCP_SERVER.lock().unwrap().set_ip(ip.clone());
    TCP_SERVER.lock().unwrap().set_port(port.clone());
    println!("updated address : {}:{}", ip, port);
}

fn update_tcp_server_history(app: &AppHandle, history: &TcpServerHistory) {
    app.emit("update_tcp_server_history", history).unwrap();
}

// TCP Client

#[derive(Default)]
struct TcpClient {
    ip: String,
    port: String,
    stream: Mutex<Option<TcpStream>>,
    history: RefCell<String>,
}

#[derive(Serialize)]
struct TcpServerStatus {
    status: String,
}
#[derive(Serialize)]
struct TcpServerHistory {
    history: String,
}
#[derive(Serialize)]
struct TcpClientHistory{
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
            Ok(stream) => {
                self.stream.lock().unwrap().replace(stream);
                thread::spawn(move || {
                    // handle_tcp_client_stream(&mut TCP_CLIENT.lock().unwrap().stream.lock().unwrap().as_mut().unwrap());
                });
            }
            Err(e) => {
                println!("Failed to open stream: {:?}", e);
            }
        }
    }

    fn disconnect(&mut self) {
        self.stream.lock().unwrap().take();
    }
    fn send(&mut self, message: String) {
        let mut stream = self.stream.lock().unwrap();
        let stream = stream.as_mut().unwrap();
        stream.write(message.as_bytes()).unwrap();
    }
}

fn handle_tcp_client_stream(stream: &mut TcpStream) {
    println!("handling stream");
    let mut reader = BufReader::new(stream);
    let mut buffer = [0; 512];

    loop {
        println!("waiting for message");
        match reader.read(&mut buffer) {
            Ok(0) => {
                println!("Client disconnected");
                break;
            }
            Ok(n) => {
                let message = String::from_utf8_lossy(&buffer[..n]);
                println!("message received: {}", message);
                let message = format!("received: {}\n", message);
                TCP_CLIENT
                    .lock()
                    .unwrap()
                    .history
                    .borrow_mut()
                    .push_str(&message);
                let tcp_client_history = TcpClientHistory {
                    history: TCP_CLIENT.lock().unwrap().history.borrow().clone(),
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
    TCP_CLIENT.lock().unwrap().disconnect();
    TCP_CLIENT.lock().unwrap().connect();

    if !TCP_CLIENT.lock().unwrap().stream.lock().unwrap().is_none() {
        let tcp_status = TcpClientStatus {
            status: "connected".to_string(),
        };
        let app_handle = get_apphandle();
        update_tcp_client_status(&app_handle, &tcp_status);
    }
}

#[tauri::command(async)]
async fn tcp_client_disconnect() {
    TCP_CLIENT.lock().unwrap().disconnect();
    let tcp_status = TcpClientStatus {
        status: "disconnected".to_string(),
    };
    let app_handle = get_apphandle();
    update_tcp_client_status(&app_handle, &tcp_status);
}

fn update_tcp_client_status(app: &AppHandle, status: &TcpClientStatus) {
    app.emit("update_tcp_client_status", status).unwrap();
}

fn update_tcp_server_status(app: &AppHandle, status: &TcpServerStatus) {
    app.emit("update_tcp_server_status", status).unwrap();
}

#[tauri::command(async)]
async fn tcp_client_send(message: String) {
    TCP_CLIENT.lock().unwrap().send(message.clone());

    let message = format!("sent: {}\n", message);
    TCP_CLIENT.lock().unwrap().history.borrow_mut().push_str(&message);
    let tcp_client_history = TcpClientHistory {
        history: TCP_CLIENT.lock().unwrap().history.borrow().to_string(),
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
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
            update_tcp_server_form,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
