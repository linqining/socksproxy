
extern crate crypto;


use std::net::{TcpListener, TcpStream, Shutdown,SocketAddr};
use std::io::{BufRead, BufReader, Read, Write};
use std::{io, thread};
use std::slice;
use std::collections::HashSet;
use std::fmt::Error;

pub struct Server{
    addr: String,
}

// todo 移動到協議模块
const AUTH_NO_AUTH:u8 = 0;
const CMD_CONNECT:u8 = 0x01;

const ADDR_TYPE_IPV4:u8 = 0x01;
const ADDR_TYPE_DOMAIN:u8= 0x03;
const ADDR_TYPE_IPV6:u8 = 0x04;


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
            thread::spawn(move ||{
                Server::handle_connection(stream)
            });
        }
    }

    fn handle_connection(mut  stream: TcpStream) {
        let mut user_resp_stream = stream.try_clone().expect("clone failed");
        let mut  buf_reader = BufReader::new(&mut stream);
        let mut buf:Vec<u8> =vec![0;2];

        let res = stream.read_exact(&mut buf);
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

        let res = stream.read_exact(&mut buf);
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
        if !mth_set.contains(&AUTH_NO_AUTH){
            // 根据协议,需要回复一个0xFF
            let resp:Vec<u8> = vec![0x05,0xff];
            stream.write_all(&*resp);
            // stream.flush().unwrap();
            // stream.shutdown(Shutdown::Both);
            return
        }else{

            if let Err(err) = user_resp_stream.write_all(&*vec![0x05, 0x00]){
                println!("response method failed: {}",err);
                return
            }
        }



        let mut req_buffer = [0u8; 1024];
        let n = stream.read(&mut req_buffer).unwrap();
        if n <=7 {
            return
        }

        let dest_addr = &req_buffer[4..(n-2)];
        let dest_port = &req_buffer[(&n-2)..n];
        println!("req buffer{:?}",req_buffer);
        if req_buffer[1] != CMD_CONNECT{
            // rust 删除值是,tcp会自动关闭
            // 怎么实现的?
            return
        }
        if req_buffer[3]!= ADDR_TYPE_IPV4{
            return
        }
        // match req_buffer[3]{
        //     ADDR_TYPE_IPV4=>{
        //
        //     },
        //     ADDR_TYPE_DOMAIN=>{
        //
        //     },
        //     ADDR_TYPE_IPV6=>(),
        //     _=>return (),
        // }

        // let tmp_addr:[u8;4] = core::array::from_fn(|i| dest_addr[i].to_owned());
        let tmp_addr:[u8;4]=[39,156,66,10];
        let tmp_addr:[u8;4]=[59,82,122,115];
        let mut tmp_port:u16 = (dest_port[1].to_owned() as u16);
        if dest_port[0]>0{
            tmp_port += 8<<(dest_port[0].to_owned() as u16)
        }
        // println!("get addr {:?}",tmp_addr);
        let addrs = [
            SocketAddr::from((tmp_addr, tmp_port)),
        ];
        if let Ok(mut dest_read_stream) = TcpStream::connect(&addrs[..]) {
            let mut dest_write_stream = dest_read_stream.try_clone().expect("clone dst conn failed");
            let rply:Vec<u8> =vec![0x05, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
            user_resp_stream.write_all(&*rply);
            println!("Connected to the server!");


            thread::spawn(move ||{
                let mut req_buffer = [0u8; 1024];
                loop {
                    let n = stream.read(&mut req_buffer).unwrap();
                    if n ==0{
                        break;
                    }
                    dest_write_stream.write(&req_buffer[..n]);
                }
            });



            let mut resp_buffer = [0u8; 1024];
            loop {
                let n = dest_read_stream.read(&mut resp_buffer).unwrap();
                if n==0{
                    break;
                }
                println!("resp {:?}",resp_buffer.as_slice());
                user_resp_stream.write(&resp_buffer[..n]).unwrap();
            }
        } else {
            println!("Couldn't connect to server...");
        }
        println!("connection close")
    }
}
