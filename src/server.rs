
extern crate crypto;
use std::net::{TcpListener,TcpStream};
use std::io::{BufRead, BufReader, Read, Write};
use std::{io, thread};
use std::slice;
use std::collections::HashSet;

pub struct Server{
    addr: String,
}

// todo 移動到協議中
const noauth:u8 = 0;


// https://www.ietf.org/rfc/rfc1928.txt
impl Server{

    pub fn new( addr: String)->Server{
        Server{
            addr,
        }
    }

    pub fn run (&self){
        let listener = TcpListener::bind(&self.addr).unwrap();

        for stream in listener.incoming() {
            let stream = stream.unwrap();
            println!("Connection established!");
            thread::spawn(||{
                Server::handle_connection(stream)
            });
        }
    }

    fn handle_connection(mut stream: TcpStream) {
        let mut  buf_reader = BufReader::new(&mut stream);
        let mut buf:Vec<u8> =vec![0;2];

        let res = buf_reader.read_exact(&mut buf);
        if let(Err(e)) = res{
            println!("{}",e);
            return
        }

        println!("buflen {}",buf.len());
        println!("{:?}",buf);
        let  ver = buf.get(0);
        if let(Some(ver_int)) = ver{
            // 不是socks5不支持
            if *ver_int!=5{
                println!("not version 5");
                return
            }
        }else{
            return
        }

        let methods_cnt = buf.get(1);
        let mut methods_cnt_num:u8=0;
        if let(Some(methods_cnt_inner)) = methods_cnt {
            // 必須在1-255
            if *methods_cnt_inner == 0{
                println!("methods num{}",*methods_cnt_inner);
                return
            }
            methods_cnt_num = *methods_cnt_inner
        }else{
            return
        }
        println!("methods cnt {}",&methods_cnt_num);
        buf.resize(methods_cnt_num as usize,0);

        let res = buf_reader.read_exact(&mut buf);
        if let(Err(e)) = res{
            println!("{}",e);
            return
        }

        println!("methods list {:?}",buf);
        let mut mth_set:HashSet<u8> = HashSet::new();
        for elm in buf.iter(){
            mth_set.insert(elm.to_owned());
        }
        println!("methods set len: {}",mth_set.len());
        // 目前仅实现无认证
        if !mth_set.contains(&noauth){
            // 根据协议,需要回复一个0xFF
            let resp:Vec<u8> = vec![0x05,0xff];
            stream.write(&*resp);
            return
        }

    }
}
