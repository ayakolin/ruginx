use std::{
    fs,
    io::{prelude::*,BufReader},
    net::{TcpListener,TcpStream},
};
use ruginx::ThreadPool;
fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    //设定线程池，避免多用户访问时占用完所有资源
    let pool = ThreadPool::new(4);
    for stream in listener.incoming().take(2){
        //incoming返回一个迭代器,元素类型是 Result<TcpStream,std::io::Error>，即可能失败，也可能成功的连接
        let stream = stream.unwrap();
        pool.excute(||{
            handle_connection(stream);
        });
    }
}
fn handle_connection(mut stream: TcpStream){
    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();
    // let http_request:Vec<_> = buf_reader
    // //lines()是迭代器适配器，它把一个按字节读取的流，转换成按行读取的迭代器
    // //即读取到换行符就把这一行变成一个 string
    // .lines()
    // //map 对所有 line 执行一个闭包，若 result 返回 err 则终止进程
    // .map(|result| result.unwrap())
    // //take_while的作用是只要闭包返回 true，就继续取，返回 false，就停止
    // .take_while(|line|!line.is_empty())
    // .collect();
    let (status_line, filename) = if request_line == "GET / HTTP/1.1" {
            ("HTTP/1.1 200 OK", "hello.html")
        } else {
            ("HTTP/1.1 404 NOT FOUND", "404.html")
        };
    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();

    let response =
        format!("{status_line}\r\nContent-Length: {length}\r\n\r\n\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
    }
