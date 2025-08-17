use std::{collections::HashMap, io::{self, BufRead, Write}, net::{TcpListener, TcpStream}, sync::{mpsc::{channel, Receiver, Sender}, Arc, Mutex}, thread};

type Sockets<'a> = Arc<Mutex<HashMap<String, TcpStream>>>;

struct Message {
    id: String,
    msg: String
}

fn main() {
    let sockets: Sockets = Arc::new(Mutex::new(HashMap::new()));
    let listener = TcpListener::bind("localhost:8000").unwrap();

    let (sender, receiver) = channel::<Message>();
    let arc_sender = Arc::new(sender);
    consumer(receiver, sockets.clone());

    loop {
        let (socket, _) = listener.accept().unwrap();
        handle_socket(arc_sender.clone(), socket, sockets.clone());
    }
}

fn handle_socket(sender: Arc<Sender<Message>>, mut socket: TcpStream, sockets: Sockets) {
    thread::spawn(move || {
        let id = rand::random::<i32>().to_string();

        socket.write(b"Please enter your name: ").unwrap();
        socket.flush().unwrap();
        let user_name = read_msg(&socket).unwrap();
        
        {
            let mut sockets = sockets.lock().unwrap();
            sockets.insert(id.clone(), socket.try_clone().unwrap());
        }

        sender.send(Message { id: id.clone(), msg: format!("{} has connected", user_name) }).unwrap();
        loop {
            match read_msg(&socket) {
                Some(msg) => {
                    sender.send(Message { id: id.clone(), msg: format!("{}: {}", user_name, msg) }).unwrap();
                },
                None => {

                    {
                        let mut sockets = sockets.lock().unwrap();
                        sockets.remove(&id);
                    }

                    sender.send(Message { id: id.clone(), msg: format!("{} has been disconnected", user_name) }).unwrap();
                    println!("Connection closed");
                    break
                }
            }
        }
    });
}

fn read_msg(socket: &TcpStream) -> Option<String> {
    let mut buf: Vec<u8> = vec![];
    let mut buf_reader = io::BufReader::new(socket);
    match buf_reader.read_until(b'\n', &mut buf){
        Ok(len) => {
            if len == 0 {
                return None
            }else {
                let buf_read = &buf[..len];
                return Some(String::from_utf8_lossy(buf_read).to_string().trim().to_string());
            }
        },
        Err(_) => {
            return None
        }
    }
}

fn consumer(receiver: Receiver<Message>, sockets: Sockets) {
    thread::spawn(move || {
        loop {
            let val = receiver.recv().unwrap();
            println!("{}", val.msg);
            
            sockets.lock().unwrap().iter().for_each(|(id, mut socket)| {
                if id.to_string() == val.id {
                    return;
                }
                socket.write(val.msg.as_bytes()).unwrap();
                socket.write(b"\n").unwrap();
                socket.flush().unwrap();
            });
        }
    });
}