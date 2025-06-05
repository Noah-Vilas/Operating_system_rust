
use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use core::{pin::Pin, task::{Poll, Context}};
use futures_util::stream::Stream;
use futures_util::StreamExt;
use futures_util::task::AtomicWaker;

use crate::println;

use spin::Mutex;
use alloc::string::String;
use alloc::sync::Arc;

////////////////

//  COMMAND STREAM TO 

////////////////

static WAKER: AtomicWaker = AtomicWaker::new();
pub static COMMAND_CHANNEL: OnceCell<ArrayQueue<String>> = OnceCell::uninit();

pub struct CommandStream {
    _private: (),
}

impl CommandStream {
    pub fn new() -> Self {
        COMMAND_CHANNEL.try_init_once(|| ArrayQueue::new(10)).unwrap();
        CommandStream { _private: () }
    }
}

impl Stream for CommandStream {
    type Item = String;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<String>> {
        let queue = COMMAND_CHANNEL.try_get().unwrap();

        if let Some(cmd) = queue.pop() {
            return Poll::Ready(Some(cmd));
        }

        WAKER.register(&cx.waker());

        match queue.pop() {
            Some(cmd) => {
                WAKER.take();
                Poll::Ready(Some(cmd))
            },
            None => Poll::Pending,
        }
    }
}

pub(crate) fn add_command(cmd: String) {
    if let Ok(queue) = COMMAND_CHANNEL.try_get() {
        if let Err(_) = queue.push(cmd) {
            println!("WARNING: scancode queue full; dropping keyboard input");
        } else {
            WAKER.wake(); // new
        }
    } else {
        println!("WARNING: scancode queue uninitialized");
    }
}


/////////////////////


// CLI

/////////////////////

fn handle_command(command: &str){
    println!("hello");
}

pub async fn CLI_START() {
    println!("CLI_START running...");
    let mut command_stream = CommandStream::new(); // defined below

    while let Some(line) = command_stream.next().await {
        handle_command(&line);
    }
}