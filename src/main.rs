extern crate mio;

use mio::buf::Buf;
use mio::event;
use mio::IoReader;
use mio::IoWriter;
use mio::IoAcceptor;
use mio::net::Socket;

enum Message {
    SayHi,
}

const SERVER: mio::Token = mio::Token(0);

struct Client {
    sock: mio::net::tcp::TcpSocket,
    writeable: bool,
    write_buf: Vec<u8>,
}

impl Client {
    fn new (sock: mio::net::tcp::TcpSocket) -> Client {
        Client {
            sock: sock,
            writeable: false,
            write_buf: Vec::new(),
        }
    }
    fn read (&mut self) -> Result<(), ()> {
        let mut buf = mio::buf::ByteBuf::mut_with_capacity(1024);
        //TODO: find a cleaner way to do this?
        if let Err(_) = self.sock.read(&mut buf) {
            return Err(());
        }
        let buf = buf.flip();
        buf.bytes().iter().map(|x| print!("{:02X} ", *x)).last();
        println!("");
        Ok(())
    }
    fn set_writeable (&mut self) {
        self.writeable = true;
    }
    fn push_write (&mut self, buf: &[u8]) {
        self.write_buf.push_all(buf);
    }
    fn needs_write (&self) -> bool {
        self.write_buf.len() > 0
    }
}

struct Handler {
    server:      mio::net::tcp::TcpAcceptor,
    token_index: usize,
    clients:     Vec<Client>,
}

impl Handler {
    fn new (server: mio::net::tcp::TcpAcceptor) -> Handler {
        Handler {
            server:      server,
            token_index: 1,
            clients:     Vec::new(),
        }
    }
    fn accept (&mut self, eloop: &mut mio::EventLoop<usize, Message>) {
        let client = self.server.accept().unwrap().unwrap();
        let token = mio::Token(self.token_index);
        self.token_index += 1;

        eloop.register(&client, token).ok().expect("accept failed");
        self.clients.push(Client::new(client));
        println!("new client");
    }
    fn read (&mut self, token: usize) -> Result<(), ()> {
        let client = &mut self.clients[token - 1];
        let res = client.read();
        println!("read: {}", token);
        res
    }
    fn maybe_write (&mut self, token: usize) {
        let client = &mut self.clients[token - 1];
        if client.needs_write() {
            //TODO: do
            println!("would write: {}", token);
        } else {
            println!("no write: {}", token);
        }
    }
}

impl mio::Handler<usize, Message> for Handler {
    fn readable (&mut self, eloop: &mut mio::EventLoop<usize, Message>, token: mio::Token, hint: event::ReadHint) {
        match token {
            SERVER => self.accept(eloop),
            mio::Token(x) => {
                if let Err(_) = self.read(x) {
                    // pull this token out of the loop
                    //TODO: free resources associated with this client by messing
                    //      with the client Vec
                    eloop.deregister(&self.clients[x - 1].sock);
                }
            },
        }
    }

    fn writable (&mut self, eloop: &mut mio::EventLoop<usize, Message>, token: mio::Token) {
        println!("writeable");
        match token {
            SERVER => {
                println!("server writeable");
            },
            mio::Token(x) => {
                self.maybe_write(x);
            }
        }
    }
}

fn main() {
    let mut eloop = mio::EventLoop::<usize, Message>::new().unwrap();

    let addr = mio::net::SockAddr::parse("127.0.0.1:59000").unwrap();
    let srv = mio::net::tcp::TcpSocket::v4().unwrap();
    srv.set_reuseaddr(true).unwrap();
    let srv = srv.bind(&addr).unwrap().listen(256us).unwrap();

    eloop.register(&srv, SERVER).unwrap();

    eloop.run(Handler::new(srv)).ok().expect("eloop failed to execute");
}
