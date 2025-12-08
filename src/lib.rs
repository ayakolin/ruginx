use std::{
    sync::{Arc, Mutex, mpsc::{self, Receiver}}, thread,};

pub struct ThreadPool{
    workers:Vec<Worker>,
    //sender是消息通道，用于作为任务队列
    sender: Option<mpsc::Sender<Job>>,
}
//FnOnce 表示闭包可以被调用一次
//Send 表示闭包可以在线程间传递
//'static 表示闭包拥有 'static 生命周期，即它不包含任何非 'static 引用
type Job = Box<dyn FnOnce() + Send + 'static>;
//为 ThreadPool 实现 new 函数
impl  ThreadPool{
    pub fn new(size:usize) -> ThreadPool{
        //确定线程池大于 0
        assert!(size > 0);
        //线程池拥有发送端，而每个 worker 拥有接收端
        //worker 之间共享接收端，所以使用 Arc 和 Mutex 来实现共享和互斥
        let (sender, receiver) = mpsc::channel();
        //Arc 用于多所有权，Mutex 用于互斥访问
        //Arc means Atomic Reference Counted
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
        ThreadPool{
            workers,
            sender:Some(sender)}
    }
    pub fn excute<F>(&self,f:F)
    where 
        F: FnOnce()+Send+'static,
        {
            let job = Box::new(f);
            self.sender.as_ref().unwrap().send(job).unwrap();
        }
}
impl Drop for ThreadPool{
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in &mut self.workers{
            println!("Shutting down workers {}",worker.id);
            //使用模式匹配和take拿走worker.thread的所有权，避开不可变引用的限制。
            //如果worker.thread已经是none，什么都不会发生，符合预期
            //如果包含一个线程，那就拿走所有权
            if let Some(thread) = worker.thread.take(){
                thread.join().unwrap();
            }
        }
    }
}
//对 Worker 的实现
//用于存储进程
struct Worker{
    //独立的 worker id，其实也可以叫线程 id
    id: usize,
    thread: Option<thread::JoinHandle<()>>
}
impl  Worker {
    fn new(id: usize,receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker{
        //这里要用move的原因是，在异步任务和线程或者是闭包被返回、存储，下次再调用的情况下，
        //闭包/任务/线程的执行时间，不再受当前函数作用域控制，有可能在当前函数结束之后才继续存在或执行。
        //如果闭包只是借用了外部变量，很可能导致外部变量在当前函数结束时被销毁，但闭包还使用着它的引用
        //这就是典型的悬垂引用
        //为了从源头上禁止这个情况，rust只允许两种方式：
        //闭包在当前作用域内用完->可以借用
        //闭包可能逃出当前作用域->必须使用move
        //遵循这条规则：任何引用的生命周期都不能超过它所引用的数据的生命周期
        let thread = thread::spawn(move||loop{
            //lock用于获取锁
            //假如当前持有锁的线程panic了，那么等待锁的线程就会获取一个错误
            //recv用于阻塞消息通道，如果没有任何任务，那么当前的调用线程将一直等待，直到接收到新任务
            let message = receiver.lock().unwrap().recv();
            match message{
                Ok(job) =>{
                    println!("Worker {id} got a job; executing.");
                    job();
            }
                Err(_) => {
                    println!("Worker {id} disconnected; shutting down.");
                    break;
                }
        }});
        Worker{id,thread:Some(thread)}
    }
}