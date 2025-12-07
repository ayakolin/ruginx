use std::{
    sync::{Arc, Mutex, mpsc::{self, Receiver}}, thread,};

pub struct ThreadPool{
    workers:Vec<Worker>,
    //sender是消息通道，用于作为任务队列
    sender: mpsc::Sender<Job>,
}
struct Job{
    //存储任务的闭包
    job: Box<dyn FnOnce()+Send+'static>,
}
//为 ThreadPool 实现 new 函数
impl  ThreadPool{
    pub fn new(size:usize) -> ThreadPool{
        //确定线程池大于 0
        assert!(size > 0);
        //线程池拥有发送端，而每个 worker 拥有接收端
        //worker 之间共享接收端，所以使用 Arc 和 Mutex 来实现共享和互斥
        let (sender, receiver) = mpsc::channel();
        //Arc 用于多所有权，Mutex 用于互斥访问
        let receiver = Arc::new(Mutex::new(receiver));
        //workers存储线程
        let mut workers = Vec::with_capacity(size);
        for id in 0..size{
            //在此处创建一些进程，并存储它们
            //并不能使用 thread::spawn，因为它会立即执行传入的任务
            //我们的期望是，创建线程和执行分离，所以不可能使用 thread::spawn
            //所以创建了一个 Worker 类，它要负责存储线程，在时机合适时创建线程，并传递任务
            workers.push(Worker::new(id,Arc::clone(&receiver)));
        }
        ThreadPool{workers,sender}
    }
    pub fn excute<F>(&self,f:F)
    //待实现
    where 
        F: FnOnce()+Send+'static,
        {

        }
}
//对 Worker 的实现
//用于存储进程
struct Worker{
    //独立的 worker id，其实也可以叫线程 id
    id: usize,
    thread: thread::JoinHandle<()>
}
impl  Worker {
    fn new(id: usize,reciver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker{
        let thread = thread::spawn(||{});
        Worker{id,thread}
    }
}