
use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use core::{pin::Pin, task::{Poll, Context}};
use futures_util::stream::Stream;
use futures_util::StreamExt;
use futures_util::task::AtomicWaker;

use crate::println;
use crate::task::system_vars::system_vars;
use crate::task::read_drive::{read_directory, create_file, zero_sector, create_root_directory};

use alloc::boxed::Box;
use alloc::vec::Vec;
use alloc::string::String;

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
pub enum LexType{
    Number,
    Stri,
    Add,
    Mult,
    Div,
    Sub,
    DirPointer,
    Ls,
    Stringy,
    CreateDir,
    ZeroDir,
    CreateRoot,
    CreateFile
}



fn lexer(command: &str) -> Vec<Box<(String, LexType)>> {
    use alloc::{vec::Vec, string::String};

    let mut vec: Vec<Box<(String, LexType)>> = Vec::new();
    let mut tmp = String::new();
    let mut chars = command.char_indices().peekable();

    while let Some((_i, ch)) = chars.next() {
        if ch != '+' && ch != '*' && ch != '/' && ch != '-' && ch != ' '{
            tmp.push(ch);
        }

        let at_end = chars.peek().is_none();
        if ch == ' ' || ch == '+' || ch == '*' || ch == '/' || ch == '-' || at_end {

            if !tmp.is_empty() {
                let token_type = if tmp.to_lowercase()=="dirpointer"{
                    LexType::DirPointer
                }else if tmp.to_lowercase()=="ls"{
                    LexType::Ls
                }else if tmp.to_lowercase()=="cdir" {
                    LexType::CreateDir
                }else if tmp.to_lowercase()=="zb" {
                    LexType::ZeroDir
                }else if tmp.to_lowercase()=="cr" {
                    LexType::CreateRoot
                }else if tmp.to_lowercase()=="cf" {
                    LexType::CreateFile
                }else{
                    LexType::Stringy
                };
                vec.push(Box::new((tmp.clone(), token_type)));
                tmp.clear();
            }
        }
    }

    vec
    


}



unsafe fn dirpointer(){
    println!("Current dir: {}", system_vars.current_dir);
}











fn handle_command(command: &str){
    let tokens = lexer(command);
    println!();
    if tokens.len() >= 3 {
        let t0 = &tokens[0];
        let t1 = &tokens[1];
        let t2 = &tokens[2];
        if t0.1 == LexType::Number && t1.1 == LexType::Add && t2.1 == LexType::Number {
            let n1 = t0.0.parse::<i32>().unwrap_or(0);
            let n2 = t2.0.parse::<i32>().unwrap_or(0);
            println!("{} {}", n1, n2);
            println!("{}", n1 + n2);
        } else if t0.1 == LexType::Number && t1.1 == LexType::Mult && t2.1 == LexType::Number{
            let n1 = t0.0.parse::<i32>().unwrap_or(0);
            let n2 = t2.0.parse::<i32>().unwrap_or(0);
            println!("{}", n1*n2);
        }else if t0.1 == LexType::Number && t1.1 == LexType::Div && t2.1 == LexType::Number{
            let n1 = t0.0.parse::<i32>().unwrap_or(0);
            let n2 = t2.0.parse::<i32>().unwrap_or(0);
            println!("{}", (n1 as f32 /n2 as f32));
        } else if t0.1 == LexType::Number && t1.1 == LexType::Sub && t2.1 == LexType::Number{
            let n1 = t0.0.parse::<i32>().unwrap_or(0);
            let n2 = t2.0.parse::<i32>().unwrap_or(0);
            println!("{}", n1-n2);
        }else{
            println!("Pattern does not match.");
        }
    } else if tokens[0].1 ==LexType::DirPointer{
        unsafe{dirpointer();}
    }else if tokens[0].1 == LexType::Ls{
        if tokens.len()==1{
            unsafe{read_directory(system_vars.current_dir);}
        }else{
            let s: &String = &tokens[1].0;
            let num: u32 = s.parse::<u32>().unwrap_or(0);
            unsafe{read_directory(num)}
        }
    }else if tokens[0].1 == LexType::CreateFile && tokens[1].1 == LexType::Stringy{
        unsafe{create_file(&tokens[1].0, system_vars.current_dir)}
    }
    else if tokens[0].1 == LexType::ZeroDir{
        if tokens.len() == 2{
            let s: &String = &tokens[1].0;
            let num: u32 = s.parse::<u32>().unwrap_or(0);
            unsafe{zero_sector(num)}
        }else{
        
        }
    }
    else if tokens[0].1 == LexType::CreateRoot{
        unsafe{create_root_directory()}
    }else{
        println!("not a valid command");
    }
}

pub async fn CLI_START() {
    println!("CLI_START running...");
    let mut command_stream = CommandStream::new(); // defined below

    while let Some(line) = command_stream.next().await {
        handle_command(&line);
    }
}


