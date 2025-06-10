
use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use core::{pin::Pin, task::{Poll, Context}};
use futures_util::stream::Stream;
use futures_util::StreamExt;
use futures_util::task::AtomicWaker;

use crate::println;

use alloc::boxed::Box;
use alloc::vec::Vec;
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

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum lex_type{
    Number,
    Stri,
    Add,
    Mult,
    Div,
    Sub
}



fn lexer(command: &str) -> Vec<Box<(String, lex_type)>> {
    use alloc::{vec::Vec, string::String};

    let mut vec: Vec<Box<(String, lex_type)>> = Vec::new();
    let mut tmp = String::new();
    let mut chars = command.char_indices().peekable();

    while let Some((_i, ch)) = chars.next() {
        if ch != '+' && ch != '*' && ch != '/' && ch != '-' && ch != ' '{
            tmp.push(ch);
        }

        let at_end = chars.peek().is_none();
        if ch == ' ' || ch == '+' || ch == '*' || ch == '/' || ch == '-' || at_end {
            if !tmp.is_empty() {
                vec.push(Box::new((tmp.clone(), lex_type::Number)));
                tmp.clear();
            }
            if ch == '+' {
                vec.push(Box::new(("+".into(), lex_type::Add)));
            }else if ch == '*'{
                vec.push(Box::new(("*".into(), lex_type::Mult)));
            }else if ch == '/'{
                vec.push(Box::new(("/".into(), lex_type::Div)));
            }else if ch == '-'{
                vec.push(Box::new(("-".into(), lex_type::Sub)));
            }
        }
    }

    vec
}














fn handle_command(command: &str){
    let tokens = lexer(command);
    println!();
    if tokens.len() >= 3 {
        let t0 = &tokens[0];
        let t1 = &tokens[1];
        let t2 = &tokens[2];
        if t0.1 == lex_type::Number && t1.1 == lex_type::Add && t2.1 == lex_type::Number {
            let n1 = t0.0.parse::<i32>().unwrap_or(0);
            let n2 = t2.0.parse::<i32>().unwrap_or(0);
            println!("{} {}", n1, n2);
            println!("{}", n1 + n2);
        } else if t0.1 == lex_type::Number && t1.1 == lex_type::Mult && t2.1 == lex_type::Number{
            let n1 = t0.0.parse::<i32>().unwrap_or(0);
            let n2 = t2.0.parse::<i32>().unwrap_or(0);
            println!("{}", n1*n2);
        }else if t0.1 == lex_type::Number && t1.1 == lex_type::Div && t2.1 == lex_type::Number{
            let n1 = t0.0.parse::<i32>().unwrap_or(0);
            let n2 = t2.0.parse::<i32>().unwrap_or(0);
            println!("{}", (n1 as f32 /n2 as f32));
        } else if t0.1 == lex_type::Number && t1.1 == lex_type::Sub && t2.1 == lex_type::Number{
            let n1 = t0.0.parse::<i32>().unwrap_or(0);
            let n2 = t2.0.parse::<i32>().unwrap_or(0);
            println!("{}", n1-n2);
        }else{
            println!("Pattern does not match.");
        }
    } else {
        println!("Not enough tokens.");
    }
}

pub async fn CLI_START() {
    println!("CLI_START running...");
    let mut command_stream = CommandStream::new(); // defined below

    while let Some(line) = command_stream.next().await {
        handle_command(&line);
    }
}


