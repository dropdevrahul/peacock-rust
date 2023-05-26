use std::error::Error;
use std::net::TcpStream;
use std::io::{BufReader,BufRead, Read, Write};
use std::str;

const COMMAND_LENGTH:usize = 11;
const KEY_LENGTH:usize = 64;
const MAX_PAYLOAD_LENGTH_INT:usize = 2048;
const GET_COMMAND:&str = "GET";
const SET_COMMAND:&str = "SET";
const DEL_COMMAND:&str = "DEL";
const GET_TTL_COMMAND:&str = "GET_TTL";
const SET_TTL_COMMAND:&str = "SET_TTL";
const HEADER_SEP:u8 = b'\n';

struct Header {
    len: usize,
}

impl Header {
    fn to_bytes(self) -> Vec<u8> {
        format!("{}\n",self.len.to_string()).into_bytes()
    }
}

fn read_headers( reader: &mut BufReader<&mut TcpStream>) -> Result<Header, String> {
    let mut buff = String::from("");
    let _ = match reader.read_line(&mut buff){
        Ok(n) => n,
        Err(error) => return Err(error.to_string())
    };

    //let s = match str::from_utf8(&mut buff) {
    //    Ok(v) => v,
    //    Err(error) => return Err(error.to_string()),
    //};

    println!("{buff}");
    let (s, _) = buff.split_at(buff.len() - 1);
    println!("{s}");
    let len = s.parse::<usize>().unwrap();
    println!("{s}");
    let header = Header{
        len: len,
    };

    Ok(header)
}

fn read_body( reader: &mut BufReader<&mut TcpStream>, len: usize) -> Result<Vec<u8>, String> {
   let mut buff = vec![0_u8; len];
   let n = match reader.read_exact(&mut buff) {
        Ok(r) => r,
        Err(error) => return Err(error.to_string()),
   };

   Ok(buff)
}

fn read_payload( buff: &Vec<u8>, payload: &mut Vec<u8>) {
    let s = COMMAND_LENGTH + KEY_LENGTH;
    for i in s..buff.len() {
        payload.push(buff[i])
    }
}

fn get_key( buff: &Vec<u8>) -> String {
   String::from_utf8(buff[COMMAND_LENGTH..KEY_LENGTH].to_vec()).unwrap()
}

fn get_cmd( buff: &Vec<u8>) -> String {
   String::from_utf8(buff[..COMMAND_LENGTH].to_vec()).unwrap()
}

pub struct PeacockClient {
    host: String,
    port: String,
}

impl PeacockClient {
    fn connect(&self) -> Result<TcpStream, std::io::Error> {
        let addr = format!("{}:{}", self.host, self.port);
        let res = match TcpStream::connect(addr) {
            Ok(r) => r,
            Err(error) => return Err(error),
        };
        return Ok(res)
    }

    fn pad(&mut self, s:String, len: usize) -> String{
        return format!("{:0width$}", s, width=len);
    }

    fn send(&mut self, mut cmd: String, mut key: String, payload: String) -> Result<Vec<u8>, std::io::Error> {
       cmd = self.pad(cmd, COMMAND_LENGTH);
       key = self.pad(key, KEY_LENGTH);
       let mut res = match self.connect() {
           Ok(stream) => stream,
           Err(error) => return Err(error),
       };

       let mut body = format!("{}{}{}", cmd, key, payload).into_bytes();
       let header = Header{
        len: body.len(),
       };

       let mut cmd_bytes = header.to_bytes();
       cmd_bytes.append(&mut body);

       res.write(&cmd_bytes);
       let mut r = BufReader::new(&mut res);
       let rh = read_headers(&mut r).unwrap();
       let rb= read_body(&mut r, rh.len).unwrap();
       
       return Ok(rb);
    }

    fn get(&mut self, key: String) -> Result<Vec<u8>, std::io::Error> {
        return self.send(String::from("GET"), key, String::from(""));
    }

    fn set(&mut self, key: String, val: String) -> Result<Vec<u8>, std::io::Error> {
        return self.send(String::from("SET"), key, val);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_set() {
        let mut client = PeacockClient{
            host: String::from("0.0.0.0"),
            port: String::from("9999"),
        };

        let val = String::from("abc");
        let res = client.set(String::from("a"), val).unwrap();
        assert_eq!(String::from_utf8(res).unwrap(), String::from("0"))
    }

    #[test]
    fn test_get() {
        let mut client = PeacockClient{
            host: String::from("0.0.0.0"),
            port: String::from("9999"),
        };

        let val = String::from("abc");
        let _ = client.set(String::from("a"), val).unwrap();
        let res = client.get(String::from("a")).unwrap();
        assert_eq!(String::from_utf8(res).unwrap(), String::from("abc"))
    }
}

